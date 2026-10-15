use crate::authorization::{AuthorizationEvaluator, AuthorizationRequest};
use crate::{
    validate_authorization, AuthorizationResult, EvidenceObjectStore, EvidenceOriginal, Id,
    ResourceRef, ResourceType,
};

/// Provider-neutral request for retrieval of evidence content.
///
/// The request carries intent only. The protected boundary obtains the
/// authorization result from the configured canonical evaluator and obtains
/// the current time locally; callers cannot submit a fabricated Allow result
/// or choose a stale timestamp.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceQueryRequest {
    pub evidence_ref: ResourceRef,
    pub authorization_request: AuthorizationRequest,
}

impl EvidenceQueryRequest {
    pub fn new(evidence_ref: ResourceRef, authorization_request: AuthorizationRequest) -> Self {
        Self {
            evidence_ref,
            authorization_request,
        }
    }

    fn validate_context(&self) -> Result<(), EvidenceQueryError> {
        let request = &self.authorization_request;
        if request.resource_ref != self.evidence_ref
            || request.resource_ref.resource_type != ResourceType::Evidence
            || request.action.resource_type != ResourceType::Action
            || request.action.id != "evidence.read"
        {
            return Err(EvidenceQueryError::AuthorizationMismatch);
        }
        Ok(())
    }

