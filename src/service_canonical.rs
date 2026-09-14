use crate::authorization::{
    AuthorizationDecision, AuthorizationEvaluator, AuthorizationRequest, AuthorizationResult,
};
use crate::command::{execute_authoritative_command, AtomicCommandPlan, AuthoritativeCommandError};
use crate::persistence::{
    PersistenceError, ResourceWrite, ResourceWriteMode, Revision, UnitOfWorkContext,
    UnitOfWorkError, UnitOfWorkFactory,
};
use crate::{Case, CaseState, Id, ResourceRef, ResourceType};
use serde_json::json;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceError {
    AuthorizationDenied,
    AuthorizationMismatch,
    AuthorizationExpired,
    Conflict,
    Duplicate,
    InvalidInput,
    NotFound,
    Persistence(PersistenceError),
    RollbackFailure {
        operation: Box<Self>,
        rollback: PersistenceError,
    },
    SerializationFailure,
}

impl From<PersistenceError> for ServiceError {
    fn from(error: PersistenceError) -> Self {
        Self::Persistence(error)
    }
}

impl From<AuthoritativeCommandError> for ServiceError {
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
pub enum CaseCommand {
    Create { case: Case },
    Transition {
        case_id: Id,
        expected_revision: Revision,
        next: CaseState,
    },
}

/// Canonical command context required to authorize a protected case operation.
/// No authorization semantics are inferred from actor identity alone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandContext {
    pub request_id: Id,
    pub authorization_ref: ResourceRef,
    pub actor_id: Id,
    pub operation_id: Id,
    pub correlation_id: Id,
    pub purpose: String,
    pub policy_refs: Vec<ResourceRef>,
    pub jurisdiction_ref: Option<ResourceRef>,
    pub data_class: Option<String>,
    pub requested_at_epoch_seconds: u64,
    pub freshness_seconds: Option<u64>,
    pub now_epoch_seconds: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandResult {
    pub case_id: Id,
    pub revision: Revision,
    pub event_id: Id,
}

pub struct CaseService<'a, F, E> {
    factory: &'a mut F,
    evaluator: &'a E,
}

impl<'a, F, E> CaseService<'a, F, E>
where
    F: UnitOfWorkFactory,
    E: AuthorizationEvaluator,
{
    pub fn new(factory: &'a mut F, evaluator: &'a E) -> Self {
        Self { factory, evaluator }
    }

    pub fn execute(
        &mut self,
        context: CommandContext,
        command: CaseCommand,
    ) -> Result<CommandResult, ServiceError> {
        let (case_id, action) = command_target(&command)?;
        let subject_ref = resource_ref(ResourceType::Party, context.actor_id.clone())
            .map_err(|_| ServiceError::InvalidInput)?;
        let resource_ref = resource_ref(ResourceType::Case, case_id.clone())
            .map_err(|_| ServiceError::InvalidInput)?;
        let authorization_request = AuthorizationRequest {
            request_id: context.request_id,
            authorization_ref: context.authorization_ref,
            subject_ref,
            action,
            resource_ref,
            purpose: context.purpose,
            policy_refs: context.policy_refs,
            jurisdiction_ref: context.jurisdiction_ref,
            data_class: context.data_class,
            requested_at_epoch_seconds: context.requested_at_epoch_seconds,
            freshness_seconds: context.freshness_seconds,
        };
        let authorization = self.evaluator.evaluate(&authorization_request);
        authorize(&authorization, &authorization_request, context.now_epoch_seconds)?;

        let mut plan = AtomicCommandPlan::new(context.operation_id.clone())
            .map_err(|_| ServiceError::InvalidInput)?;
        plan.claim_operation()
            .map_err(|_| ServiceError::InvalidInput)?;

        let actor_id = context.actor_id;
        let correlation_id = context.correlation_id;
        let operation_id = context.operation_id;

        execute_authoritative_command(self.factory, plan, move |uow, plan| {
            execute_case_command(uow, plan, actor_id, correlation_id, operation_id, command)
        })
        .map_err(ServiceError::from)
    }
}

fn authorize(
    result: &AuthorizationResult,
    request: &AuthorizationRequest,
    now_epoch_seconds: u64,
) -> Result<(), ServiceError> {
    if result.request_id != request.request_id
        || result.authorization_ref != request.authorization_ref
        || result.subject_ref != request.subject_ref
        || result.action != request.action
        || result.resource_ref != request.resource_ref
        || result.purpose != request.purpose
        || result.jurisdiction_ref != request.jurisdiction_ref
        || result.data_class != request.data_class
    {
        return Err(ServiceError::AuthorizationMismatch);
    }
    if result.evaluated_at_epoch_seconds != now_epoch_seconds {
        return Err(ServiceError::AuthorizationMismatch);
    }
    if result
        .expires_at_epoch_seconds
        .is_some_and(|expires_at| now_epoch_seconds > expires_at)
    {
        return Err(ServiceError::AuthorizationExpired);
    }
    match result.decision {
        AuthorizationDecision::Allow => Ok(()),
        AuthorizationDecision::Deny | AuthorizationDecision::NotApplicable => {
            Err(ServiceError::AuthorizationDenied)
        }
    }
}

