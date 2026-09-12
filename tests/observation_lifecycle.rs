use serde_json::json;
use sidereth_core::persistence::{
    PersistenceError, ResourceLink, ResourceRecord, ResourceWrite, ResourceWriteMode, UnitOfWork,
    UnitOfWorkContext, UnitOfWorkError, UnitOfWorkFactory,
};
use sidereth_core::{
    AuthorizationDecision, AuthorizationResult, EpistemicStatus, IntelligenceDataClass,
    Observation, ObservationLifecycleCommand, ObservationLifecycleContext,
    ObservationLifecycleError, ObservationLifecycleOperation, ObservationOrigin, ObservationType,
    ResourceRef, ResourceType, Revision,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, Default)]
struct State(Rc<RefCell<HashMap<ResourceRef, ResourceRecord>>>);

struct MockContext {
    state: State,
    links: Rc<RefCell<Vec<ResourceLink>>>,
}

impl UnitOfWorkContext for MockContext {
    fn read_resource(
        &mut self,
        resource_ref: &ResourceRef,
    ) -> Result<Option<ResourceRecord>, UnitOfWorkError> {
        Ok(self.state.0.borrow().get(resource_ref).cloned())
    }

    fn write_resource(&mut self, write: ResourceWrite) -> Result<(), UnitOfWorkError> {
        let mut state = self.state.0.borrow_mut();
        if write.mode == ResourceWriteMode::Insert && state.contains_key(&write.resource_ref) {
            return Err(UnitOfWorkError::Persistence(PersistenceError::Duplicate));
        }
        state.insert(
            write.resource_ref.clone(),
            ResourceRecord {
                resource_ref: write.resource_ref,
                schema_version: write.schema_version,
                revision: Revision::initial(),
                payload: write.payload,
            },
        );
        Ok(())
    }

    fn link_resources(&mut self, link: ResourceLink) -> Result<(), UnitOfWorkError> {
        self.links.borrow_mut().push(link);
        Ok(())
    }
}

struct MockUow {
    context: MockContext,
}

impl UnitOfWork for MockUow {
    type Context = MockContext;

    fn execute<R, F>(&mut self, operation: F) -> Result<R, UnitOfWorkError>
    where
        F: FnOnce(&mut Self::Context) -> Result<R, UnitOfWorkError>,
    {
        operation(&mut self.context)
    }

    fn commit(self) -> Result<(), PersistenceError> {
        Ok(())
    }

    fn rollback(self) -> Result<(), PersistenceError> {
        Ok(())
    }
}

#[derive(Clone, Default)]
struct MockFactory {
    state: State,
    links: Rc<RefCell<Vec<ResourceLink>>>,
}

impl UnitOfWorkFactory for MockFactory {
    type Uow = MockUow;

    fn begin(&mut self) -> Result<Self::Uow, PersistenceError> {
        Ok(MockUow {
            context: MockContext {
                state: self.state.clone(),
                links: self.links.clone(),
            },
        })
    }
}

fn observation(id: &str, assertion: &str) -> Observation {
    Observation {
        observation_id: id.into(),
        schema_version: 1,
        subject_ref: ResourceRef::new(ResourceType::Case, "case-1").unwrap(),
        observation_type: ObservationType::Condition,
        observation_origin: ObservationOrigin::DirectObservation,
        observed_at: "2026-09-12T09:00:00Z".into(),
        recorded_at: "2026-09-12T09:02:00Z".into(),
        assertion: json!({"condition": assertion}),
        epistemic_status: EpistemicStatus::Observed,
        source_refs: vec![ResourceRef::new(ResourceType::Evidence, "evidence-1").unwrap()],
        evidence_refs: vec![ResourceRef::new(ResourceType::Evidence, "evidence-1").unwrap()],
        provenance_ref: None,
        context_refs: Vec::new(),
        data_class: IntelligenceDataClass::Restricted,
    }
}

fn context(
    operation: ObservationLifecycleOperation,
    prior_id: &str,
) -> ObservationLifecycleContext {
    let actor_ref = ResourceRef::new(ResourceType::Party, "party-1").unwrap();
    ObservationLifecycleContext {
        actor_ref: actor_ref.clone(),
        operation_id: format!("op-{}", operation.action_for_test()),
        correlation_id: "corr-1".into(),
        now_epoch_seconds: 100,
        authorization: AuthorizationResult {
            request_id: "request-1".into(),
            authorization_ref: ResourceRef::new(ResourceType::Other, "auth-1").unwrap(),
            subject_ref: actor_ref,
            action: ResourceRef::new(ResourceType::Action, operation.action_for_test()).unwrap(),
            resource_ref: ResourceRef::new(ResourceType::Observation, prior_id).unwrap(),
            purpose: "preserve observation lifecycle".into(),
            jurisdiction_ref: None,
            data_class: Some("RESTRICTED".into()),
            decision: AuthorizationDecision::Allow,
            constraints: Vec::new(),
            policy_refs: vec![ResourceRef::new(ResourceType::Other, "policy-1").unwrap()],
            evaluated_at_epoch_seconds: 90,
            expires_at_epoch_seconds: Some(120),
        },
    }
}

