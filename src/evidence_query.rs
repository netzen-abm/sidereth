use crate::authorization::AuthorizationRequest;
use crate::{
    AuthorizationDecision, AuthorizationResult, EvidenceObjectStore, EvidenceOriginal, Id,
    ResourceRef, ResourceType,
};

/// Provider-neutral request for retrieval of evidence content.
///
/// The authorization request and result are both retained so the retrieval
/// boundary can prove that the exact context evaluated by the authorization
/// boundary is the context being executed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceQueryRequest {
    pub evidence_ref: ResourceRef,
    pub authorization_request: AuthorizationRequest,
    pub authorization: AuthorizationResult,
    pub now_epoch_seconds: u64,
}

impl EvidenceQueryRequest {
    pub fn new(
        evidence_ref: ResourceRef,
        authorization_request: AuthorizationRequest,
        authorization: AuthorizationResult,
        now_epoch_seconds: u64,
    ) -> Self {
        Self {
            evidence_ref,
            authorization_request,
            authorization,
            now_epoch_seconds,
        }
    }

    fn validate(&self) -> Result<(), EvidenceQueryError> {
        let request = &self.authorization_request;
        let authorization = &self.authorization;
        if request.request_id.is_empty()
            || request.authorization_ref.id.is_empty()
            || request.subject_ref.id.is_empty()
            || request.action.id.is_empty()
            || request.resource_ref.id.is_empty()
            || request.purpose.trim().is_empty()
        {
            return Err(EvidenceQueryError::InvalidRequest);
        }
        if request.resource_ref != self.evidence_ref
            || request.resource_ref.resource_type != ResourceType::Evidence
        {
            return Err(EvidenceQueryError::AuthorizationMismatch);
        }
        if request.requested_at_epoch_seconds > self.now_epoch_seconds
            || request.freshness_seconds.is_some_and(|freshness| {
                self.now_epoch_seconds - request.requested_at_epoch_seconds > freshness
            })
        {
            return Err(EvidenceQueryError::AuthorizationExpired);
        }
        if authorization.request_id != request.request_id
            || authorization.authorization_ref != request.authorization_ref
            || authorization.subject_ref != request.subject_ref
            || authorization.action != request.action
            || authorization.resource_ref != request.resource_ref
            || authorization.purpose != request.purpose
            || authorization.jurisdiction_ref != request.jurisdiction_ref
            || authorization.data_class != request.data_class
            || authorization.policy_refs != request.policy_refs
        {
            return Err(EvidenceQueryError::AuthorizationMismatch);
        }
        if authorization.decision != AuthorizationDecision::Allow {
            return Err(EvidenceQueryError::AuthorizationDenied);
        }
        if authorization.evaluated_at_epoch_seconds > self.now_epoch_seconds {
            return Err(EvidenceQueryError::AuthorizationInvalid);
        }
        if authorization
            .expires_at_epoch_seconds
            .map(|expires_at| self.now_epoch_seconds >= expires_at)
            .unwrap_or(false)
        {
            return Err(EvidenceQueryError::AuthorizationExpired);
        }
        Ok(())
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
/// validates the canonical authorization context before reading either the
/// evidence metadata or its content object.
pub struct AuthorizedEvidenceQuery<'a, R: crate::EvidenceRepository, O: EvidenceObjectStore> {
    repository: &'a R,
    object_store: &'a O,
}

impl<'a, R: crate::EvidenceRepository, O: EvidenceObjectStore> AuthorizedEvidenceQuery<'a, R, O> {
    pub fn new(repository: &'a R, object_store: &'a O) -> Self {
        Self {
            repository,
            object_store,
        }
    }