fn execute_case_command<C: UnitOfWorkContext>(
    uow: &mut C,
    plan: &AtomicCommandPlan,
    actor_id: Id,
    correlation_id: Id,
    operation_id: Id,
    command: CaseCommand,
) -> Result<CommandResult, UnitOfWorkError> {
    let (case_id, revision, event_type, payload) = match command {
        CaseCommand::Create { case } => {
            let case_id = case.case_id.clone();
            let case_ref = resource_ref(ResourceType::Case, case_id.clone())?;
            let payload = serde_json::to_value(&case).map_err(|_| {
                UnitOfWorkError::Persistence(PersistenceError::SerializationFailure)
            })?;
            uow.write_resource(ResourceWrite::new(
                case_ref,
                1,
                payload,
                ResourceWriteMode::Insert,
            )?)?;
            (
                case_id,
                Revision::initial(),
                "case.created",
                json!({ "state": case.state }),
            )
        }
        CaseCommand::Transition {
            case_id,
            expected_revision,
            next,
        } => {
            let case_ref = resource_ref(ResourceType::Case, case_id.clone())?;
            let record = uow
                .read_resource(&case_ref)?
                .ok_or(UnitOfWorkError::Persistence(PersistenceError::NotFound))?;
            if record.revision != expected_revision {
                return Err(UnitOfWorkError::Persistence(PersistenceError::Conflict));
            }
            let mut case: Case = serde_json::from_value(record.payload).map_err(|_| {
                UnitOfWorkError::Persistence(PersistenceError::SerializationFailure)
            })?;
            case.transition(next)
                .map_err(|_| UnitOfWorkError::Persistence(PersistenceError::ValidationFailure))?;
            let payload = serde_json::to_value(&case).map_err(|_| {
                UnitOfWorkError::Persistence(PersistenceError::SerializationFailure)
            })?;
            let write = ResourceWrite::new(
                case_ref,
                record.schema_version,
                payload,
                ResourceWriteMode::Upsert,
            )?
            .with_expected_revision(expected_revision);
            let revision = expected_revision.next()?;
            uow.write_resource(write)?;
            (
                case_id,
                revision,
                "case.transition",
                json!({ "state": case.state }),
            )
        }
    };

    let event_id = format!("event-{}-{}", operation_id, revision.value);
    let audit_id = format!("audit-{}", operation_id);
    let provenance_id = format!("provenance-{}", operation_id);

    uow.write_resource(ResourceWrite::new(
        resource_ref(ResourceType::Event, event_id.clone())?,
        1,
        json!({
            "event_id": event_id,
            "event_type": event_type,
            "aggregate_type": "case",
            "aggregate_id": case_id,
            "occurred_at": "service",
            "actor_type": "user",
            "actor_id": actor_id,
            "schema_version": 1,
            "payload": payload,
            "source_refs": [],
            "correlation_id": correlation_id,
            "causation_id": null
        }),
        ResourceWriteMode::Insert,
    )?)?;

    uow.write_resource(ResourceWrite::new(
        resource_ref(ResourceType::Audit, audit_id)?,
        1,
        json!({
            "audit_id": format!("audit-{}", operation_id),
            "actor_id": actor_id,
            "action": event_type,
            "aggregate_type": "case",
            "aggregate_id": case_id,
            "occurred_at": "service",
            "operation_id": operation_id
        }),
        ResourceWriteMode::Insert,
    )?)?;

    uow.write_resource(ResourceWrite::new(
        resource_ref(ResourceType::Provenance, provenance_id)?,
        1,
        json!({
            "provenance_id": format!("provenance-{}", operation_id),
            "actor_ref": ResourceRef::new(ResourceType::Party, actor_id).ok(),
            "source_refs": [],
            "input_refs": [resource_ref(ResourceType::Case, case_id.clone())?],
            "operation": event_type,
            "occurred_at": "service"
        }),
        ResourceWriteMode::Insert,
    )?)?;

    let _ = plan;
    Ok(CommandResult {
        case_id,
        revision,
        event_id,
    })
}

fn resource_ref(resource_type: ResourceType, id: Id) -> Result<ResourceRef, UnitOfWorkError> {
    ResourceRef::new(resource_type, id).map_err(|_| UnitOfWorkError::InvalidOperation)
}

