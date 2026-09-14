use crate::{AuditRecord, AuditSink, AuthorizationDecision, AuthorizationResult, Id};

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
            Some(until) => Ok(now >= until.as_str()),
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

pub struct AuthorizedAudit<'a, A> {
    pub audit: &'a mut A,
}

impl<'a, A> AuthorizedAudit<'a, A>
where
    A: AuditSink,
{
    /// Consumes an already-evaluated canonical authorization result.
    ///
    /// This boundary does not establish independent authorization semantics.
    /// It only accepts an Allow result and preserves the canonical evaluator as
    /// the sole authorization decision boundary for protected audit actions.
    pub fn authorize_and_audit(
        &mut self,
        authorization: &AuthorizationResult,
        audit: AuditRecord,
    ) -> Result<(), EvidenceError> {
        if authorization.decision != AuthorizationDecision::Allow {
            return Err(EvidenceError::Unauthorized);
        }
        self.audit
            .record(audit)
            .map_err(|_| EvidenceError::InvalidInput)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        authorization::AuthorizationConstraint, InMemoryAudit, ResourceRef, ResourceType,
    };

    fn authorization(decision: AuthorizationDecision) -> AuthorizationResult {
        let subject_ref = ResourceRef::new(ResourceType::Party, "user-1").unwrap();
        let action = ResourceRef::new(ResourceType::Other, "evidence.read").unwrap();
        let resource_ref = ResourceRef::new(ResourceType::Evidence, "evidence-1").unwrap();
        AuthorizationResult {
            request_id: "request-1".into(),
            authorization_ref: ResourceRef::new(ResourceType::Other, "auth-1").unwrap(),
            subject_ref,
            action,
            resource_ref,
            purpose: "audit evidence read".into(),
            jurisdiction_ref: None,
            data_class: Some("public".into()),
            decision,
            constraints: vec![AuthorizationConstraint {
                key: "scope".into(),
                value: "exact_resource".into(),
            }],
            policy_refs: vec![],
            evaluated_at_epoch_seconds: 1_000,
            expires_at_epoch_seconds: Some(2_000),
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
    fn denied_canonical_authorization_is_rejected_before_audit() {
        let mut audit_store = InMemoryAudit::default();
        let mut boundary = AuthorizedAudit {
            audit: &mut audit_store,
        };

        let result = boundary.authorize_and_audit(
            &authorization(AuthorizationDecision::Deny),
            audit(),
        );

        assert_eq!(result, Err(EvidenceError::Unauthorized));
        assert!(audit_store.records().is_empty());
    }

    #[test]
    fn not_applicable_canonical_authorization_is_rejected_before_audit() {
        let mut audit_store = InMemoryAudit::default();
        let mut boundary = AuthorizedAudit {
            audit: &mut audit_store,
        };

        let result = boundary.authorize_and_audit(
            &authorization(AuthorizationDecision::NotApplicable),
            audit(),
        );

        assert_eq!(result, Err(EvidenceError::Unauthorized));
        assert!(audit_store.records().is_empty());
    }

    #[test]
    fn canonical_allow_is_audited() {
        let mut audit_store = InMemoryAudit::default();
        let mut boundary = AuthorizedAudit {
            audit: &mut audit_store,
        };

        boundary
            .authorize_and_audit(&authorization(AuthorizationDecision::Allow), audit())
            .unwrap();

        assert_eq!(audit_store.records().len(), 1);
    }
}
