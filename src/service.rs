use crate::authorization::{AuthorizationDecision, AuthorizationResult};
use crate::command::{execute_authoritative_command, AtomicCommandPlan, AuthoritativeCommandError};
use crate::persistence::{
    PersistenceError, ResourceWrite, ResourceWriteMode, Revision, UnitOfWorkContext,
    UnitOfWorkError, UnitOfWorkFactory,
};
use crate::{Case, CaseState, Id, ResourceRef, ResourceType};
use serde_json::json;

const CASE_CREATE_ACTION: &str = "case.create";
const CASE_TRANSITION_ACTION: &str = "case.transition";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceError {
    AuthorizationDenied,
    AuthorizationMismatch,
    AuthorizationInvalid,
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
    Create {
        case: Case,
    },
    Transition {
        case_id: Id,
        expected_revision: Revision,
        next: CaseState,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandContext {
    pub actor_id: Id,
    pub operation_id: Id,
    pub correlation_id: Id,
    pub purpose: String,
    pub jurisdiction_ref: Option<ResourceRef>,
    pub requested_data_class: Option<String>,
    pub now_epoch_seconds: u64,
    pub authorization: AuthorizationResult,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandResult {
    pub case_id: Id,
    pub revision: Revision,
    pub event_id: Id,
}

pub struct CaseService<'a, F> {
    factory: &'a mut F,
}

impl<'a, F> CaseService<'a, F>
where
    F: UnitOfWorkFactory,
{
    pub fn new(factory: &'a mut F) -> Self {
        Self { factory }
    }

    pub fn execute(
        &mut self,
        context: CommandContext,
        command: CaseCommand,
    ) -> Result<CommandResult, ServiceError> {
        let (case_ref, action_ref) = command_target(&command)?;
        validate_authorization(&context, &case_ref, &action_ref)?;

        let mut plan = AtomicCommandPlan::new(context.operation_id.clone())
            .map_err(|_| ServiceError::InvalidInput)?;
        plan.claim_operation()
            .map_err(|_| ServiceError::InvalidInput)?;

        let actor_id = context.actor_id.clone();
        let correlation_id = context.correlation_id.clone();
        let operation_id = context.operation_id.clone();

        execute_authoritative_command(self.factory, plan, move |uow, plan| {
            execute_case_command(uow, plan, actor_id, correlation_id, operation_id, command)
        })
        .map_err(ServiceError::from)
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

fn command_target(command: &CaseCommand) -> Result<(ResourceRef, ResourceRef), ServiceError> {
    match command {
        CaseCommand::Create { case } if case.case_id.is_empty() => Err(ServiceError::InvalidInput),
        CaseCommand::Create { case } => Ok((
            ResourceRef::new(ResourceType::Case, case.case_id.clone())
                .map_err(|_| ServiceError::InvalidInput)?,
            ResourceRef::new(ResourceType::Action, CASE_CREATE_ACTION)
                .map_err(|_| ServiceError::InvalidInput)?,
        )),
        CaseCommand::Transition { case_id, .. } if case_id.is_empty() => {
            Err(ServiceError::InvalidInput)
        }
        CaseCommand::Transition { case_id, .. } => Ok((
            ResourceRef::new(ResourceType::Case, case_id.clone())
                .map_err(|_| ServiceError::InvalidInput)?,
            ResourceRef::new(ResourceType::Action, CASE_TRANSITION_ACTION)
                .map_err(|_| ServiceError::InvalidInput)?,
        )),
    }
}

fn validate_authorization(
    context: &CommandContext,
    resource_ref: &ResourceRef,
    action_ref: &ResourceRef,
) -> Result<(), ServiceError> {
    let authorization = &context.authorization;

    match authorization.decision {
        AuthorizationDecision::Allow => {}
        AuthorizationDecision::Deny | AuthorizationDecision::NotApplicable => {
            return Err(ServiceError::AuthorizationDenied)
        }
    }

    if authorization.authorization_ref.id.is_empty()
        || authorization.request_id.is_empty()
        || authorization.subject_ref.id.is_empty()
        || authorization.action.id.is_empty()
        || authorization.resource_ref.id.is_empty()
        || authorization.purpose.is_empty()
    {
        return Err(ServiceError::AuthorizationInvalid);
    }

    let expected_subject = ResourceRef::new(ResourceType::Party, context.actor_id.clone())
        .map_err(|_| ServiceError::AuthorizationInvalid)?;
    if authorization.subject_ref != expected_subject
        || authorization.action != *action_ref
        || authorization.resource_ref != *resource_ref
        || authorization.purpose != context.purpose
        || authorization.jurisdiction_ref != context.jurisdiction_ref
        || authorization.data_class != context.requested_data_class
    {
        return Err(ServiceError::AuthorizationMismatch);
    }

    if context.now_epoch_seconds < authorization.evaluated_at_epoch_seconds {
        return Err(ServiceError::AuthorizationInvalid);
    }
    if authorization
        .expires_at_epoch_seconds
        .is_some_and(|expires_at| context.now_epoch_seconds > expires_at)
    {
        return Err(ServiceError::AuthorizationExpired);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authorization::{AuthorizationConstraint, AuthorizationDecision};
    use crate::persistence::{ResourceRecord, UnitOfWork, UnitOfWorkContext};
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
            let current = state.get(&write.resource_ref).cloned();
            match (write.mode, current, write.expected_revision) {
                (ResourceWriteMode::Insert, Some(_), _) => {
                    Err(UnitOfWorkError::Persistence(PersistenceError::Duplicate))
                }
                (ResourceWriteMode::Insert, None, _) => {
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
                (ResourceWriteMode::Upsert, Some(current), Some(expected))
                    if current.revision != expected =>
                {
                    Err(UnitOfWorkError::Persistence(PersistenceError::Conflict))
                }
                (ResourceWriteMode::Upsert, Some(_current), Some(expected)) => {
                    state.insert(
                        write.resource_ref.clone(),
                        ResourceRecord {
                            resource_ref: write.resource_ref,
                            schema_version: write.schema_version,
                            revision: expected.next()?,
                            payload: write.payload,
                        },
                    );
                    Ok(())
                }
                (ResourceWriteMode::Upsert, None, Some(_)) => {
                    Err(UnitOfWorkError::Persistence(PersistenceError::Conflict))
                }
                (ResourceWriteMode::Upsert, Some(current), None) => {
                    state.insert(
                        write.resource_ref.clone(),
                        ResourceRecord {
                            resource_ref: write.resource_ref,
                            schema_version: write.schema_version,
                            revision: current.revision.next()?,
                            payload: write.payload,
                        },
                    );
                    Ok(())
                }
                (ResourceWriteMode::Upsert, None, None) => {
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
            }
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
            let snapshot = self.state.0.borrow().clone();
            Ok(MockUow {
                context: MockContext {
                    state: self.state.clone(),
                },
                snapshot,
            })
        }
    }

    fn reference(resource_type: ResourceType, id: &str) -> ResourceRef {
        ResourceRef::new(resource_type, id).unwrap()
    }

    fn authorization(
        decision: AuthorizationDecision,
        subject_id: &str,
        action_id: &str,
        case_id: &str,
        purpose: &str,
        data_class: Option<&str>,
        evaluated_at: u64,
        expires_at: Option<u64>,
    ) -> AuthorizationResult {
        AuthorizationResult {
            request_id: "request-1".into(),
            authorization_ref: reference(ResourceType::Other, "auth-1"),
            subject_ref: reference(ResourceType::Party, subject_id),
            action: reference(ResourceType::Action, action_id),
            resource_ref: reference(ResourceType::Case, case_id),
            purpose: purpose.into(),
            jurisdiction_ref: Some(reference(ResourceType::Jurisdiction, "jurisdiction-1")),
            data_class: data_class.map(str::to_owned),
            decision,
            constraints: vec![AuthorizationConstraint {
                key: "access_mode".into(),
                value: "write".into(),
            }],
            policy_refs: vec![reference(ResourceType::Other, "policy-1")],
            evaluated_at_epoch_seconds: evaluated_at,
            expires_at_epoch_seconds: expires_at,
        }
    }

    fn context(authorization: AuthorizationResult) -> CommandContext {
        CommandContext {
            actor_id: "user-1".into(),
            operation_id: "op-1".into(),
            correlation_id: "corr-1".into(),
            purpose: "case management".into(),
            jurisdiction_ref: Some(reference(ResourceType::Jurisdiction, "jurisdiction-1")),
            requested_data_class: Some("case-restricted".into()),
            now_epoch_seconds: 110,
            authorization,
        }
    }

    fn allowed_context() -> CommandContext {
        context(authorization(
            AuthorizationDecision::Allow,
            "user-1",
            CASE_CREATE_ACTION,
            "case-1",
            "case management",
            Some("case-restricted"),
            100,
            Some(160),
        ))
    }

    #[test]
    fn create_writes_case_event_audit_provenance_and_idempotency_atomically() {
        let mut factory = MockFactory::default();
        let state = factory.state.clone();
        let mut service = CaseService::new(&mut factory);
        let result = service
            .execute(
                allowed_context(),
                CaseCommand::Create {
                    case: Case::new("case-1".into()).unwrap(),
                },
            )
            .unwrap();
        assert_eq!(result.revision.value, 0);
        let records = state.0.borrow();
        assert!(records.contains_key(&reference(ResourceType::Case, "case-1")));
        assert!(records.contains_key(&reference(ResourceType::Event, &result.event_id)));
        assert!(records.contains_key(&reference(ResourceType::Audit, "audit-op-1")));
        assert!(records.contains_key(&reference(ResourceType::Provenance, "provenance-op-1")));
        assert!(records.contains_key(&reference(ResourceType::Idempotency, "op-1")));
    }

    #[test]
    fn denied_authorization_does_not_claim_idempotency() {
        let mut factory = MockFactory::default();
        let mut denied = allowed_context();
        denied.authorization.decision = AuthorizationDecision::Deny;
        let mut service = CaseService::new(&mut factory);
        assert_eq!(
            service.execute(
                denied,
                CaseCommand::Create {
                    case: Case::new("case-1".into()).unwrap(),
                },
            ),
            Err(ServiceError::AuthorizationDenied)
        );
        assert!(factory.state.0.borrow().is_empty());
    }

    #[test]
    fn wrong_subject_is_rejected_before_mutation() {
        let mut factory = MockFactory::default();
        let mut authorization = allowed_context();
        authorization.authorization.subject_ref = reference(ResourceType::Party, "user-2");
        let mut service = CaseService::new(&mut factory);
        assert_eq!(
            service.execute(
                authorization,
                CaseCommand::Create {
                    case: Case::new("case-1".into()).unwrap(),
                },
            ),
            Err(ServiceError::AuthorizationMismatch)
        );
        assert!(factory.state.0.borrow().is_empty());
    }

    #[test]
    fn wrong_action_is_rejected_before_mutation() {
        let mut factory = MockFactory::default();
        let mut authorization = allowed_context();
        authorization.authorization.action = reference(ResourceType::Action, "case.delete");
        let mut service = CaseService::new(&mut factory);
        assert_eq!(
            service.execute(
                authorization,
                CaseCommand::Create {
                    case: Case::new("case-1".into()).unwrap(),
                },
            ),
            Err(ServiceError::AuthorizationMismatch)
        );
    }

    #[test]
    fn wrong_resource_is_rejected_before_mutation() {
        let mut factory = MockFactory::default();
        let mut authorization = allowed_context();
        authorization.authorization.resource_ref = reference(ResourceType::Case, "case-2");
        let mut service = CaseService::new(&mut factory);
        assert_eq!(
            service.execute(
                authorization,
                CaseCommand::Create {
                    case: Case::new("case-1".into()).unwrap(),
                },
            ),
            Err(ServiceError::AuthorizationMismatch)
        );
    }

    #[test]
    fn wrong_purpose_is_rejected_before_mutation() {
        let mut factory = MockFactory::default();
        let mut authorization = allowed_context();
        authorization.authorization.purpose = "different purpose".into();
        let mut service = CaseService::new(&mut factory);
        assert_eq!(
            service.execute(
                authorization,
                CaseCommand::Create {
                    case: Case::new("case-1".into()).unwrap(),
                },
            ),
            Err(ServiceError::AuthorizationMismatch)
        );
    }

    #[test]
    fn wrong_data_class_is_rejected_before_mutation() {
        let mut factory = MockFactory::default();
        let mut authorization = allowed_context();
        authorization.authorization.data_class = Some("public".into());
        let mut service = CaseService::new(&mut factory);
        assert_eq!(
            service.execute(
                authorization,
                CaseCommand::Create {
                    case: Case::new("case-1".into()).unwrap(),
                },
            ),
            Err(ServiceError::AuthorizationMismatch)
        );
    }

    #[test]
    fn expired_authorization_is_rejected_before_mutation() {
        let mut factory = MockFactory::default();
        let mut authorization = allowed_context();
        authorization.authorization.expires_at_epoch_seconds = Some(109);
        let mut service = CaseService::new(&mut factory);
        assert_eq!(
            service.execute(
                authorization,
                CaseCommand::Create {
                    case: Case::new("case-1".into()).unwrap(),
                },
            ),
            Err(ServiceError::AuthorizationExpired)
        );
    }

    #[test]
    fn future_evaluated_authorization_is_rejected() {
        let mut factory = MockFactory::default();
        let mut authorization = allowed_context();
        authorization.authorization.evaluated_at_epoch_seconds = 111;
        let mut service = CaseService::new(&mut factory);
        assert_eq!(
            service.execute(
                authorization,
                CaseCommand::Create {
                    case: Case::new("case-1".into()).unwrap(),
                },
            ),
            Err(ServiceError::AuthorizationInvalid)
        );
    }

    #[test]
    fn transition_uses_transactional_read_and_cas() {
        let mut factory = MockFactory::default();
        let state = factory.state.clone();
        let mut service = CaseService::new(&mut factory);
        service
            .execute(
                allowed_context(),
                CaseCommand::Create {
                    case: Case::new("case-1".into()).unwrap(),
                },
            )
            .unwrap();
        let mut transition_context = allowed_context();
        transition_context.operation_id = "op-2".into();
        transition_context.correlation_id = "corr-2".into();
        transition_context.authorization = authorization(
            AuthorizationDecision::Allow,
            "user-1",
            CASE_TRANSITION_ACTION,
            "case-1",
            "case management",
            Some("case-restricted"),
            100,
            Some(160),
        );
        let result = service
            .execute(
                transition_context,
                CaseCommand::Transition {
                    case_id: "case-1".into(),
                    expected_revision: Revision::initial(),
                    next: CaseState::Active,
                },
            )
            .unwrap();
        assert_eq!(result.revision.value, 1);
        assert_eq!(
            state
                .0
                .borrow()
                .get(&reference(ResourceType::Case, "case-1"))
                .unwrap()
                .revision
                .value,
            1
        );
    }

    #[test]
    fn stale_transition_is_rejected_without_side_effects() {
        let mut factory = MockFactory::default();
        let state = factory.state.clone();
        let mut service = CaseService::new(&mut factory);
        service
            .execute(
                allowed_context(),
                CaseCommand::Create {
                    case: Case::new("case-1".into()).unwrap(),
                },
            )
            .unwrap();
        let mut transition_context = allowed_context();
        transition_context.operation_id = "op-2".into();
        transition_context.correlation_id = "corr-2".into();
        transition_context.authorization = authorization(
            AuthorizationDecision::Allow,
            "user-1",
            CASE_TRANSITION_ACTION,
            "case-1",
            "case management",
            Some("case-restricted"),
            100,
            Some(160),
        );
        service
            .execute(
                transition_context.clone(),
                CaseCommand::Transition {
                    case_id: "case-1".into(),
                    expected_revision: Revision::initial(),
                    next: CaseState::Active,
                },
            )
            .unwrap();
        let before = state.0.borrow().clone();
        let mut stale = transition_context;
        stale.operation_id = "op-3".into();
        stale.correlation_id = "corr-3".into();
        stale.authorization.request_id = "request-3".into();
        assert_eq!(
            service.execute(
                stale,
                CaseCommand::Transition {
                    case_id: "case-1".into(),
                    expected_revision: Revision::initial(),
                    next: CaseState::Closed,
                },
            ),
            Err(ServiceError::Conflict)
        );
        assert_eq!(*state.0.borrow(), before);
    }

    #[test]
    fn duplicate_operation_is_rejected_by_atomic_idempotency_record() {
        let mut factory = MockFactory::default();
        let mut service = CaseService::new(&mut factory);
        service
            .execute(
                allowed_context(),
                CaseCommand::Create {
                    case: Case::new("case-1".into()).unwrap(),
                },
            )
            .unwrap();
        assert_eq!(
            service.execute(
                allowed_context(),
                CaseCommand::Create {
                    case: Case::new("case-2".into()).unwrap(),
                },
            ),
            Err(ServiceError::Duplicate)
        );
    }

    #[allow(dead_code)]
    fn _value(_: Value) {}
}
