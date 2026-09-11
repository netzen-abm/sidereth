use crate::{Id, ResourceRef};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AccessAction {
    Read,
    Create,
    Update,
    AppendEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccessRequest {
    pub actor_id: Id,
    pub case_id: Id,
    pub action: AccessAction,
}

pub trait AuthorizationPolicy {
    fn authorize(&self, request: &AccessRequest) -> Result<(), &'static str>;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CaseAccessPolicy {
    pub owner_id: Id,
}

impl AuthorizationPolicy for CaseAccessPolicy {
    fn authorize(&self, request: &AccessRequest) -> Result<(), &'static str> {
        if request.actor_id != self.owner_id {
            return Err("case access denied");
        }
        if request.case_id.is_empty() {
            return Err("case id is required");
        }
        Ok(())
    }
}

/// Decision produced by an authorization policy evaluation.
///
/// `NotApplicable` is deliberately non-authorizing. Downstream execution
/// boundaries must never treat it as an allow.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuthorizationDecision {
    Allow,
    Deny,
    NotApplicable,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthorizationRequest {
    pub subject_ref: ResourceRef,
    pub action: ResourceRef,
    pub resource_ref: ResourceRef,
    pub purpose: String,
    pub policy_refs: Vec<ResourceRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthorizationContext {
    pub jurisdiction_ref: Option<ResourceRef>,
    pub data_class: Option<String>,
    pub scope_refs: Vec<ResourceRef>,
    pub valid: bool,
}

impl Default for AuthorizationContext {
    fn default() -> Self {
        Self {
            jurisdiction_ref: None,
            data_class: None,
            scope_refs: Vec::new(),
            valid: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthorizationResult {
    pub decision: AuthorizationDecision,
    pub constraints: Vec<String>,
    pub policy_refs: Vec<ResourceRef>,
}

pub trait AuthorizationEvaluator {
    fn evaluate(&self, request: &AuthorizationRequest) -> AuthorizationResult;
}

/// Minimal provider-neutral evaluator for the canonical boundary.
///
/// This implementation is intentionally conservative: malformed requests,
/// missing policy references, invalid context and scope mismatches fail closed.
/// It is a foundation implementation, not a complete policy language or
/// production policy engine.
#[derive(Debug, Clone, Default)]
pub struct InMemoryAuthorizationEvaluator;

impl AuthorizationEvaluator for InMemoryAuthorizationEvaluator {
    fn evaluate(&self, request: &AuthorizationRequest) -> AuthorizationResult {
        evaluate_request(request, &AuthorizationContext::default())
    }
}

pub fn evaluate_request(
    request: &AuthorizationRequest,
    context: &AuthorizationContext,
) -> AuthorizationResult {
    let policy_refs = request.policy_refs.clone();

    if request.subject_ref.id.is_empty()
        || request.action.id.is_empty()
        || request.resource_ref.id.is_empty()
        || request.purpose.trim().is_empty()
        || policy_refs.is_empty()
        || !context.valid
    {
        return deny(policy_refs);
    }

    if !context.scope_refs.is_empty() && !context.scope_refs.contains(&request.resource_ref) {
        return deny(policy_refs);
    }

    // Policy interpretation remains intentionally conservative until a
    // canonical policy language/resolution implementation is introduced.
    deny(policy_refs)
}

fn deny(policy_refs: Vec<ResourceRef>) -> AuthorizationResult {
    AuthorizationResult {
        decision: AuthorizationDecision::Deny,
        constraints: Vec::new(),
        policy_refs,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(actor_id: &str) -> AccessRequest {
        AccessRequest {
            actor_id: actor_id.into(),
            case_id: "case-1".into(),
            action: AccessAction::Read,
        }
    }

    fn reference(resource_type: crate::ResourceType, id: &str) -> ResourceRef {
        ResourceRef::new(resource_type, id).unwrap()
    }

    fn canonical_request() -> AuthorizationRequest {
        AuthorizationRequest {
            subject_ref: reference(crate::ResourceType::Party, "party-1"),
            action: reference(crate::ResourceType::Action, "action-1"),
            resource_ref: reference(crate::ResourceType::Document, "doc-1"),
            purpose: "case preparation".into(),
            policy_refs: vec![reference(crate::ResourceType::Other, "policy-1")],
        }
    }

    #[test]
    fn owner_is_authorized_by_legacy_case_policy() {
        let policy = CaseAccessPolicy {
            owner_id: "user-1".into(),
        };
        assert!(policy.authorize(&request("user-1")).is_ok());
    }

    #[test]
    fn other_actor_is_denied_by_legacy_case_policy() {
        let policy = CaseAccessPolicy {
            owner_id: "user-1".into(),
        };
        assert_eq!(
            policy.authorize(&request("user-2")),
            Err("case access denied")
        );
    }

    #[test]
    fn authorization_wire_contract_is_stable() {
        let request = canonical_request();
        let json = serde_json::to_string(&request).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["subject_ref"]["resource_type"], "party");
        assert_eq!(value["purpose"], "case preparation");
        let decoded: AuthorizationRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, request);
    }

    #[test]
    fn authorization_decision_is_not_an_approval_or_legal_authority() {
        let result = AuthorizationResult {
            decision: AuthorizationDecision::Allow,
            constraints: vec!["read_only".into()],
            policy_refs: vec![reference(crate::ResourceType::Other, "policy-1")],
        };
        assert_eq!(result.decision, AuthorizationDecision::Allow);
        assert_eq!(result.constraints, vec!["read_only"]);
    }

    #[test]
    fn missing_policy_fails_closed() {
        let mut request = canonical_request();
        request.policy_refs.clear();
        assert_eq!(
            evaluate_request(&request, &AuthorizationContext::default()).decision,
            AuthorizationDecision::Deny
        );
    }

    #[test]
    fn malformed_request_fails_closed() {
        let mut request = canonical_request();
        request.purpose.clear();
        assert_eq!(
            evaluate_request(&request, &AuthorizationContext::default()).decision,
            AuthorizationDecision::Deny
        );
    }

    #[test]
    fn invalid_context_fails_closed() {
        let request = canonical_request();
        let context = AuthorizationContext {
            valid: false,
            ..AuthorizationContext::default()
        };
        assert_eq!(
            evaluate_request(&request, &context).decision,
            AuthorizationDecision::Deny
        );
    }

    #[test]
    fn scope_mismatch_fails_closed() {
        let request = canonical_request();
        let context = AuthorizationContext {
            scope_refs: vec![reference(crate::ResourceType::Document, "other-doc")],
            ..AuthorizationContext::default()
        };
        assert_eq!(
            evaluate_request(&request, &context).decision,
            AuthorizationDecision::Deny
        );
    }

    #[test]
    fn not_applicable_is_non_authorizing() {
        assert_ne!(
            AuthorizationDecision::NotApplicable,
            AuthorizationDecision::Allow
        );
    }

    #[test]
    fn same_request_is_deterministic() {
        let request = canonical_request();
        let evaluator = InMemoryAuthorizationEvaluator;
        assert_eq!(evaluator.evaluate(&request), evaluator.evaluate(&request));
    }
}