    fn validate_result(
        &self,
        authorization: &AuthorizationResult,
        now_epoch_seconds: u64,
    ) -> Result<(), EvidenceQueryError> {
        validate_authorization(
            &self.authorization_request,
            authorization,
            now_epoch_seconds,
        )
        .map_err(|error| match error {
            crate::AuthorizationValidationError::InvalidRequest => {
                EvidenceQueryError::InvalidRequest
            }
            crate::AuthorizationValidationError::RequestResultMismatch => {
                EvidenceQueryError::AuthorizationMismatch
            }
            crate::AuthorizationValidationError::NotYetEvaluated
            | crate::AuthorizationValidationError::ConstraintViolation => {
                EvidenceQueryError::AuthorizationInvalid
            }
            crate::AuthorizationValidationError::Expired => {
                EvidenceQueryError::AuthorizationExpired
            }
            crate::AuthorizationValidationError::Denied => EvidenceQueryError::AuthorizationDenied,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceQueryResult {
    pub original: EvidenceOriginal,
    pub content: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceQueryError {
    AuthorizationDenied,
    AuthorizationMismatch,
    AuthorizationInvalid,
    AuthorizationExpired,
    InvalidRequest,
    EvidenceNotFound,
    StorageFailure,
}

/// Canonical protected evidence-content retrieval boundary.
///
/// Object storage remains authorization-neutral. Callers must enter through
/// this boundary when retrieving protected evidence content. The boundary
/// evaluates the exact request, validates the result against current time,
/// then reads metadata/content and verifies the content hash.
pub struct AuthorizedEvidenceQuery<
    'a,
    R: crate::EvidenceRepository,
    O: EvidenceObjectStore,
    E: AuthorizationEvaluator,
> {
    repository: &'a R,
    object_store: &'a O,
    evaluator: &'a E,
}

impl<'a, R: crate::EvidenceRepository, O: EvidenceObjectStore, E: AuthorizationEvaluator>
    AuthorizedEvidenceQuery<'a, R, O, E>
{
    pub fn new(repository: &'a R, object_store: &'a O, evaluator: &'a E) -> Self {
        Self {
            repository,
            object_store,
            evaluator,
        }
    }

    pub fn get(
        &self,
        request: EvidenceQueryRequest,
    ) -> Result<EvidenceQueryResult, EvidenceQueryError> {
        request.validate_context()?;
        let authorization = self.evaluator.evaluate(&request.authorization_request);
        let now_epoch_seconds = crate::authorization_enforcement::current_epoch_seconds()
            .ok_or(EvidenceQueryError::AuthorizationInvalid)?;
        request.validate_result(&authorization, now_epoch_seconds)?;

        let evidence_id: Id = request.evidence_ref.id.clone();
        let original = self
            .repository
            .get_original(&evidence_id)
            .map_err(|_| EvidenceQueryError::StorageFailure)?
            .ok_or(EvidenceQueryError::EvidenceNotFound)?;
        if ResourceRef::new(ResourceType::Evidence, original.evidence_id.clone())
            .map_err(|_| EvidenceQueryError::InvalidRequest)?
            != request.evidence_ref
        {
            return Err(EvidenceQueryError::AuthorizationMismatch);
        }
        let content = self
            .object_store
            .get(&original.storage_ref)
            .map_err(|_| EvidenceQueryError::StorageFailure)?
            .ok_or(EvidenceQueryError::EvidenceNotFound)?;
        if crate::sha256_hex(&content) != original.content_hash {
            return Err(EvidenceQueryError::StorageFailure);
        }
        Ok(EvidenceQueryResult { original, content })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authorization::{AuthorizationConstraint, AuthorizationDecision};
    use crate::{EvidenceRepository, InMemoryEvidenceVault};

    #[derive(Clone)]
    struct TestEvaluator {
        decision: AuthorizationDecision,
        constraints: Vec<AuthorizationConstraint>,
        expires_in_seconds: Option<u64>,
        expired: bool,
        mismatched_purpose: bool,
    }

    impl Default for TestEvaluator {
        fn default() -> Self {
            Self {
                decision: AuthorizationDecision::Allow,
                constraints: vec![AuthorizationConstraint {
                    key: "scope".into(),
                    value: "exact_evidence".into(),
                }],
                expires_in_seconds: Some(3_600),
                expired: false,
                mismatched_purpose: false,
            }
        }
    }

    impl AuthorizationEvaluator for TestEvaluator {
        fn evaluate(&self, request: &AuthorizationRequest) -> AuthorizationResult {
            let now = crate::authorization_enforcement::current_epoch_seconds().expect("system clock must be available");
            let mut result = AuthorizationResult {
                request_id: request.request_id.clone(),
                authorization_ref: request.authorization_ref.clone(),
                subject_ref: request.subject_ref.clone(),
                action: request.action.clone(),
                resource_ref: request.resource_ref.clone(),
                purpose: request.purpose.clone(),
                jurisdiction_ref: request.jurisdiction_ref.clone(),
                data_class: request.data_class.clone(),
                decision: self.decision,
                constraints: self.constraints.clone(),
                policy_refs: request.policy_refs.clone(),
                evaluated_at_epoch_seconds: now,
                expires_at_epoch_seconds: self.expires_in_seconds.map(|seconds| {
                    if self.expired {
                        now.saturating_sub(seconds.max(1))
                    } else {
                        now.saturating_add(seconds)
                    }
                }),
            };
            if self.mismatched_purpose {
                result.purpose = "different purpose".into();
            }
            result
        }
    }

    fn authorization_request(evidence_ref: &ResourceRef) -> AuthorizationRequest {
        let now = current_epoch_seconds().unwrap();
        AuthorizationRequest {
            request_id: "request-1".into(),
            authorization_ref: ResourceRef::new(ResourceType::Other, "auth-1").unwrap(),
            subject_ref: ResourceRef::new(ResourceType::Party, "party-1").unwrap(),
            action: ResourceRef::new(ResourceType::Action, "evidence.read").unwrap(),
            resource_ref: evidence_ref.clone(),
            purpose: "protected evidence retrieval".into(),
            policy_refs: vec![ResourceRef::new(ResourceType::Other, "policy-1").unwrap()],
            jurisdiction_ref: None,
            data_class: Some("restricted".into()),
            requested_at_epoch_seconds: now,
            freshness_seconds: Some(1_000),
        }
    }

    fn evidence() -> EvidenceOriginal {
        EvidenceOriginal::from_capture(crate::EvidenceCapture {
            evidence_id: "evidence-1".into(),
            schema_version: 1,
            case_id: Some("case-1".into()),
            incident_id: None,
            captured_at: "2026-09-03T10:00:00Z".into(),
            captured_by: "user-1".into(),
            media_type: "text/plain".into(),
            storage_ref: "object-1".into(),
            content: b"original evidence",
        })
        .unwrap()
    }

    fn vault() -> InMemoryEvidenceVault {
        let mut vault = InMemoryEvidenceVault::default();
        let original = evidence();
        crate::EvidenceObjectStore::put(
            &mut vault,
            original.storage_ref.clone(),
            b"original evidence".to_vec(),
        )
        .unwrap();
        EvidenceRepository::save_original(&mut vault, original).unwrap();
        vault
    }

    fn request() -> EvidenceQueryRequest {
        let evidence_ref = ResourceRef::new(ResourceType::Evidence, "evidence-1").unwrap();
        EvidenceQueryRequest::new(evidence_ref.clone(), authorization_request(&evidence_ref))
    }

    #[test]
    fn authorized_retrieval_returns_metadata_and_content() {
        let vault = vault();
        let evaluator = TestEvaluator::default();
        let query = AuthorizedEvidenceQuery::new(&vault, &vault, &evaluator);
        let result = query.get(request()).unwrap();
        assert_eq!(result.original.evidence_id, "evidence-1");
        assert_eq!(result.content, b"original evidence");
    }

    #[test]
    fn denied_authorization_is_rejected_before_storage_read() {
        let vault = vault();
        let evaluator = TestEvaluator {
            decision: AuthorizationDecision::Deny,
            ..TestEvaluator::default()
        };
        let query = AuthorizedEvidenceQuery::new(&vault, &vault, &evaluator);
        assert_eq!(
            query.get(request()),
            Err(EvidenceQueryError::AuthorizationDenied)
        );
    }

    #[test]
    fn authorization_must_bind_to_exact_evidence() {
        let vault = vault();
        let mut request = request();
        request.authorization_request.resource_ref =
            ResourceRef::new(ResourceType::Evidence, "evidence-2").unwrap();
        let evaluator = TestEvaluator::default();
        let query = AuthorizedEvidenceQuery::new(&vault, &vault, &evaluator);
        assert_eq!(
            query.get(request),
            Err(EvidenceQueryError::AuthorizationMismatch)
        );
    }

    #[test]
    fn evaluator_result_must_match_request_context() {
        let vault = vault();
        let evaluator = TestEvaluator {
            mismatched_purpose: true,
            ..TestEvaluator::default()
        };
        let query = AuthorizedEvidenceQuery::new(&vault, &vault, &evaluator);
        assert_eq!(
            query.get(request()),
            Err(EvidenceQueryError::AuthorizationMismatch)
        );
    }

    #[test]
    fn authorization_for_a_different_action_cannot_read_evidence() {
        let vault = vault();
        let mut request = request();
        request.authorization_request.action =
            ResourceRef::new(ResourceType::Action, "evidence.delete").unwrap();
        let evaluator = TestEvaluator::default();
        let query = AuthorizedEvidenceQuery::new(&vault, &vault, &evaluator);
        assert_eq!(
            query.get(request),
            Err(EvidenceQueryError::AuthorizationMismatch)
        );
    }

    #[test]
    fn expired_authorization_is_rejected() {
        let vault = vault();
        let evaluator = TestEvaluator {
            expired: true,
            ..TestEvaluator::default()
        };
        let query = AuthorizedEvidenceQuery::new(&vault, &vault, &evaluator);
        assert_eq!(
            query.get(request()),
            Err(EvidenceQueryError::AuthorizationExpired)
        );
    }

    #[test]
    fn stale_request_is_rejected() {
        let vault = vault();
        let mut request = request();
        request.authorization_request.requested_at_epoch_seconds =
            current_epoch_seconds().unwrap().saturating_sub(1_000);
        request.authorization_request.freshness_seconds = Some(100);
        let evaluator = TestEvaluator::default();
        let query = AuthorizedEvidenceQuery::new(&vault, &vault, &evaluator);
        assert_eq!(
            query.get(request),
            Err(EvidenceQueryError::AuthorizationExpired)
        );
    }

    #[test]
    fn unknown_authorization_constraint_fails_closed() {
        let vault = vault();
        let evaluator = TestEvaluator {
            constraints: vec![
                AuthorizationConstraint {
                    key: "scope".into(),
                    value: "exact_evidence".into(),
                },
                AuthorizationConstraint {
                    key: "future_constraint".into(),
                    value: "allow_all".into(),
                },
            ],
            ..TestEvaluator::default()
        };
        let query = AuthorizedEvidenceQuery::new(&vault, &vault, &evaluator);
        assert_eq!(
            query.get(request()),
            Err(EvidenceQueryError::AuthorizationInvalid)
        );
    }

    #[test]
    fn tampered_content_is_rejected() {
        let mut vault = vault();
        vault.tamper_object("object-1", b"tampered evidence");
        let evaluator = TestEvaluator::default();
        let query = AuthorizedEvidenceQuery::new(&vault, &vault, &evaluator);
        assert_eq!(
            query.get(request()),
            Err(EvidenceQueryError::StorageFailure)
        );
    }
}
