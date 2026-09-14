use crate::command::{execute_authoritative_command, AtomicCommandPlan, AuthoritativeCommandError};
use crate::persistence::{
    PersistenceError, ResourceLink, ResourceLinkClass, ResourceWrite, ResourceWriteMode, Revision,
    UnitOfWorkContext, UnitOfWorkError, UnitOfWorkFactory,
};
use crate::{
    AuthorizationDecision, AuthorizationRequest, AuthorizationResult, Id, Observation, ResourceRef,
    ResourceType,
};
use serde_json::json;

const OBSERVATION_CORRECT_ACTION: &str = "observation.correct";
const OBSERVATION_SUPERSEDE_ACTION: &str = "observation.supersede";
const OBSERVATION_CONTRADICT_ACTION: &str = "observation.contradict";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservationLifecycleOperation {
    Correct,
    Supersede,
    Contradict,
}

impl ObservationLifecycleOperation {
    fn action_id(self) -> &'static str {
        match self {
            Self::Correct => OBSERVATION_CORRECT_ACTION,
            Self::Supersede => OBSERVATION_SUPERSEDE_ACTION,
            Self::Contradict => OBSERVATION_CONTRADICT_ACTION,
        }
    }

    fn relation(self) -> &'static str {
        match self {
            Self::Correct => "corrects",
            Self::Supersede => "supersedes",
            Self::Contradict => "contradicts",
        }
    }

    fn event_type(self) -> &'static str {
        match self {
            Self::Correct => "observation.corrected",
            Self::Supersede => "observation.superseded",
            Self::Contradict => "observation.contradicted",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObservationLifecycleError {
    AuthorizationDenied,
    AuthorizationMismatch,
    AuthorizationExpired,
    PriorObservationNotFound,
    SameObservation,
    InvalidInput,
    Conflict,
    Duplicate,
    Persistence(PersistenceError),
    RollbackFailure {
        operation: Box<Self>,
        rollback: PersistenceError,
    },
}

impl From<AuthoritativeCommandError> for ObservationLifecycleError {
    fn from(error: AuthoritativeCommandError) -> Self {
        match error {
            AuthoritativeCommandError::Persistence(error) => match error {
                PersistenceError::Conflict => Self::Conflict,
                PersistenceError::Duplicate | PersistenceError::IdempotencyAlreadyClaimed => {
                    Self::Duplicate
                }
                PersistenceError::NotFound => Self::PriorObservationNotFound,
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
pub struct ObservationLifecycleCommandContext {
    pub actor_ref: ResourceRef,
    pub operation_id: Id,
    pub correlation_id: Id,
    pub now_epoch_seconds: u64,
    pub authorization_request: AuthorizationRequest,
    pub authorization: AuthorizationResult,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservationLifecycleCommandResult {
    pub observation_id: Id,
    pub prior_observation_id: Id,
    pub revision: Revision,
    pub event_id: Id,
}

pub struct ObservationLifecycleCommand<'a, F> {
    factory: &'a mut F,
}

impl<'a, F> ObservationLifecycleCommand<'a, F>
where
    F: UnitOfWorkFactory,
{
    pub fn new(factory: &'a mut F) -> Self {
        Self { factory }
    }

    pub fn apply(
        &mut self,
        context: ObservationLifecycleCommandContext,
        operation: ObservationLifecycleOperation,
        prior_observation: ResourceRef,
        observation: Observation,
        reason: impl Into<String>,
    ) -> Result<ObservationLifecycleCommandResult, ObservationLifecycleError> {
        let reason = reason.into();
        validate_authorization(&context, operation, &observation)?;
        if prior_observation.resource_type != ResourceType::Observation
            || prior_observation.id.is_empty()
        {
            return Err(ObservationLifecycleError::InvalidInput);
        }
        let new_ref = ResourceRef::new(
            ResourceType::Observation,
            observation.observation_id.clone(),
        )
        .map_err(|_| ObservationLifecycleError::InvalidInput)?;
        if prior_observation == new_ref {
            return Err(ObservationLifecycleError::SameObservation);
        }
        if reason.trim().is_empty() {
            return Err(ObservationLifecycleError::InvalidInput);
        }
        observation
            .validate()
            .map_err(|_| ObservationLifecycleError::InvalidInput)?;

        let mut plan = AtomicCommandPlan::new(context.operation_id.clone())
            .map_err(|_| ObservationLifecycleError::InvalidInput)?;
        plan.claim_operation()
            .map_err(|_| ObservationLifecycleError::InvalidInput)?;

        let persist_context = ObservationLifecyclePersistContext {
            actor_ref: context.actor_ref,
            correlation_id: context.correlation_id,
            operation_id: context.operation_id,
            authorization: context.authorization,
            operation,
            prior_observation,
            observation,
            reason,
        };

        execute_authoritative_command(self.factory, plan, move |uow, _plan| {
            persist_lifecycle(uow, persist_context)
        })
        .map_err(ObservationLifecycleError::from)
    }

    pub fn correct(
        &mut self,
        context: ObservationLifecycleCommandContext,
        prior_observation: ResourceRef,
        observation: Observation,
        reason: impl Into<String>,
    ) -> Result<ObservationLifecycleCommandResult, ObservationLifecycleError> {
        self.apply(
            context,
            ObservationLifecycleOperation::Correct,
            prior_observation,
            observation,
            reason,
        )
    }

    pub fn supersede(
        &mut self,
        context: ObservationLifecycleCommandContext,
        prior_observation: ResourceRef,
        observation: Observation,
        reason: impl Into<String>,
    ) -> Result<ObservationLifecycleCommandResult, ObservationLifecycleError> {
        self.apply(
            context,
            ObservationLifecycleOperation::Supersede,
            prior_observation,
            observation,
            reason,
        )
    }

    pub fn contradict(
        &mut self,
        context: ObservationLifecycleCommandContext,
        prior_observation: ResourceRef,
        observation: Observation,
        reason: impl Into<String>,
    ) -> Result<ObservationLifecycleCommandResult, ObservationLifecycleError> {
        self.apply(
            context,
            ObservationLifecycleOperation::Contradict,
            prior_observation,
            observation,
            reason,
        )
    }
}

fn validate_authorization(
    context: &ObservationLifecycleCommandContext,
    operation: ObservationLifecycleOperation,
    observation: &Observation,
) -> Result<(), ObservationLifecycleError> {
    if context.operation_id.is_empty()
        || context.correlation_id.is_empty()
        || context.actor_ref.id.is_empty()
    {
        return Err(ObservationLifecycleError::InvalidInput);
    }
    let observation_ref = ResourceRef::new(
        ResourceType::Observation,
        observation.observation_id.clone(),
    )
    .map_err(|_| ObservationLifecycleError::InvalidInput)?;
    let expected_action = ResourceRef::new(ResourceType::Action, operation.action_id())
        .map_err(|_| ObservationLifecycleError::InvalidInput)?;
    let expected_data_class = serde_json::to_value(observation.data_class)
        .map_err(|_| ObservationLifecycleError::InvalidInput)?;
    let expected_data_class = expected_data_class
        .as_str()
        .ok_or(ObservationLifecycleError::InvalidInput)?;
    let authorization_request = &context.authorization_request;
    let authorization = &context.authorization;
    if authorization_request.request_id.is_empty()
        || authorization_request.authorization_ref.id.is_empty()
        || authorization_request.subject_ref.id.is_empty()
        || authorization_request.action.id.is_empty()
        || authorization_request.resource_ref.id.is_empty()
        || authorization_request.purpose.trim().is_empty()
    {
        return Err(ObservationLifecycleError::AuthorizationMismatch);
    }
    if authorization_request.subject_ref != context.actor_ref
        || authorization_request.action != expected_action
        || authorization_request.resource_ref != observation_ref
        || authorization_request.data_class.as_deref() != Some(expected_data_class)
    {
        return Err(ObservationLifecycleError::AuthorizationMismatch);
    }
    if authorization_request.requested_at_epoch_seconds > context.now_epoch_seconds
        || authorization_request
            .freshness_seconds
            .is_some_and(|freshness| {
                context.now_epoch_seconds - authorization_request.requested_at_epoch_seconds
                    > freshness
            })
    {
        return Err(ObservationLifecycleError::AuthorizationExpired);
    }
    if authorization.decision != AuthorizationDecision::Allow {
        return Err(ObservationLifecycleError::AuthorizationDenied);
    }
    if authorization.request_id != authorization_request.request_id
        || authorization.authorization_ref != authorization_request.authorization_ref
        || authorization.subject_ref != authorization_request.subject_ref
        || authorization.action != authorization_request.action
        || authorization.resource_ref != authorization_request.resource_ref
        || authorization.purpose != authorization_request.purpose
        || authorization.jurisdiction_ref != authorization_request.jurisdiction_ref
        || authorization.data_class != authorization_request.data_class
        || authorization.policy_refs != authorization_request.policy_refs
    {
        return Err(ObservationLifecycleError::AuthorizationMismatch);
    }
    if authorization.evaluated_at_epoch_seconds > context.now_epoch_seconds
        || authorization
            .expires_at_epoch_seconds
            .is_some_and(|expires| context.now_epoch_seconds > expires)
    {
        return Err(ObservationLifecycleError::AuthorizationExpired);
    }
    Ok(())
}

struct ObservationLifecyclePersistContext {
    actor_ref: ResourceRef,
    correlation_id: Id,
    operation_id: Id,
    authorization: AuthorizationResult,
    operation: ObservationLifecycleOperation,
    prior_observation: ResourceRef,
    observation: Observation,
    reason: String,
}

fn persist_lifecycle<C: UnitOfWorkContext>(
    uow: &mut C,
    context: ObservationLifecyclePersistContext,
) -> Result<ObservationLifecycleCommandResult, UnitOfWorkError> {
    let ObservationLifecyclePersistContext {
        actor_ref,
        correlation_id,
        operation_id,
        authorization,
        operation,
        prior_observation,
        observation,
        reason,
    } = context;

    let prior = uow
        .read_resource(&prior_observation)?
        .ok_or(UnitOfWorkError::Persistence(PersistenceError::NotFound))?;
    if prior.resource_ref.resource_type != ResourceType::Observation {
        return Err(UnitOfWorkError::InvalidOperation);
    }
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
        observation.schema_version as u16,
        payload,
        ResourceWriteMode::Insert,
    )?)?;

    uow.link_resources(ResourceLink::new_with_class(
        observation_ref.clone(),
        operation.relation(),
        prior_observation.clone(),
        ResourceLinkClass::Strong,
    )?)?;

    let event_id = format!("observation-lifecycle-event-{}", operation_id);
    let audit_id = format!("audit-{}", operation_id);
    let provenance_id = format!("provenance-{}", operation_id);

    uow.write_resource(ResourceWrite::new(
        ResourceRef::new(ResourceType::Event, event_id.clone())
            .map_err(|_| UnitOfWorkError::InvalidOperation)?,
        1,
        json!({
            "event_id": event_id,
            "event_type": operation.event_type(),
            "aggregate_type": "observation",
            "aggregate_id": observation_id.clone(),
            "occurred_at": recorded_at.clone(),
            "actor_type": actor_type,
            "actor_id": actor_id.clone(),
            "schema_version": 1,
            "payload": {
                "observation_ref": observation_ref.clone(),
                "prior_observation_ref": prior_observation.clone(),
                "relation": operation.relation(),
                "reason": reason.clone(),
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
            "action": operation.event_type(),
            "aggregate_type": "observation",
            "aggregate_id": observation_id.clone(),
            "occurred_at": recorded_at.clone(),
            "operation_id": operation_id,
            "authorization_ref": authorization_ref.clone(),
            "prior_observation_ref": prior_observation.clone(),
            "reason": reason.clone()
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
            "input_refs": [observation_ref.clone(), prior_observation.clone(), authorization_ref],
            "operation": operation.event_type(),
            "occurred_at": recorded_at
        }),
        ResourceWriteMode::Insert,
    )?)?;

    Ok(ObservationLifecycleCommandResult {
        observation_id,
        prior_observation_id: prior_observation.id,
        revision: Revision::initial(),
        event_id,
    })
}
