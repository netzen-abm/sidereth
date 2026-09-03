use crate::audit::{AuditRecord, AuditSink};
use crate::authorization::{AccessAction, AccessRequest, AuthorizationPolicy};
use crate::persistence::{CaseStore, IdempotencyClaim, IdempotencyStore, Persisted, PersistenceError};
use crate::{Case, CaseState, Id};

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

pub struct CaseService<'a, S, P, A> {
    store: &'a mut S,
    policy: &'a P,
    audit: &'a mut A,
}

impl<'a, S, P, A> CaseService<'a, S, P, A>
where
    S: CaseStore + IdempotencyStore,
    P: AuthorizationPolicy,
    A: AuditSink,
{
    pub fn new(store: &'a mut S, policy: &'a P, audit: &'a mut A) -> Self {
        Self { store, policy, audit }
    }

    pub fn create_case(
        &mut self,
        actor_id: Id,
        operation_id: Id,
        case: Case,
    ) -> Result<(), ServiceError> {
        authorize(&*self.policy, &actor_id, &case.case_id, AccessAction::Create)?;
        match self.store.claim(operation_id).map_err(ServiceError::from)? {
            IdempotencyClaim::AlreadyClaimed => return Err(ServiceError::Duplicate),
            IdempotencyClaim::Claimed => {}
        }
        self.store
            .create_case(Persisted::new(1, case.clone()).map_err(ServiceError::from)?)
            .map_err(ServiceError::from)?;
        self.audit
            .record(AuditRecord {
                audit_id: format!("audit-{}", case.case_id),
                actor_id,
                action: "case.create".into(),
                aggregate_type: "case".into(),
                aggregate_id: case.case_id,
                occurred_at: "service".into(),
            })
            .map_err(|_| ServiceError::AuditFailure)
    }

    pub fn transition_case(
        &mut self,
        actor_id: Id,
        operation_id: Id,
        case_id: Id,
        expected_revision: crate::Revision,
        next: CaseState,
    ) -> Result<crate::Revision, ServiceError> {
        authorize(&*self.policy, &actor_id, &case_id, AccessAction::Update)?;
        match self.store.claim(operation_id).map_err(ServiceError::from)? {
            IdempotencyClaim::AlreadyClaimed => return Err(ServiceError::Duplicate),
            IdempotencyClaim::Claimed => {}
        }
        let mut case = self
            .store
            .get_case(&case_id)
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound)?
            .value;
        case.transition(next).map_err(|_| ServiceError::InvalidInput)?;
        let revision = self
            .store
            .update_case(&case_id, expected_revision, case)
            .map_err(ServiceError::from)?;
        self.audit
            .record(AuditRecord {
                audit_id: format!("audit-{}-{}", case_id, revision.value),
                actor_id,
                action: "case.transition".into(),
                aggregate_type: "case".into(),
                aggregate_id: case_id,
                occurred_at: "service".into(),
            })
            .map_err(|_| ServiceError::AuditFailure)?;
        Ok(revision)
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
    use crate::{CaseStore, CaseAccessPolicy, InMemoryAudit, LocalFileStore, Revision};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn root() -> std::path::PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("sidereth-service-{stamp}"))
    }

    #[test]
    fn service_authorizes_before_mutation_and_audits_success() {
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
        assert_eq!(audit.records().len(), 1);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn unauthorized_mutation_is_rejected() {
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
        assert_eq!(audit.records().len(), 0);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn transition_uses_expected_revision() {
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
        let revision = service
            .transition_case(
                "user-1".into(),
                "operation-2".into(),
                "case-1".into(),
                Revision::initial(),
                CaseState::Active,
            )
            .unwrap();
        assert_eq!(revision.value, 1);
        let _ = std::fs::remove_dir_all(root);
    }
}
