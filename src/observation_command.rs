use crate::command::{execute_authoritative_command, AtomicCommandPlan, AuthoritativeCommandError};
use crate::persistence::{
    PersistenceError, ResourceWrite, ResourceWriteMode, Revision, UnitOfWorkContext,
    UnitOfWorkError, UnitOfWorkFactory,
};
use crate::{AuthorizationDecision, AuthorizationResult, Id, Observation, ResourceRef, ResourceType};
use serde_json::json;

const OBSERVATION_CREATE_ACTION: &str = "observation.create";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObservationCommandError {
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
}

impl From<AuthoritativeCommandError> for ObservationCommandError {
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
            AuthoritativeCommandError::RollbackFailure { operation, rollback } => {
                Self::RollbackFailure {
                    operation: Box::new(Self::from(*operation)),
                    rollback,
                }
            }
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

pub struct ObservationCommand<'a, F> {
    factory: &'a mut F,
}

impl<'a, F> ObservationCommand<'a, F>
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
    ) -> Result<ObservationCommandResult, ObservationCommandError> {
        validate_authorization(&context, &observation)?;
        observation
            .validate()
            .map_err(|_| ObservationCommandError::InvalidInput)?;

        let mut plan = AtomicCommandPlan::new(context.operation_id.clone())
            .map_err(|_| ObservationCommandError::InvalidInput)?;
        plan.claim_operation()
            .map_err(|_| ObservationCommandError::InvalidInput)?;

        let actor_ref = context.actor_ref;
        let correlation_id = context.correlation_id;
        let operation_id = context.operation_id;
        let authorization = context.authorization;

        execute_authoritative_command(self.factory, plan, move |uow, plan| {
            persist_observation(
                uow,
                plan,
                actor_ref,
                correlation_id,
                operation_id,
                authorization,
                observation,
            )
        })
        .map_err(ObservationCommandError::from)
    }
}

fn validate_authorization(
    context: &ObservationCommandContext,
    observation: &Observation,
) -> Result<(), ObservationCommandError> {
    if context.operation_id.is_empty()
        || context.correlation_id.is_empty()
        || context.actor_ref.id.is_empty()
    {
        return Err(ObservationCommandError::InvalidInput);
    }

    let observation_ref =
        ResourceRef::new(ResourceType::Observation, observation.observation_id.clone())
            .map_err(|_| ObservationCommandError::InvalidInput)?;
    let expected_action = ResourceRef::new(ResourceType::Action, OBSERVATION_CREATE_ACTION)
        .map_err(|_| ObservationCommandError::InvalidInput)?;
    let expected_data_class = serde_json::to_value(observation.data_class)
        .map_err(|_| ObservationCommandError::InvalidInput)?;
    let expected_data_class = expected_data_class
        .as_str()
        .ok_or(ObservationCommandError::InvalidInput)?;
    let authorization = &context.authorization;

    if authorization.decision != AuthorizationDecision::Allow {
        return Err(ObservationCommandError::AuthorizationDenied);
    }
    if authorization.request_id.is_empty()
        || authorization.authorization_ref.id.is_empty()
        || authorization.purpose.trim().is_empty()
    {
        return Err(ObservationCommandError::AuthorizationMismatch);
    }
    if authorization.subject_ref != context.actor_ref
        || authorization.action != expected_action
        || authorization.resource_ref != observation_ref
        || authorization.data_class.as_deref() != Some(expected_data_class)
    {
        return Err(ObservationCommandError::AuthorizationMismatch);
    }
    if authorization.evaluated_at_epoch_seconds > context.now_epoch_seconds
        || authorization
            .expires_at_epoch_seconds
            .is_some_and(|expires| context.now_epoch_seconds > expires)
    {
        return Err(ObservationCommandError::AuthorizationExpired);
    }

    Ok(())
}

fn persist_observation<C: UnitOfWorkContext>(
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
    let recorded_at = observation.recorded_at.clone();
    let source_refs = observation.source_refs.clone();
    let authorization_ref = authorization.authorization_ref.clone();
    let actor_id = actor_ref.id.clone();
    let actor_type = actor_ref.resource_type;
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
            "aggregate_id": observation_id.clone(),
            "occurred_at": recorded_at.clone(),
            "actor_type": actor_type,
            "actor_id": actor_id.clone(),
            "schema_version": 1,
            "payload": {
                "observation_ref": observation_ref.clone(),
                "authorization_ref": authorization_ref.clone()
            },
            "source_refs": source_refs.clone(),
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
            "actor_id": actor_id,
            "action": "observation.created",
            "aggregate_type": "observation",
            "aggregate_id": observation_id.clone(),
            "occurred_at": recorded_at.clone(),
            "operation_id": operation_id,
            "authorization_ref": authorization_ref.clone()
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
            "source_refs": source_refs,
            "input_refs": [observation_ref, authorization_ref],
            "operation": "observation.created",
            "occurred_at": recorded_at
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
