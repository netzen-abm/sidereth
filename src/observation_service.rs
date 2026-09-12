use crate::command::{execute_authoritative_command, AtomicCommandPlan, AuthoritativeCommandError};
use crate::persistence::{
    PersistenceError, ResourceWrite, ResourceWriteMode, Revision, UnitOfWorkContext,
    UnitOfWorkError, UnitOfWorkFactory,
};
use crate::{AuthorizationDecision, AuthorizationResult, Id, Observation, ResourceRef, ResourceType};
use serde_json::json;

const OBSERVATION_CREATE_ACTION: &str = "observation.create";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObservationServiceError {
    AuthorizationDenied,
    AuthorizationMismatch,
    AuthorizationExpired,
    Conflict,
    Duplicate,
    InvalidInput,
    Persistence(PersistenceError),
    RollbackFailure {
        operation: Box<Self>,
        rollback: PersistenceError,
    },
    SerializationFailure,
}

impl From<PersistenceError> for ObservationServiceError {
    fn from(error: PersistenceError) -> Self {
        Self::Persistence(error)
    }
}

impl From<AuthoritativeCommandError> for ObservationServiceError {
    fn from(error: AuthoritativeCommandError) -> Self {
        match error {
            AuthoritativeCommandError::Persistence(error) => match error {
                PersistenceError::Conflict => Self::Conflict,
                PersistenceError::Duplicate | PersistenceError::IdempotencyAlreadyClaimed => {
                    Self::Duplicate
                }
                other => Self::Persistence(other),
            },
            AuthoritativeCommandError::InvalidOperation => Self::InvalidInput,
            AuthoritativeCommandError::RollbackFailure {
                operation,
                rollback,
            } => Self::RollbackFailure {
                operation: Box::new(Self::from(*operation)),
                rollback,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservationCommandContext {
    pub actor_ref: ResourceRef,
    pub operation_id: Id,
    pub correlation_id: Id,
    pub now_epoch_seconds: u64,
    pub authorization: AuthorizationResult,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservationCommandResult {
    pub observation_id: Id,
    pub revision: Revision,
    pub event_id: Id,
}

pub struct ObservationService<'a, F> {
    factory: &'a mut F,
}

impl<'a, F> ObservationService<'a, F>
where
    F: UnitOfWorkFactory,
{
    pub fn new(factory: &'a mut F) -> Self {
        Self { factory }
    }

    pub fn create(
        &mut self,
        context: ObservationCommandContext,
        observation: Observation,
    ) -> Result<ObservationCommandResult, ObservationServiceError> {
        validate_authorization(&context, &observation)?;
        observation
            .validate()
            .map_err(|_| ObservationServiceError::InvalidInput)?;

        let mut plan = AtomicCommandPlan::new(context.operation_id.clone())
            .map_err(|_| ObservationServiceError::InvalidInput)?;
        plan.claim_operation()
            .map_err(|_| ObservationServiceError::InvalidInput)?;

        let actor_ref = context.actor_ref.clone();
        let correlation_id = context.correlation_id.clone();
        let operation_id = context.operation_id.clone();
        let authorization = context.authorization.clone();

        execute_authoritative_command(self.factory, plan, move |uow, plan| {
            execute_create_observation(
                uow,
                plan,
                actor_ref,
                correlation_id,
                operation_id,
                authorization,
                observation,
            )
        })
        .map_err(ObservationServiceError::from)
    }
}

fn validate_authorization(
    context: &ObservationCommandContext,
    observation: &Observation,
) -> Result<(), ObservationServiceError> {
    if context.operation_id.is_empty() || context.correlation_id.is_empty() {
        return Err(ObservationServiceError::InvalidInput);
    }
    if context.actor_ref.id.is_empty() {
        return Err(ObservationServiceError::InvalidInput);
    }

    let observation_ref = ResourceRef::new(ResourceType::Observation, observation.observation_id.clone())
        .map_err(|_| ObservationServiceError::InvalidInput)?;
    let expected_action = ResourceRef::new(ResourceType::Action, OBSERVATION_CREATE_ACTION)
        .map_err(|_| ObservationServiceError::InvalidInput)?;

    let authorization = &context.authorization;
    if authorization.decision != AuthorizationDecision::Allow {
        return Err(ObservationServiceError::AuthorizationDenied);
    }
    if authorization.authorization_ref.id.is_empty()
        || authorization.request_id.is_empty()
        || authorization.purpose.trim().is_empty()
    {
        return Err(ObservationServiceError::AuthorizationMismatch);
    }
    if authorization.subject_ref != context.actor_ref
        || authorization.action != expected_action
        || authorization.resource_ref != observation_ref
    {
        return Err(ObservationServiceError::AuthorizationMismatch);
    }
    if authorization.evaluated_at_epoch_seconds > context.now_epoch_seconds {
        return Err(ObservationServiceError::AuthorizationExpired);
    }
    if authorization
        .expires_at_epoch_seconds
        .is_some_and(|expires| context.now_epoch_seconds > expires)
    {
        return Err(ObservationServiceError::AuthorizationExpired);
    }

    Ok(())
}

fn execute_create_observation<C: UnitOfWorkContext>(
    uow: &mut C,
    plan: &AtomicCommandPlan,
    actor_ref: ResourceRef,
    correlation_id: Id,
    operation_id: Id,
    authorization: AuthorizationResult,
    observation: Observation,
) -> Result<ObservationCommandResult, UnitOfWorkError> {
    let observation_id = observation.observation_id.clone();
    let observation_ref = ResourceRef::new(ResourceType::Observation, observation_id.clone())
        .map_err(|_| UnitOfWorkError::InvalidOperation)?;
    let payload = serde_json::to_value(&observation)
        .map_err(|_| UnitOfWorkError::Persistence(PersistenceError::SerializationFailure))?;

    uow.write_resource(ResourceWrite::new(
        observation_ref.clone(),
        observation.schema_version,
        payload,
        ResourceWriteMode::Insert,
    )?)?;

    let event_id = format!("observation-event-{}", operation_id);
    let audit_id = format!("audit-{}", operation_id);
    let provenance_id = format!("provenance-{}", operation_id);

    uow.write_resource(ResourceWrite::new(
        ResourceRef::new(ResourceType::Event, event_id.clone())
            .map_err(|_| UnitOfWorkError::InvalidOperation)?,
        1,
        json!({
            "event_id": event_id,
            "event_type": "observation.created",
            "aggregate_type": "observation",
            "aggregate_id": observation_id,
            "occurred_at": observation.recorded_at,
            "actor_type": actor_ref.resource_type,
            "actor_id": actor_ref.id,
            "schema_version": 1,
            "payload": {
                "observation_ref": observation_ref,
                "authorization_ref": authorization.authorization_ref
            },
            "source_refs": observation.source_refs,
            "correlation_id": correlation_id,
            "causation_id": null
        }),
        ResourceWriteMode::Insert,
    )?)?;

    uow.write_resource(ResourceWrite::new(
        ResourceRef::new(ResourceType::Audit, audit_id.clone())
            .map_err(|_| UnitOfWorkError::InvalidOperation)?,
        1,
        json!({
            "audit_id": audit_id,
            "actor_id": actor_ref.id,
            "action": "observation.created",
            "aggregate_type": "observation",
            "aggregate_id": observation_id,
            "occurred_at": observation.recorded_at,
            "operation_id": operation_id,
            "authorization_ref": authorization.authorization_ref
        }),
        ResourceWriteMode::Insert,
    )?)?;

    uow.write_resource(ResourceWrite::new(
        ResourceRef::new(ResourceType::Provenance, provenance_id.clone())
            .map_err(|_| UnitOfWorkError::InvalidOperation)?,
        1,
        json!({
            "provenance_id": provenance_id,
            "actor_ref": actor_ref,
            "source_refs": observation.source_refs,
            "input_refs": [observation_ref, authorization.authorization_ref],
            "operation": "observation.created",
            "occurred_at": observation.recorded_at
        }),
        ResourceWriteMode::Insert,
    )?)?;

    let _ = plan;
    Ok(ObservationCommandResult {
        observation_id,
        revision: Revision::initial(),
        event_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intelligence::{IntelligenceDataClass, EpistemicStatus};
    use crate::persistence::{ResourceRecord, UnitOfWork};
    use crate::ObservationOrigin;
    use serde_json::Value;
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
            let revision = state
                .get(&write.resource_ref)
                .map(|record| record.revision.next())
                .transpose()?
                .unwrap_or_else(Revision::initial);
            state.insert(
                write.resource_ref.clone(),
                ResourceRecord {
                    resource_ref: write.resource_ref,
                    schema_version: write.schema_version,
                    revision,
                    payload: write.payload,
                },
            );
            Ok(())
        }

        fn link_resources(
            &mut self,
            _link: crate::persistence::ResourceLink,
        ) -> Result<(), UnitOfWorkError> {
            Ok(())
        }
    }

    struct MockUow {
        context: MockContext,
        snapshot: HashMap<ResourceRef, ResourceRecord>,
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
            let mut state = self.context.state.0.borrow_mut();
            state.clear();
            state.extend(self.snapshot);
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
                snapshot: self.state.0.borrow().clone(),
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
            observation_type: crate::ObservationType::Condition,
            observation_origin: ObservationOrigin::DirectObservation,
            observed_at: "2026-09-12T09:00:00Z".into(),
            recorded_at: "2026-09-12T09:01:00Z".into(),
            assertion: json!({"condition": "present"}),
            epistemic_status: EpistemicStatus::Observed,
            source_refs: vec![ResourceRef::new(ResourceType::Evidence, "evidence-1").unwrap()],
            evidence_refs: vec![ResourceRef::new(ResourceType::Evidence, "evidence-1").unwrap()],
            provenance_ref: Some(ResourceRef::new(ResourceType::Provenance, "prov-input-1").unwrap()),
            context_refs: Vec::new(),
            data_class: IntelligenceDataClass::CaseRestricted,
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
                action: ResourceRef::new(ResourceType::Action, OBSERVATION_CREATE_ACTION).unwrap(),
                resource_ref: ResourceRef::new(ResourceType::Observation, "obs-1").unwrap(),
                purpose: "record observation".into(),
                jurisdiction_ref: None,
                data_class: Some("case-restricted".into()),
                decision: AuthorizationDecision::Allow,
                constraints: Vec::new(),
                policy_refs: vec![ResourceRef::new(ResourceType::Other, "policy-1").unwrap()],
                evaluated_at_epoch_seconds: 90,
                expires_at_epoch_seconds: Some(120),
            },
        }
    }

    #[test]
    fn create_writes_observation_event_audit_provenance_and_idempotency_atomically() {
        let mut factory = MockFactory::default();
        let state = factory.state.clone();
        let mut service = ObservationService::new(&mut factory);

        let result = service.create(context(), observation()).unwrap();

        let records = state.0.borrow();
        assert_eq!(result.revision, Revision::initial());
        assert!(records.contains_key(&ResourceRef::new(ResourceType::Observation, "obs-1").unwrap()));
        assert!(records.contains_key(&ResourceRef::new(ResourceType::Event, &result.event_id).unwrap()));
        assert!(records.contains_key(&ResourceRef::new(ResourceType::Audit, "audit-op-1").unwrap()));
        assert!(records.contains_key(&ResourceRef::new(ResourceType::Provenance, "provenance-op-1").unwrap()));
        assert!(records.contains_key(&ResourceRef::new(ResourceType::Idempotency, "op-1").unwrap()));

        let event = records
            .get(&ResourceRef::new(ResourceType::Event, &result.event_id).unwrap())
            .unwrap();
        assert_eq!(event.payload["event_type"], "observation.created");
        assert_eq!(event.payload["payload"]["authorization_ref"]["id"], "auth-1");
    }

    #[test]
    fn mismatched_authorization_resource_is_rejected_before_mutation() {
        let mut factory = MockFactory::default();
        let state = factory.state.clone();
        let mut context = context();
        context.authorization.resource_ref =
            ResourceRef::new(ResourceType::Observation, "other-observation").unwrap();
        let mut service = ObservationService::new(&mut factory);

        assert_eq!(
            service.create(context, observation()),
            Err(ObservationServiceError::AuthorizationMismatch)
        );
        assert!(state.0.borrow().is_empty());
    }

    #[test]
    fn denied_authorization_is_rejected_before_mutation() {
        let mut factory = MockFactory::default();
        let state = factory.state.clone();
        let mut context = context();
        context.authorization.decision = AuthorizationDecision::Deny;
        let mut service = ObservationService::new(&mut factory);

        assert_eq!(
            service.create(context, observation()),
            Err(ObservationServiceError::AuthorizationDenied)
        );
        assert!(state.0.borrow().is_empty());
    }

    #[test]
    fn expired_authorization_is_rejected_before_mutation() {
        let mut factory = MockFactory::default();
        let state = factory.state.clone();
        let mut context = context();
        context.now_epoch_seconds = 121;
        let mut service = ObservationService::new(&mut factory);

        assert_eq!(
            service.create(context, observation()),
            Err(ObservationServiceError::AuthorizationExpired)
        );
        assert!(state.0.borrow().is_empty());
    }

    #[test]
    fn authorization_is_not_an_epistemic_upgrade() {
        let mut factory = MockFactory::default();
        let state = factory.state.clone();
        let mut value = observation();
        value.epistemic_status = EpistemicStatus::Inferred;
        let mut service = ObservationService::new(&mut factory);

        service.create(context(), value).unwrap();
        let record = state
            .0
            .borrow()
            .get(&ResourceRef::new(ResourceType::Observation, "obs-1").unwrap())
            .cloned()
            .unwrap();
        assert_eq!(record.payload["epistemic_status"], "INFERRED");
        assert_ne!(record.payload["epistemic_status"], Value::String("OBSERVED".into()));
    }
}
