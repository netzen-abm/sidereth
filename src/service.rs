use crate::authorization::{AccessAction, AccessRequest, AuthorizationPolicy};
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandResult {
    pub case_id: Id,
    pub revision: Revision,
    pub event_id: Id,
}

pub struct CaseService<'a, F, P> {
    factory: &'a mut F,
    policy: &'a P,
}

impl<'a, F, P> CaseService<'a, F, P>
where
    F: UnitOfWorkFactory,
    P: AuthorizationPolicy,
{
    pub fn new(factory: &'a mut F, policy: &'a P) -> Self {
        Self { factory, policy }
    }

    pub fn execute(
        &mut self,
        context: CommandContext,
        command: CaseCommand,
    ) -> Result<CommandResult, ServiceError> {
        let (case_id, action) = command_target(&command)?;
        authorize(self.policy, &context.actor_id, &case_id, action)?;

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

    pub fn create_case(
        &mut self,
        actor_id: Id,
        operation_id: Id,
        case: Case,
    ) -> Result<CommandResult, ServiceError> {
        self.execute(
            CommandContext {
                actor_id,
                operation_id: operation_id.clone(),
                correlation_id: operation_id,
            },
            CaseCommand::Create { case },
        )
    }

    pub fn transition_case(
        &mut self,
        actor_id: Id,
        operation_id: Id,
        case_id: Id,
        expected_revision: Revision,
        next: CaseState,
    ) -> Result<CommandResult, ServiceError> {
        self.execute(
            CommandContext {
                actor_id,
                operation_id: operation_id.clone(),
                correlation_id: operation_id,
            },
            CaseCommand::Transition {
                case_id,
                expected_revision,
                next,
            },
        )
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

fn command_target(command: &CaseCommand) -> Result<(Id, AccessAction), ServiceError> {
    match command {
        CaseCommand::Create { case } if case.case_id.is_empty() => Err(ServiceError::InvalidInput),
        CaseCommand::Create { case } => Ok((case.case_id.clone(), AccessAction::Create)),
        CaseCommand::Transition { case_id, .. } if case_id.is_empty() => {
            Err(ServiceError::InvalidInput)
        }
        CaseCommand::Transition { case_id, .. } => Ok((case_id.clone(), AccessAction::Update)),
    }
}

fn authorize<P: AuthorizationPolicy>(
    policy: &P,
    actor_id: &Id,
    case_id: &Id,
    action: AccessAction,
) -> Result<(), ServiceError> {
    policy
        .authorize(&AccessRequest {
            actor_id: actor_id.clone(),
            case_id: case_id.clone(),
            action,
        })
        .map_err(|_| ServiceError::AuthorizationDenied)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::{ResourceRecord, UnitOfWork, UnitOfWorkContext};
    use crate::CaseAccessPolicy;
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

    #[test]
    fn create_writes_case_event_audit_provenance_and_idempotency_atomically() {
        let mut factory = MockFactory::default();
        let policy = CaseAccessPolicy {
            owner_id: "user-1".into(),
        };
        let state = factory.state.clone();
        let mut service = CaseService::new(&mut factory, &policy);
        let result = service
            .create_case(
                "user-1".into(),
                "op-1".into(),
                Case::new("case-1".into()).unwrap(),
            )
            .unwrap();
        assert_eq!(result.revision.value, 0);
        let records = state.0.borrow();
        assert!(records.contains_key(&ResourceRef::new(ResourceType::Case, "case-1").unwrap()));
        assert!(
            records.contains_key(&ResourceRef::new(ResourceType::Event, &result.event_id).unwrap())
        );
        assert!(records.contains_key(&ResourceRef::new(ResourceType::Audit, "audit-op-1").unwrap()));
        assert!(records
            .contains_key(&ResourceRef::new(ResourceType::Provenance, "provenance-op-1").unwrap()));
        assert!(records.contains_key(&ResourceRef::new(ResourceType::Idempotency, "op-1").unwrap()));
    }

    #[test]
    fn unauthorized_command_does_not_claim_idempotency() {
        let mut factory = MockFactory::default();
        let policy = CaseAccessPolicy {
            owner_id: "user-1".into(),
        };
        let mut service = CaseService::new(&mut factory, &policy);
        assert_eq!(
            service.create_case(
                "user-2".into(),
                "op-1".into(),
                Case::new("case-1".into()).unwrap()
            ),
            Err(ServiceError::AuthorizationDenied)
        );
        assert!(factory.state.0.borrow().is_empty());
    }

    #[test]
    fn transition_uses_transactional_read_and_cas() {
        let mut factory = MockFactory::default();
        let policy = CaseAccessPolicy {
            owner_id: "user-1".into(),
        };
        let mut service = CaseService::new(&mut factory, &policy);
        service
            .create_case(
                "user-1".into(),
                "op-1".into(),
                Case::new("case-1".into()).unwrap(),
            )
            .unwrap();
        let result = service
            .transition_case(
                "user-1".into(),
                "op-2".into(),
                "case-1".into(),
                Revision::initial(),
                CaseState::Active,
            )
            .unwrap();
        assert_eq!(result.revision.value, 1);
        let state = factory.state.0.borrow();
        assert_eq!(
            state
                .get(&ResourceRef::new(ResourceType::Case, "case-1").unwrap())
                .unwrap()
                .revision
                .value,
            1
        );
    }

    #[test]
    fn stale_transition_is_rejected_without_side_effects() {
        let mut factory = MockFactory::default();
        let policy = CaseAccessPolicy {
            owner_id: "user-1".into(),
        };
        let state = factory.state.clone();
        let mut service = CaseService::new(&mut factory, &policy);
        service
            .create_case(
                "user-1".into(),
                "op-1".into(),
                Case::new("case-1".into()).unwrap(),
            )
            .unwrap();
        service
            .transition_case(
                "user-1".into(),
                "op-2".into(),
                "case-1".into(),
                Revision::initial(),
                CaseState::Active,
            )
            .unwrap();
        let before = state.0.borrow().clone();
        assert_eq!(
            service.transition_case(
                "user-1".into(),
                "op-3".into(),
                "case-1".into(),
                Revision::initial(),
                CaseState::Closed
            ),
            Err(ServiceError::Conflict)
        );
        assert_eq!(*state.0.borrow(), before);
    }

    #[test]
    fn duplicate_operation_is_rejected_by_atomic_idempotency_record() {
        let mut factory = MockFactory::default();
        let policy = CaseAccessPolicy {
            owner_id: "user-1".into(),
        };
        let mut service = CaseService::new(&mut factory, &policy);
        service
            .create_case(
                "user-1".into(),
                "op-1".into(),
                Case::new("case-1".into()).unwrap(),
            )
            .unwrap();
        assert_eq!(
            service.create_case(
                "user-1".into(),
                "op-1".into(),
                Case::new("case-2".into()).unwrap()
            ),
            Err(ServiceError::Duplicate)
        );
    }

    #[allow(dead_code)]
    fn _value(_: Value) {}
}
