use serde_json::json;
use sidereth::persistence::{
    PersistenceError, ResourceRecord, ResourceWrite, ResourceWriteMode, UnitOfWork,
    UnitOfWorkContext, UnitOfWorkError, UnitOfWorkFactory,
};
use sidereth::{
    AuthorizationDecision, AuthorizationResult, EpistemicStatus, IntelligenceDataClass,
    Observation, ObservationCommand, ObservationCommandContext, ObservationCommandError,
    ObservationOrigin, ObservationType, ResourceRef, ResourceType, Revision,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, Default)]
struct State(Rc<RefCell<HashMap<ResourceRef, ResourceRecord>>>);

struct MockContext {
    state: State,
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

    fn link_resources(
        &mut self,
        _link: sidereth::persistence::ResourceLink,
    ) -> Result<(), UnitOfWorkError> {
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

fn observation() -> Observation {
    Observation {
        observation_id: "obs-1".into(),
        schema_version: 1,
        subject_ref: ResourceRef::new(ResourceType::Case, "case-1").unwrap(),
        observation_type: ObservationType::Condition,
        observation_origin: ObservationOrigin::DirectObservation,
        observed_at: "2026-09-12T09:00:00Z".into(),
        recorded_at: "2026-09-12T09:01:00Z".into(),
        assertion: json!({"condition": "present"}),
        epistemic_status: EpistemicStatus::Observed,
        source_refs: vec![ResourceRef::new(ResourceType::Evidence, "evidence-1").unwrap()],
        evidence_refs: vec![ResourceRef::new(ResourceType::Evidence, "evidence-1").unwrap()],
        provenance_ref: Some(
            ResourceRef::new(ResourceType::Provenance, "input-prov-1").unwrap(),
        ),
        context_refs: Vec::new(),
        data_class: IntelligenceDataClass::Restricted,
    }
}

fn context() -> ObservationCommandContext {
    let actor_ref = ResourceRef::new(ResourceType::Party, "party-1").unwrap();
    ObservationCommandContext {
        actor_ref: actor_ref.clone(),
        operation_id: "op-1".into(),
        correlation_id: "corr-1".into(),
        now_epoch_seconds: 100,
        authorization: AuthorizationResult {
            request_id: "request-1".into(),
            authorization_ref: ResourceRef::new(ResourceType::Other, "auth-1").unwrap(),
            subject_ref: actor_ref,
            action: ResourceRef::new(ResourceType::Action, "observation.create").unwrap(),
            resource_ref: ResourceRef::new(ResourceType::Observation, "obs-1").unwrap(),
            purpose: "record observation".into(),
            jurisdiction_ref: None,
            data_class: Some("restricted".into()),
            decision: AuthorizationDecision::Allow,
            constraints: Vec::new(),
            policy_refs: vec![ResourceRef::new(ResourceType::Other, "policy-1").unwrap()],
            evaluated_at_epoch_seconds: 90,
            expires_at_epoch_seconds: Some(120),
        },
    }
}

#[test]
fn create_is_authorized_and_atomic() {
    let mut factory = MockFactory::default();
    let state = factory.state.clone();
    let mut command = ObservationCommand::new(&mut factory);

    let result = command.create(context(), observation()).unwrap();

    let records = state.0.borrow();
    assert_eq!(result.revision, Revision::initial());
    assert!(records.contains_key(
        &ResourceRef::new(ResourceType::Observation, "obs-1").unwrap()
    ));
    assert!(records.contains_key(
        &ResourceRef::new(ResourceType::Event, &result.event_id).unwrap()
    ));
    assert!(records.contains_key(
        &ResourceRef::new(ResourceType::Audit, "audit-op-1").unwrap()
    ));
    assert!(records.contains_key(
        &ResourceRef::new(ResourceType::Provenance, "provenance-op-1").unwrap()
    ));
    assert!(records.contains_key(
        &ResourceRef::new(ResourceType::Idempotency, "op-1").unwrap()
    ));
}

#[test]
fn authorization_must_bind_exact_observation_resource() {
    let mut factory = MockFactory::default();
    let state = factory.state.clone();
    let mut context = context();
    context.authorization.resource_ref =
        ResourceRef::new(ResourceType::Observation, "other").unwrap();
    let mut command = ObservationCommand::new(&mut factory);

    assert_eq!(
        command.create(context, observation()),
        Err(ObservationCommandError::AuthorizationMismatch)
    );
    assert!(state.0.borrow().is_empty());
}

#[test]
fn denied_or_expired_authorization_cannot_mutate() {
    let mut factory = MockFactory::default();
    let state = factory.state.clone();
    let mut denied = context();
    denied.authorization.decision = AuthorizationDecision::Deny;
    let mut command = ObservationCommand::new(&mut factory);
    assert_eq!(
        command.create(denied, observation()),
        Err(ObservationCommandError::AuthorizationDenied)
    );

    let mut expired = context();
    expired.now_epoch_seconds = 121;
    assert_eq!(
        command.create(expired, observation()),
        Err(ObservationCommandError::AuthorizationExpired)
    );
    assert!(state.0.borrow().is_empty());
}

#[test]
fn authorization_does_not_upgrade_epistemic_status() {
    let mut factory = MockFactory::default();
    let state = factory.state.clone();
    let mut value = observation();
    value.epistemic_status = EpistemicStatus::Inferred;
    let mut command = ObservationCommand::new(&mut factory);

    command.create(context(), value).unwrap();
    let record = state
        .0
        .borrow()
        .get(&ResourceRef::new(ResourceType::Observation, "obs-1").unwrap())
        .cloned()
        .unwrap();
    assert_eq!(record.payload["epistemic_status"], "INFERRED");
}