fn command_target(command: &CaseCommand) -> Result<(Id, ResourceRef), ServiceError> {
    match command {
        CaseCommand::Create { case } if case.case_id.is_empty() => Err(ServiceError::InvalidInput),
        CaseCommand::Create { case } => Ok((
            case.case_id.clone(),
            ResourceRef::new(ResourceType::Action, "case.create")
                .map_err(|_| ServiceError::InvalidInput)?,
        )),
        CaseCommand::Transition { case_id, .. } if case_id.is_empty() => {
            Err(ServiceError::InvalidInput)
        }
        CaseCommand::Transition { case_id, .. } => Ok((
            case_id.clone(),
            ResourceRef::new(ResourceType::Action, "case.update")
                .map_err(|_| ServiceError::InvalidInput)?,
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authorization::{AuthorizationConstraint, AuthorizationRule, StaticAuthorizationEvaluator};
    use crate::persistence::{ResourceRecord, UnitOfWork, UnitOfWorkContext};
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;

    #[derive(Clone, Default)]
    struct State(Rc<RefCell<HashMap<ResourceRef, ResourceRecord>>>);

    struct MockContext { state: State }

    impl UnitOfWorkContext for MockContext {
        fn read_resource(&mut self, resource_ref: &ResourceRef) -> Result<Option<ResourceRecord>, UnitOfWorkError> {
            Ok(self.state.0.borrow().get(resource_ref).cloned())
        }
        fn write_resource(&mut self, write: ResourceWrite) -> Result<(), UnitOfWorkError> {
            let mut state = self.state.0.borrow_mut();
            let current = state.get(&write.resource_ref).cloned();
            match (write.mode, current, write.expected_revision) {
                (ResourceWriteMode::Insert, Some(_), _) => Err(UnitOfWorkError::Persistence(PersistenceError::Duplicate)),
                (ResourceWriteMode::Insert, None, _) => { state.insert(write.resource_ref.clone(), ResourceRecord { resource_ref: write.resource_ref, schema_version: write.schema_version, revision: Revision::initial(), payload: write.payload }); Ok(()) }
                (ResourceWriteMode::Upsert, Some(current), Some(expected)) if current.revision != expected => Err(UnitOfWorkError::Persistence(PersistenceError::Conflict)),
                (ResourceWriteMode::Upsert, Some(_), Some(expected)) => { state.insert(write.resource_ref.clone(), ResourceRecord { resource_ref: write.resource_ref, schema_version: write.schema_version, revision: expected.next()?, payload: write.payload }); Ok(()) }
                (ResourceWriteMode::Upsert, None, Some(_)) => Err(UnitOfWorkError::Persistence(PersistenceError::Conflict)),
                (ResourceWriteMode::Upsert, Some(current), None) => { state.insert(write.resource_ref.clone(), ResourceRecord { resource_ref: write.resource_ref, schema_version: write.schema_version, revision: current.revision.next()?, payload: write.payload }); Ok(()) }
                (ResourceWriteMode::Upsert, None, None) => { state.insert(write.resource_ref.clone(), ResourceRecord { resource_ref: write.resource_ref, schema_version: write.schema_version, revision: Revision::initial(), payload: write.payload }); Ok(()) }
            }
        }
        fn link_resources(&mut self, _link: crate::persistence::ResourceLink) -> Result<(), UnitOfWorkError> { Ok(()) }
    }

    struct MockUow { context: MockContext, snapshot: HashMap<ResourceRef, ResourceRecord> }
    impl UnitOfWork for MockUow {
        type Context = MockContext;
        fn execute<R, F>(&mut self, operation: F) -> Result<R, UnitOfWorkError> where F: FnOnce(&mut Self::Context) -> Result<R, UnitOfWorkError> { operation(&mut self.context) }
        fn commit(self) -> Result<(), PersistenceError> { Ok(()) }
        fn rollback(self) -> Result<(), PersistenceError> { let mut state = self.context.state.0.borrow_mut(); state.clear(); state.extend(self.snapshot); Ok(()) }
    }

    #[derive(Clone, Default)]
    struct MockFactory { state: State }
    impl UnitOfWorkFactory for MockFactory {
        type Uow = MockUow;
        fn begin(&mut self) -> Result<Self::Uow, PersistenceError> { let snapshot = self.state.0.borrow().clone(); Ok(MockUow { context: MockContext { state: self.state.clone() }, snapshot }) }
    }

    fn reference(resource_type: ResourceType, id: &str) -> ResourceRef { ResourceRef::new(resource_type, id).unwrap() }

    fn context(action: &str, case_id: &str, actor: &str, operation: &str) -> CommandContext {
        CommandContext {
            request_id: format!("request-{operation}"),
            authorization_ref: reference(ResourceType::Other, "auth-1"),
            actor_id: actor.into(),
            operation_id: operation.into(),
            correlation_id: format!("correlation-{operation}"),
            purpose: "case operation".into(),
            policy_refs: vec![reference(ResourceType::Other, "policy-1")],
            jurisdiction_ref: Some(reference(ResourceType::Jurisdiction, "jurisdiction-1")),
            data_class: Some("case-restricted".into()),
            requested_at_epoch_seconds: 100,
            freshness_seconds: Some(60),
            now_epoch_seconds: 110,
        }
    }

    fn evaluator(actor: &str, action: &str, case_id: &str, decision: AuthorizationDecision) -> StaticAuthorizationEvaluator {
        StaticAuthorizationEvaluator {
            rules: vec![AuthorizationRule {
                policy_ref: reference(ResourceType::Other, "policy-1"),
                subject_ref: reference(ResourceType::Party, actor),
                action: reference(ResourceType::Action, action),
                resource_ref: reference(ResourceType::Case, case_id),
                purpose: "case operation".into(),
                jurisdiction_ref: Some(reference(ResourceType::Jurisdiction, "jurisdiction-1")),
                data_class: Some("case-restricted".into()),
                decision,
                constraints: vec![AuthorizationConstraint { key: "mode".into(), value: "case".into() }],
            }],
            now_epoch_seconds: 110,
        }
    }

    #[test]
    fn canonical_allow_permits_case_creation() {
        let mut factory = MockFactory::default();
        let evaluator = evaluator("user-1", "case.create", "case-1", AuthorizationDecision::Allow);
        let mut service = CaseService::new(&mut factory, &evaluator);
        let result = service.execute(context("case.create", "case-1", "user-1", "op-1"), CaseCommand::Create { case: Case::new("case-1".into()).unwrap() }).unwrap();
        assert_eq!(result.revision.value, 0);
    }

    #[test]
    fn deny_does_not_claim_idempotency() {
        let mut factory = MockFactory::default();
        let evaluator = evaluator("user-1", "case.create", "case-1", AuthorizationDecision::Deny);
        let mut service = CaseService::new(&mut factory, &evaluator);
        assert_eq!(service.execute(context("case.create", "case-1", "user-1", "op-1"), CaseCommand::Create { case: Case::new("case-1".into()).unwrap() }), Err(ServiceError::AuthorizationDenied));
        assert!(factory.state.0.borrow().is_empty());
    }

    #[test]
    fn mismatched_subject_is_rejected_even_when_result_allows_other_subject() {
        let mut factory = MockFactory::default();
        let evaluator = evaluator("user-1", "case.create", "case-1", AuthorizationDecision::Allow);
        let mut service = CaseService::new(&mut factory, &evaluator);
        assert_eq!(service.execute(context("case.create", "case-1", "user-2", "op-1"), CaseCommand::Create { case: Case::new("case-1".into()).unwrap() }), Err(ServiceError::AuthorizationDenied));
    }

    #[test]
    fn mismatched_purpose_cannot_use_allow_result() {
        let mut factory = MockFactory::default();
        let evaluator = evaluator("user-1", "case.create", "case-1", AuthorizationDecision::Allow);
        let mut command_context = context("case.create", "case-1", "user-1", "op-1");
        command_context.purpose = "different purpose".into();
        let mut service = CaseService::new(&mut factory, &evaluator);
        assert_eq!(service.execute(command_context, CaseCommand::Create { case: Case::new("case-1".into()).unwrap() }), Err(ServiceError::AuthorizationDenied));
    }

    #[test]
    fn expired_authorization_is_rejected() {
        let mut factory = MockFactory::default();
        let evaluator = evaluator("user-1", "case.create", "case-1", AuthorizationDecision::Allow);
        let mut command_context = context("case.create", "case-1", "user-1", "op-1");
        command_context.now_epoch_seconds = 171;
        let mut service = CaseService::new(&mut factory, &evaluator);
        assert_eq!(service.execute(command_context, CaseCommand::Create { case: Case::new("case-1".into()).unwrap() }), Err(ServiceError::AuthorizationDenied));
    }

    #[test]
    fn not_applicable_is_fail_closed() {
        let mut factory = MockFactory::default();
        let evaluator = StaticAuthorizationEvaluator { rules: vec![], now_epoch_seconds: 110 };
        let mut service = CaseService::new(&mut factory, &evaluator);
        assert_eq!(service.execute(context("case.create", "case-1", "user-1", "op-1"), CaseCommand::Create { case: Case::new("case-1".into()).unwrap() }), Err(ServiceError::AuthorizationDenied));
    }
}
