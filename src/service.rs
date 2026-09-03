use crate::audit::{AuditRecord, AuditSink};
use crate::authorization::{AccessAction, AccessRequest, AuthorizationPolicy};
use crate::event::EventEnvelope;
use crate::persistence::{
    CaseStore, EventStore, IdempotencyClaim, IdempotencyStore, Persisted, PersistenceError,
};
use crate::{Case, CaseState, Id};
use serde_json::json;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceError {
    AuthorizationDenied,
    Conflict,
    Duplicate,
    InvalidInput,
    NotFound,
    Persistence(PersistenceError),
    AuditFailure,
}

impl From<PersistenceError> for ServiceError {
    fn from(error: PersistenceError) -> Self {
        Self::Persistence(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaseCommand {
    Create { case: Case },
    Transition {
        case_id: Id,
        expected_revision: crate::Revision,
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
    pub revision: crate::Revision,
    pub event_id: Id,
}

pub struct CaseService<'a, S, P, A> {
    store: &'a mut S,
    policy: &'a P,
    audit: &'a mut A,
}

impl<'a, S, P, A> CaseService<'a, S, P, A>
where
    S: CaseStore + EventStore + IdempotencyStore,
    P: AuthorizationPolicy,
    A: AuditSink,
{
    pub fn new(store: &'a mut S, policy: &'a P, audit: &'a mut A) -> Self {
        Self { store, policy, audit }
    }

    pub fn execute(
        &mut self,
        context: CommandContext,
        command: CaseCommand,
    ) -> Result<CommandResult, ServiceError> {
        let (case_id, action) = command_target(&command)?;
        authorize(&*self.policy, &context.actor_id, &case_id, action)?;
        claim_operation(&mut *self.store, &context.operation_id)?;

        let (case_id, revision, event_type, payload) = match command {
            CaseCommand::Create { case } => {
                let case_id = case.case_id.clone();
                self.store
                    .create_case(Persisted::new(1, case.clone())?)
                    .map_err(ServiceError::from)?;
                (case_id, crate::Revision::initial(), "case.created", json!({
                    "state": case.state,
                }))
            }
            CaseCommand::Transition {
                case_id,
                expected_revision,
                next,
            } => {
                let mut case = self
                    .store
                    .get_case(&case_id)
                    .map_err(ServiceError::from)?
                    .ok_or(ServiceError::NotFound)?
                    .value;
                case.transition(next).map_err(|_| ServiceError::InvalidInput)?;
                let revision = self
                    .store
                    .update_case(&case_id, expected_revision, case.clone())
                    .map_err(ServiceError::from)?;
                (case_id, revision, "case.transition", json!({
                    "state": case.state,
                }))
            }
        };

        let event_id = format!("event-{}-{}", context.operation_id, revision.value);
        self.store
            .append_event(Persisted::new(
                1,
                EventEnvelope {
                    event_id: event_id.clone(),
                    event_type: event_type.into(),
                    aggregate_type: "case".into(),
                    aggregate_id: case_id.clone(),
                    occurred_at: "service".into(),
                    actor_type: "user".into(),
                    actor_id: context.actor_id.clone(),
                    schema_version: 1,
                    payload,
                    source_refs: vec![],
                    correlation_id: context.correlation_id,
                    causation_id: None,
                },
            )?)
            .map_err(ServiceError::from)?;

        self.audit
            .record(AuditRecord {
                audit_id: format!("audit-{}", context.operation_id),
                actor_id: context.actor_id,
                action: event_type.into(),
                aggregate_type: "case".into(),
                aggregate_id: case_id.clone(),
                occurred_at: "service".into(),
            })
            .map_err(|_| ServiceError::AuditFailure)?;

        Ok(CommandResult {
            case_id,
            revision,
            event_id,
        })
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
        expected_revision: crate::Revision,
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

fn command_target(command: &CaseCommand) -> Result<(Id, AccessAction), ServiceError> {
    match command {
        CaseCommand::Create { case } if case.case_id.is_empty() => {
            Err(ServiceError::InvalidInput)
        }
        CaseCommand::Create { case } => Ok((case.case_id.clone(), AccessAction::Create)),
        CaseCommand::Transition { case_id, .. } if case_id.is_empty() => {
            Err(ServiceError::InvalidInput)
        }
        CaseCommand::Transition { case_id, .. } => Ok((case_id.clone(), AccessAction::Update)),
    }
}

fn claim_operation<S: IdempotencyStore>(
    store: &mut S,
    operation_id: &Id,
) -> Result<(), ServiceError> {
    match store
        .claim(operation_id.clone())
        .map_err(ServiceError::from)?
    {
        IdempotencyClaim::AlreadyClaimed => Err(ServiceError::Duplicate),
        IdempotencyClaim::Claimed => Ok(()),
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
    use crate::{CaseAccessPolicy, InMemoryAudit, LocalFileStore};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn root() -> std::path::PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("sidereth-service-{stamp}"))
    }

    #[test]
    fn command_order_produces_event_and_audit() {
        let root = root();
        let mut store = LocalFileStore::open(&root).unwrap();
        let policy = CaseAccessPolicy { owner_id: "user-1".into() };
        let mut audit = InMemoryAudit::default();
        let mut service = CaseService::new(&mut store, &policy, &mut audit);
        let result = service
            .create_case(
                "user-1".into(),
                "operation-1".into(),
                Case::new("case-1".into()).unwrap(),
            )
            .unwrap();
        assert_eq!(result.revision.value, 0);
        assert!(store.get_event(&result.event_id).unwrap().is_some());
        assert_eq!(audit.records().len(), 1);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn authorization_precedes_idempotency_and_mutation() {
        let root = root();
        let mut store = LocalFileStore::open(&root).unwrap();
        let policy = CaseAccessPolicy { owner_id: "user-1".into() };
        let mut audit = InMemoryAudit::default();
        let mut service = CaseService::new(&mut store, &policy, &mut audit);
        assert_eq!(
            service.create_case(
                "user-2".into(),
                "operation-1".into(),
                Case::new("case-1".into()).unwrap(),
            ),
            Err(ServiceError::AuthorizationDenied)
        );
        assert!(!store.lookup(&"operation-1".into()).unwrap());
        assert!(store.get_case(&"case-1".into()).unwrap().is_none());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn duplicate_operation_is_rejected_before_mutation() {
        let root = root();
        let mut store = LocalFileStore::open(&root).unwrap();
        let policy = CaseAccessPolicy { owner_id: "user-1".into() };
        let mut audit = InMemoryAudit::default();
        let mut service = CaseService::new(&mut store, &policy, &mut audit);
        let case = Case::new("case-1".into()).unwrap();
        service
            .create_case("user-1".into(), "operation-1".into(), case.clone())
            .unwrap();
        assert_eq!(
            service.create_case("user-1".into(), "operation-1".into(), case),
            Err(ServiceError::Duplicate)
        );
        assert_eq!(audit.records().len(), 1);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn transition_honors_expected_revision() {
        let root = root();
        let mut store = LocalFileStore::open(&root).unwrap();
        let policy = CaseAccessPolicy { owner_id: "user-1".into() };
        let mut audit = InMemoryAudit::default();
        let mut service = CaseService::new(&mut store, &policy, &mut audit);
        service
            .create_case(
                "user-1".into(),
                "operation-1".into(),
                Case::new("case-1".into()).unwrap(),
            )
            .unwrap();
        let result = service
            .transition_case(
                "user-1".into(),
                "operation-2".into(),
                "case-1".into(),
                crate::Revision::initial(),
                CaseState::Active,
            )
            .unwrap();
        assert_eq!(result.revision.value, 1);
        let _ = std::fs::remove_dir_all(root);
    }
}
