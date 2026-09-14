use serde_json::json;
use sidereth_core::persistence::{
    PersistenceError, ResourceLink, ResourceLinkClass, ResourceRecord, ResourceWrite,
    ResourceWriteMode, UnitOfWork, UnitOfWorkContext, UnitOfWorkError, UnitOfWorkFactory,
};
use sidereth_core::{
    AuthorizationDecision, AuthorizationRequest, AuthorizationResult, EpistemicStatus,
    IntelligenceDataClass, Observation, ObservationLifecycleCommand,
    ObservationLifecycleCommandContext, ObservationLifecycleError, ObservationOrigin,
    ObservationType, ResourceRef, ResourceType, Revision,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, Default)]
struct State {
    records: Rc<RefCell<HashMap<ResourceRef, ResourceRecord>>>,
    links: Rc<RefCell<Vec<ResourceLink>>>,
}

struct MockContext {
    state: State,
}

impl UnitOfWorkContext for MockContext {
    fn read_resource(
        &mut self,
        resource_ref: &ResourceRef,
    ) -> Result<Option<ResourceRecord>, UnitOfWorkError> {
        Ok(self.state.records.borrow().get(resource_ref).cloned())
    }

    fn write_resource(&mut self, write: ResourceWrite) -> Result<(), UnitOfWorkError> {
        let mut records = self.state.records.borrow_mut();
        if write.mode == ResourceWriteMode::Insert && records.contains_key(&write.resource_ref) {
            return Err(UnitOfWorkError::Persistence(PersistenceError::Duplicate));
        }
        records.insert(
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
        self.state.links.borrow_mut().push(link);
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
}

impl UnitOfWorkFactory for MockFactory {
    type Uow = MockUow;

    fn begin(&mut self) -> Result<Self::Uow, PersistenceError> {
        Ok(MockUow {
            context: MockContext {
                state: self.state.clone(),
            },
        })
    }
}

fn observation(id: &str, assertion: serde_json::Value) -> Observation {
    Observation {
        observation_id: id.into(),
        schema_version: 1,
        subject_ref: ResourceRef::new(ResourceType::Case, "case-1").unwrap(),
        observation_type: ObservationType::Condition,
        observation_origin: ObservationOrigin::DirectObservation,
        observed_at: "2026-09-13T08:00:00Z".into(),
        recorded_at: "2026-09-13T08:01:00Z".into(),
        assertion,
        epistemic_status: EpistemicStatus::Observed,
        source_refs: vec![],
        evidence_refs: vec![],
        provenance_ref: None,
        context_refs: vec![],
        data_class: IntelligenceDataClass::Restricted,
    }
}

fn seed_prior(factory: &MockFactory, prior: &Observation) {
    factory.state.records.borrow_mut().insert(
        ResourceRef::new(ResourceType::Observation, prior.observation_id.clone()).unwrap(),
        ResourceRecord {
            resource_ref: ResourceRef::new(ResourceType::Observation, prior.observation_id.clone())
                .unwrap(),
            schema_version: 1,
            revision: Revision::initial(),
            payload: serde_json::to_value(prior).unwrap(),
        },
    );
}

fn context(action: &str, observation_id: &str) -> ObservationLifecycleCommandContext {
    let actor_ref = ResourceRef::new(ResourceType::Party, "party-1").unwrap();
    let authorization_ref = ResourceRef::new(ResourceType::Other, "auth-1").unwrap();
    let action_ref = ResourceRef::new(ResourceType::Action, action).unwrap();
    let resource_ref = ResourceRef::new(ResourceType::Observation, observation_id).unwrap();
    let policy_ref = ResourceRef::new(ResourceType::Other, "policy-1").unwrap();
    let authorization_request = AuthorizationRequest {
        request_id: format!("request-{observation_id}"),
        authorization_ref: authorization_ref.clone(),
        subject_ref: actor_ref.clone(),
        action: action_ref.clone(),
        resource_ref: resource_ref.clone(),
        purpose: "observation lifecycle".into(),
        policy_refs: vec![policy_ref.clone()],
        jurisdiction_ref: None,
        data_class: Some("RESTRICTED".into()),
        requested_at_epoch_seconds: 90,
        freshness_seconds: Some(30),
    };
    ObservationLifecycleCommandContext {
        actor_ref,
        operation_id: format!("op-{action}-{observation_id}"),
        correlation_id: format!("corr-{observation_id}"),
        now_epoch_seconds: 100,
        authorization_request: authorization_request.clone(),
        authorization: AuthorizationResult {
            request_id: authorization_request.request_id.clone(),
            authorization_ref,
            subject_ref: authorization_request.subject_ref.clone(),
            action: action_ref,
            resource_ref,
            purpose: authorization_request.purpose.clone(),
            jurisdiction_ref: None,
            data_class: authorization_request.data_class.clone(),
            decision: AuthorizationDecision::Allow,
            constraints: vec![],
            policy_refs: authorization_request.policy_refs.clone(),
            evaluated_at_epoch_seconds: 90,
            expires_at_epoch_seconds: Some(120),
        },
    }
}

#[test]
fn correction_preserves_prior_and_links_new_observation() {
    let mut factory = MockFactory::default();
    let prior = observation("obs-old", json!({"condition":"damaged"}));
    seed_prior(&factory, &prior);
    let state = factory.state.clone();
    let mut command = ObservationLifecycleCommand::new(&mut factory);

    let result = command
        .correct(
            context("observation.correct", "obs-new"),
            ResourceRef::new(ResourceType::Observation, "obs-old").unwrap(),
            observation("obs-new", json!({"condition":"repaired"})),
            "corrected after source review",
        )
        .unwrap();

    assert_eq!(result.prior_observation_id, "obs-old");
    assert!(state
        .records
        .borrow()
        .contains_key(&ResourceRef::new(ResourceType::Observation, "obs-old").unwrap()));
    assert!(state
        .records
        .borrow()
        .contains_key(&ResourceRef::new(ResourceType::Observation, "obs-new").unwrap()));
    let links = state.links.borrow();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].relation, "corrects");
    assert_eq!(links[0].class, Some(ResourceLinkClass::Strong));
    assert_eq!(links[0].target_ref.id, "obs-old");
}

