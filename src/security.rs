use crate::{AuditRecord, AuditSink, AuthorizationPolicy, Id};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvidenceError {
    NotFound,
    Unauthorized,
    Duplicate,
    IntegrityFailure,
    InvalidInput,
    RetentionBlocked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetentionPolicy {
    pub policy_id: Id,
    pub retain_until: Option<String>,
    pub legal_hold: bool,
}

impl RetentionPolicy {
    pub fn deletion_allowed(&self, now: &str) -> Result<bool, EvidenceError> {
        if self.legal_hold {
            return Ok(false);
        }
        match &self.retain_until {
            Some(until) => Ok(now >= until),
            None => Err(EvidenceError::InvalidInput),
        }
    }
}

pub trait KeyProvider {
    fn key_reference(&self, purpose: &str) -> Result<Id, EvidenceError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceExport {
    pub evidence_id: Id,
    pub case_id: Option<Id>,
    pub incident_id: Option<Id>,
    pub content_hash: String,
    pub storage_ref: Id,
}

pub trait EvidenceExporter {
    fn export(&self, evidence_id: &Id) -> Result<EvidenceExport, EvidenceError>;
}

pub struct AuthorizedAudit<'a, P, A> {
    pub policy: &'a P,
    pub audit: &'a mut A,
}

impl<'a, P, A> AuthorizedAudit<'a, P, A>
where
    P: AuthorizationPolicy,
    A: AuditSink,
{
    pub fn authorize_and_audit(
        &mut self,
        request: &crate::AccessRequest,
        audit: AuditRecord,
    ) -> Result<(), EvidenceError> {
        self.policy
            .authorize(request)
            .map_err(|_| EvidenceError::Unauthorized)?;
        self.audit
            .record(audit)
            .map_err(|_| EvidenceError::InvalidInput)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AccessAction, AccessRequest, CaseAccessPolicy, InMemoryAudit};

    fn request(actor_id: &str) -> AccessRequest {
        AccessRequest {
            actor_id: actor_id.into(),
            case_id: "case-1".into(),
            action: AccessAction::Read,
        }
    }

    fn audit() -> AuditRecord {
        AuditRecord {
            audit_id: "audit-1".into(),
            actor_id: "user-1".into(),
            action: "evidence.read".into(),
            aggregate_type: "evidence".into(),
            aggregate_id: "evidence-1".into(),
            occurred_at: "2026-09-03T10:00:00Z".into(),
        }
    }

    #[test]
    fn legal_hold_blocks_deletion() {
        let policy = RetentionPolicy {
            policy_id: "retain-1".into(),
            retain_until: Some("2026-01-01T00:00:00Z".into()),
            legal_hold: true,
        };
        assert!(!policy.deletion_allowed("2026-09-03T00:00:00Z").unwrap());
    }

    #[test]
    fn expired_retention_allows_deletion() {
        let policy = RetentionPolicy {
            policy_id: "retain-1".into(),
            retain_until: Some("2026-01-01T00:00:00Z".into()),
            legal_hold: false,
        };
        assert!(policy.deletion_allowed("2026-09-03T00:00:00Z").unwrap());
    }

    #[test]
    fn active_retention_blocks_deletion() {
        let policy = RetentionPolicy {
            policy_id: "retain-1".into(),
            retain_until: Some("2026-12-01T00:00:00Z".into()),
            legal_hold: false,
        };
        assert!(!policy.deletion_allowed("2026-09-03T00:00:00Z").unwrap());
    }

    #[test]
    fn missing_retention_deadline_is_invalid() {
        let policy = RetentionPolicy {
            policy_id: "retain-1".into(),
            retain_until: None,
            legal_hold: false,
        };
        assert_eq!(
            policy.deletion_allowed("2026-09-03T00:00:00Z"),
            Err(EvidenceError::InvalidInput)
        );
    }

    #[test]
    fn unauthorized_access_is_rejected_before_audit() {
        let policy = CaseAccessPolicy {
            owner_id: "user-1".into(),
        };
        let mut audit_store = InMemoryAudit::default();
        let mut boundary = AuthorizedAudit {
            policy: &policy,
            audit: &mut audit_store,
        };

        let result = boundary.authorize_and_audit(&request("user-2"), audit());

        assert_eq!(result, Err(EvidenceError::Unauthorized));
        assert!(audit_store.records().is_empty());
    }

    #[test]
    fn authorized_access_is_audited() {
        let policy = CaseAccessPolicy {
            owner_id: "user-1".into(),
        };
        let mut audit_store = InMemoryAudit::default();
        let mut boundary = AuthorizedAudit {
            policy: &policy,
            audit: &mut audit_store,
        };

        boundary
            .authorize_and_audit(&request("user-1"), audit())
            .unwrap();

        assert_eq!(audit_store.records().len(), 1);
    }
}
