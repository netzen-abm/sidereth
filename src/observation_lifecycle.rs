use crate::command::{
    execute_authoritative_command, AtomicCommandPlan, AuthoritativeCommandError,
};
use crate::persistence::{
    PersistenceError, ResourceLink, ResourceLinkClass, ResourceWrite, ResourceWriteMode,
    UnitOfWorkContext, UnitOfWorkError, UnitOfWorkFactory,
};
use crate::{
    AuthorizationDecision, AuthorizationResult, Id, Observation, ResourceRef, ResourceType,
};
use serde_json::json;

const OBSERVATION_CORRECT_ACTION: &str = "observation.correct";
const OBSERVATION_SUPERSEDE_ACTION: &str = "observation.supersede";
const OBSERVATION_CONTRADICT_ACTION: &str = "observation.contradict";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservationLifecycleOperation {
    Correction,
    Supersession,
    Contradiction,
}

impl ObservationLifecycleOperation {
    fn action(self) -> &'static str {
        match self {
            Self::Correction => OBSERVATION_CORRECT_ACTION,
            Self::Supersession => OBSERVATION_SUPERSEDE_ACTION,
            Self::Contradiction => OBSERVATION_CONTRADICT_ACTION,
        }
    }

    fn relation(self) -> &'static str {
        match self {
            Self::Correction => "corrected_by",
            Self::Supersession => "superseded_by",
            Self::Contradiction => "contradicts",
        }
    }

    fn event_type(self) -> &'static str {
        match self {
            Self::Correction => "observation.corrected",
            Self::Supersession => "observation.superseded",
            Self::Contradiction => "observation.contradicted",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObservationLifecycleError {
    AuthorizationDenied,
    AuthorizationMismatch,
    AuthorizationExpired,
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
pub struct ObservationLifecycleContext {
    pub actor_ref: ResourceRef,
    pub operation_id: Id,
    pub correlation_id: Id,
    pub now_epoch_seconds: u64,
    pub authorization: AuthorizationResult,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservationLifecycleResult {
    pub prior_observation_id: Id,
    pub new_observation_id: Id,
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
        operation: ObservationLifecycleOperation,
        context: ObservationLifecycleContext,
        prior: Observation,
        replacement: Observation,
    ) -> Result<ObservationLifecycleResult, ObservationLifecycleError> {
        validate_context(&context, operation, &prior, &replacement)?;
        prior
            .validate()
            .map_err(|_| ObservationLifecycleError::InvalidInput)?;
        replacement
            .validate()
            .map_err(|_| ObservationLifecycleError::InvalidInput)?;
        if prior.observation_id == replacement.observation_id {
            return Err(ObservationLifecycleError::InvalidInput);
        }

        let mut plan = AtomicCommandPlan::new(context.operation_id.clone())
            .map_err(|_| ObservationLifecycleError::InvalidInput)?;
        plan.claim_operation()
            .map_err(|_| ObservationLifecycleError::InvalidInput)?;

        let actor_ref = context.actor_ref;
        let correlation_id = context.correlation_id;
        let operation_id = context.operation_id;
        let authorization = context.authorization;
        execute_authoritative_command(self.factory, plan, move |uow, plan| {
            persist_lifecycle(
                uow,
                plan,
                operation,
                actor_ref,
                correlation_id,
                operation_id,
                authorization,
                prior,
                replacement,
            )
        })
        .map_err(ObservationLifecycleError::from)
    }
}

fn validate_context(
    context: &ObservationLifecycleContext,
    operation: ObservationLifecycleOperation,
    prior: &Observation,
    replacement: &Observation,
) -> Result<(), ObservationLifecycleError> {
    if context.operation_id.is_empty()
        || context.correlation_id.is_empty()
        || context.actor_ref.id.is_empty()
        || prior.observation_id.is_empty()
        || replacement.observation_id.is_empty()
    {
        return Err(ObservationLifecycleError::InvalidInput);
    }

    let prior_ref = ResourceRef::new(ResourceType::Observation, prior.observation_id.clone())
        .map_err(|_| ObservationLifecycleError::InvalidInput)?;
    let expected_action = ResourceRef::new(ResourceType::Action, operation.action())
        .map_err(|_| ObservationLifecycleError::InvalidInput)?;
    let expected_data_class = serde_json::to_value(replacement.data_class)
        .map_err(|_| ObservationLifecycleError::InvalidInput)?;
    let expected_data_class = expected_data_class
        .as_str()
        .ok_or(ObservationLifecycleError::InvalidInput)?;
    let authorization = &context.authorization;

    if authorization.decision != AuthorizationDecision::Allow {
        return Err(ObservationLifecycleError::AuthorizationDenied);
    }
    if authorization.request_id.is_empty()
        || authorization.authorization_ref.id.is_empty()
        || authorization.purpose.trim().is_empty()
    {
        return Err(ObservationLifecycleError::AuthorizationMismatch);
    }
    if authorization.subject_ref != context.actor_ref
        || authorization.action != expected_action
        || authorization.resource_ref != prior_ref
        || authorization.data_class.as_deref() != Some(expected_data_class)
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

fn persist_lifecycle<C: UnitOfWorkContext>(
    uow: &mut C,
    plan: &AtomicCommandPlan,
    operation: ObservationLifecycleOperation,
    actor_ref: ResourceRef,
    correlation_id: Id,
    operation_id: Id,
    authorization: AuthorizationResult,
    prior: Observation,
    replacement: Observation,
) -> Result<ObservationLifecycleResult, UnitOfWorkError> {
    let prior_id = prior.observation_id.clone();
    let replacement_id = replacement.observation_id.clone();
    let prior_ref = ResourceRef::new(ResourceType::Observation, prior_id.clone())
        .map_err(|_| UnitOfWorkError::InvalidOperation)?;
    let replacement_ref = ResourceRef::new(ResourceType::Observation, replacement_id.clone())
        .map_err(|_| UnitOfWorkError::InvalidOperation)?;
    let actor_id = actor_ref.id.clone();
    let actor_type = actor_ref.resource_type;
    let occurred_at = replacement.recorded_at.clone();
    let authorization_ref = authorization.authorization_ref.clone();

    // The replacement/new observation is authored as a new canonical identity.
    // The prior observation is never rewritten or deleted.
    uow.write_resource(ResourceWrite::new(
        replacement_ref.clone(),
        replacement.schema_version as u16,
        serde_json::to_value(&replacement)
            .map_err(|_| UnitOfWorkError::Persistence(PersistenceError::SerializationFailure))?,
        ResourceWriteMode::Insert,
    )?)?;

    // Strong is appropriate because both lifecycle endpoints are authored in
    // this transaction and therefore must exist as canonical local resources.
    uow.link_resources(ResourceLink::new_with_class(
        prior_ref.clone(),
        operation.relation(),
        replacement_ref.clone(),
        ResourceLinkClass::Strong,
    )?)?;

    let event_id = format!("observation-lifecycle-event-{}", operation_id);
    let audit_id = format!("observation-lifecycle-audit-{}", operation_id);
    let provenance_id = format!("observation-lifecycle-provenance-{}", operation_id);

    uow.write_resource(ResourceWrite::new(
        ResourceRef::new(ResourceType::Event, event_id.clone())
            .map_err(|_| UnitOfWorkError::InvalidOperation)?,
        1,
        json!({
            "event_id": event_id,
            "event_type": operation.event_type(),
            "aggregate_type": "observation",
            "aggregate_id": prior_id.clone(),
            "occurred_at": occurred_at.clone(),
            "actor_type": actor_type,
            "actor_id": actor_id.clone(),
            "schema_version": 1,
            "payload": {
                "prior_observation_ref": prior_ref.clone(),
                "replacement_observation_ref": replacement_ref.clone(),
                "relation": operation.relation(),
                "authorization_ref": authorization_ref.clone()
            },
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
            "aggregate_id": prior_id.clone(),
            "occurred_at": occurred_at.clone(),
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
            "source_refs": replacement.source_refs,
            "input_refs": [prior_ref, replacement_ref, authorization_ref],
            "operation": operation.event_type(),
            "occurred_at": occurred_at
        }),
        ResourceWriteMode::Insert,
    )?)?;

    let _ = plan;
    Ok(ObservationLifecycleResult {
        prior_observation_id: prior_id,
        new_observation_id: replacement_id,
        event_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EpistemicStatus, IntelligenceDataClass, ObservationOrigin, ObservationType};

    fn observation(id: &str) -> Observation {
        Observation {
            observation_id: id.into(),
            schema_version: 1,
            subject_ref: ResourceRef::new(ResourceType::Incident, "incident-1").unwrap(),
            observation_type: ObservationType::Condition,
            observation_origin: ObservationOrigin::DirectObservation,
            observed_at: "2026-09-12T10:00:00Z".into(),
            recorded_at: "2026-09-12T10:01:00Z".into(),
            assertion: json!({"condition":"damaged"}),
            epistemic_status: EpistemicStatus::Observed,
            source_refs: vec![],
            evidence_refs: vec![],
            provenance_ref: None,
            context_refs: vec![],
            data_class: IntelligenceDataClass::Confidential,
        }
    }

    #[test]
    fn lifecycle_operations_have_distinct_canonical_actions_and_relations() {
        assert_eq!(
            ObservationLifecycleOperation::Correction.action(),
            "observation.correct"
        );
        assert_eq!(
            ObservationLifecycleOperation::Supersession.action(),
            "observation.supersede"
        );
        assert_eq!(
            ObservationLifecycleOperation::Contradiction.action(),
            "observation.contradict"
        );
        assert_ne!(
            ObservationLifecycleOperation::Correction.relation(),
            ObservationLifecycleOperation::Supersession.relation()
        );
    }

    #[test]
    fn replacement_never_reuses_prior_identity() {
        let prior = observation("obs-1");
        let replacement = observation("obs-2");
        assert_ne!(prior.observation_id, replacement.observation_id);
    }
}