#[test]
fn supersession_is_explicit_and_does_not_delete_prior() {
    let mut factory = MockFactory::default();
    let prior = observation("obs-old", json!({"status":"open"}));
    seed_prior(&factory, &prior);
    let state = factory.state.clone();
    let mut command = ObservationLifecycleCommand::new(&mut factory);

    command
        .supersede(
            context("observation.supersede", "obs-new"),
            ResourceRef::new(ResourceType::Observation, "obs-old").unwrap(),
            observation("obs-new", json!({"status":"closed"})),
            "later observation supersedes earlier record",
        )
        .unwrap();

    assert!(state
        .records
        .borrow()
        .contains_key(&ResourceRef::new(ResourceType::Observation, "obs-old").unwrap()));
    assert_eq!(state.links.borrow()[0].relation, "supersedes");
}

#[test]
fn contradiction_preserves_both_observations() {
    let mut factory = MockFactory::default();
    let prior = observation("obs-old", json!({"condition":"present"}));
    seed_prior(&factory, &prior);
    let state = factory.state.clone();
    let mut command = ObservationLifecycleCommand::new(&mut factory);

    command
        .contradict(
            context("observation.contradict", "obs-new"),
            ResourceRef::new(ResourceType::Observation, "obs-old").unwrap(),
            observation("obs-new", json!({"condition":"absent"})),
            "independent source conflicts with prior observation",
        )
        .unwrap();

    assert_eq!(state.records.borrow().len(), 6);
    assert_eq!(state.links.borrow()[0].relation, "contradicts");
}

#[test]
fn missing_prior_is_rejected_without_creating_new_observation() {
    let mut factory = MockFactory::default();
    let state = factory.state.clone();
    let mut command = ObservationLifecycleCommand::new(&mut factory);

    let result = command.correct(
        context("observation.correct", "obs-new"),
        ResourceRef::new(ResourceType::Observation, "missing").unwrap(),
        observation("obs-new", json!({"condition":"repaired"})),
        "correction basis",
    );

    assert_eq!(
        result,
        Err(ObservationLifecycleError::PriorObservationNotFound)
    );
    assert!(state
        .records
        .borrow()
        .get(&ResourceRef::new(ResourceType::Observation, "obs-new").unwrap())
        .is_none());
    assert!(state.links.borrow().is_empty());
}

#[test]
fn lifecycle_authorization_is_bound_to_exact_operation_and_resource() {
    let mut factory = MockFactory::default();
    let prior = observation("obs-old", json!({"condition":"damaged"}));
    seed_prior(&factory, &prior);
    let state = factory.state.clone();
    let mut bad_context = context("observation.correct", "obs-new");
    bad_context.authorization.resource_ref =
        ResourceRef::new(ResourceType::Observation, "other").unwrap();
    let mut command = ObservationLifecycleCommand::new(&mut factory);

    assert_eq!(
        command.correct(
            bad_context,
            ResourceRef::new(ResourceType::Observation, "obs-old").unwrap(),
            observation("obs-new", json!({"condition":"repaired"})),
            "correction basis",
        ),
        Err(ObservationLifecycleError::AuthorizationMismatch)
    );
    assert_eq!(state.records.borrow().len(), 1);
}

#[test]
fn lifecycle_does_not_resolve_epistemic_conflict_by_itself() {
    let mut factory = MockFactory::default();
    let prior = observation("obs-old", json!({"condition":"present"}));
    seed_prior(&factory, &prior);
    let state = factory.state.clone();
    let mut command = ObservationLifecycleCommand::new(&mut factory);
    let mut new_observation = observation("obs-new", json!({"condition":"absent"}));
    new_observation.epistemic_status = EpistemicStatus::Inferred;

    command
        .contradict(
            context("observation.contradict", "obs-new"),
            ResourceRef::new(ResourceType::Observation, "obs-old").unwrap(),
            new_observation,
            "conflicting source",
        )
        .unwrap();

    let record = state
        .records
        .borrow()
        .get(&ResourceRef::new(ResourceType::Observation, "obs-new").unwrap())
        .cloned()
        .unwrap();
    assert_eq!(record.payload["epistemic_status"], "INFERRED");
}