trait OperationTestExt {
    fn action_for_test(self) -> &'static str;
}

impl OperationTestExt for ObservationLifecycleOperation {
    fn action_for_test(self) -> &'static str {
        match self {
            ObservationLifecycleOperation::Correction => "observation.correct",
            ObservationLifecycleOperation::Supersession => "observation.supersede",
            ObservationLifecycleOperation::Contradiction => "observation.contradict",
        }
    }
}

#[test]
fn correction_preserves_original_and_creates_explicit_link() {
    let mut factory = MockFactory::default();
    let prior = observation("obs-1", "damaged");
    let replacement = observation("obs-2", "repaired");
    let prior_ref = ResourceRef::new(ResourceType::Observation, "obs-1").unwrap();
    factory.state.0.borrow_mut().insert(
        prior_ref.clone(),
        ResourceRecord {
            resource_ref: prior_ref.clone(),
            schema_version: 1,
            revision: Revision::initial(),
            payload: serde_json::to_value(&prior).unwrap(),
        },
    );

    let mut command = ObservationLifecycleCommand::new(&mut factory);
    let result = command
        .apply(
            ObservationLifecycleOperation::Correction,
            context(ObservationLifecycleOperation::Correction, "obs-1"),
            prior,
            replacement,
        )
        .unwrap();

    let records = factory.state.0.borrow();
    assert!(records.contains_key(&prior_ref));
    assert!(records.contains_key(&ResourceRef::new(ResourceType::Observation, "obs-2").unwrap()));
    assert_eq!(result.prior_observation_id, "obs-1");
    assert_eq!(result.new_observation_id, "obs-2");
    assert_eq!(factory.links.borrow().len(), 1);
    assert_eq!(factory.links.borrow()[0].relation, "corrected_by");
}

#[test]
fn supersession_is_distinct_from_correction() {
    assert_ne!(
        ObservationLifecycleOperation::Supersession.action_for_test(),
        ObservationLifecycleOperation::Correction.action_for_test()
    );
}

#[test]
fn contradiction_does_not_require_or_create_a_winner() {
    let mut factory = MockFactory::default();
    let prior = observation("obs-1", "present");
    let contradictory = observation("obs-2", "absent");
    let prior_ref = ResourceRef::new(ResourceType::Observation, "obs-1").unwrap();
    factory.state.0.borrow_mut().insert(
        prior_ref.clone(),
        ResourceRecord {
            resource_ref: prior_ref.clone(),
            schema_version: 1,
            revision: Revision::initial(),
            payload: serde_json::to_value(&prior).unwrap(),
        },
    );

    let mut command = ObservationLifecycleCommand::new(&mut factory);
    command
        .apply(
            ObservationLifecycleOperation::Contradiction,
            context(ObservationLifecycleOperation::Contradiction, "obs-1"),
            prior,
            contradictory,
        )
        .unwrap();

    let records = factory.state.0.borrow();
    assert!(records.contains_key(&prior_ref));
    assert!(records.contains_key(&ResourceRef::new(ResourceType::Observation, "obs-2").unwrap()));
    assert_eq!(factory.links.borrow()[0].relation, "contradicts");
}

#[test]
fn wrong_lifecycle_authorization_cannot_mutate() {
    let mut factory = MockFactory::default();
    let prior = observation("obs-1", "present");
    let replacement = observation("obs-2", "absent");
    let mut ctx = context(ObservationLifecycleOperation::Correction, "obs-1");
    ctx.authorization.action =
        ResourceRef::new(ResourceType::Action, "observation.supersede").unwrap();
    let mut command = ObservationLifecycleCommand::new(&mut factory);

    assert_eq!(
        command.apply(
            ObservationLifecycleOperation::Correction,
            ctx,
            prior,
            replacement,
        ),
        Err(ObservationLifecycleError::AuthorizationMismatch)
    );
    assert!(factory.state.0.borrow().is_empty());
}