    pub fn get(
        &self,
        request: EvidenceQueryRequest,
    ) -> Result<EvidenceQueryResult, EvidenceQueryError> {
        request.validate()?;
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
    use crate::authorization::AuthorizationConstraint;
    use crate::{EvidenceRepository, InMemoryEvidenceVault};

    fn authorization_context(
        evidence_ref: &ResourceRef,
    ) -> (AuthorizationRequest, AuthorizationResult) {
        let request = AuthorizationRequest {
            request_id: "request-1".into(),
            authorization_ref: ResourceRef::new(ResourceType::Other, "auth-1").unwrap(),
            subject_ref: ResourceRef::new(ResourceType::Party, "party-1").unwrap(),
            action: ResourceRef::new(ResourceType::Action, "evidence.read").unwrap(),
            resource_ref: evidence_ref.clone(),
            purpose: "protected evidence retrieval".into(),
            policy_refs: vec![ResourceRef::new(ResourceType::Other, "policy-1").unwrap()],
            jurisdiction_ref: None,
            data_class: Some("restricted".into()),
            requested_at_epoch_seconds: 1_000,
            freshness_seconds: Some(1_000),
        };
        let result = AuthorizationResult {
            request_id: request.request_id.clone(),
            authorization_ref: request.authorization_ref.clone(),
            subject_ref: request.subject_ref.clone(),
            action: request.action.clone(),
            resource_ref: request.resource_ref.clone(),
            purpose: request.purpose.clone(),
            jurisdiction_ref: request.jurisdiction_ref.clone(),
            data_class: request.data_class.clone(),
            decision: AuthorizationDecision::Allow,
            constraints: vec![AuthorizationConstraint {
                key: "scope".into(),
                value: "exact_evidence".into(),
            }],
            policy_refs: request.policy_refs.clone(),
            evaluated_at_epoch_seconds: 1_000,
            expires_at_epoch_seconds: Some(2_000),
        };
        (request, result)
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
        let (authorization_request, authorization) = authorization_context(&evidence_ref);
        EvidenceQueryRequest::new(evidence_ref, authorization_request, authorization, 1_500)
    }

    #[test]
    fn authorized_retrieval_returns_metadata_and_content() {
        let vault = vault();
        let query = AuthorizedEvidenceQuery::new(&vault, &vault);
        let result = query.get(request()).unwrap();
        assert_eq!(result.original.evidence_id, "evidence-1");
        assert_eq!(result.content, b"original evidence");
    }

    #[test]
    fn denied_authorization_is_rejected_before_storage_read() {
        let vault = vault();
        let mut request = request();
        request.authorization.decision = AuthorizationDecision::Deny;
        let query = AuthorizedEvidenceQuery::new(&vault, &vault);
        assert_eq!(
            query.get(request),
            Err(EvidenceQueryError::AuthorizationDenied)
        );
    }

    #[test]
    fn authorization_must_bind_to_exact_evidence() {
        let vault = vault();
        let mut request = request();
        request.authorization_request.resource_ref =
            ResourceRef::new(ResourceType::Evidence, "evidence-2").unwrap();
        let query = AuthorizedEvidenceQuery::new(&vault, &vault);
        assert_eq!(
            query.get(request),
            Err(EvidenceQueryError::AuthorizationMismatch)
        );
    }

    #[test]
    fn authorization_result_must_match_request_context() {
        let vault = vault();
        let mut request = request();
        request.authorization.purpose = "different purpose".into();
        let query = AuthorizedEvidenceQuery::new(&vault, &vault);
        assert_eq!(
            query.get(request),
            Err(EvidenceQueryError::AuthorizationMismatch)
        );
    }

    #[test]
    fn expired_authorization_is_rejected() {
        let vault = vault();
        let mut request = request();
        request.authorization.expires_at_epoch_seconds = Some(1_500);
        let query = AuthorizedEvidenceQuery::new(&vault, &vault);
        assert_eq!(
            query.get(request),
            Err(EvidenceQueryError::AuthorizationExpired)
        );
    }

    #[test]
    fn stale_request_is_rejected() {
        let vault = vault();
        let mut request = request();
        request.authorization_request.freshness_seconds = Some(100);
        let query = AuthorizedEvidenceQuery::new(&vault, &vault);
        assert_eq!(
            query.get(request),
            Err(EvidenceQueryError::AuthorizationExpired)
        );
    }

    #[test]
    fn tampered_content_is_rejected() {
        let mut vault = vault();
        vault.tamper_object("object-1", b"tampered evidence");
        let query = AuthorizedEvidenceQuery::new(&vault, &vault);
        assert_eq!(
            query.get(request()),
            Err(EvidenceQueryError::StorageFailure)
        );
    }
}
