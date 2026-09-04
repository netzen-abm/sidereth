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
/// This is deliberately separate from human approval, legal authority,
/// policy definition, and execution status. A decision records the result of
/// evaluating a request; it does not itself grant legal authority.
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
pub struct AuthorizationResult {
    pub decision: AuthorizationDecision,
    pub constraints: Vec<String>,
    pub policy_refs: Vec<ResourceRef>,
}

pub trait AuthorizationEvaluator {
    fn evaluate(&self, request: &AuthorizationRequest) -> AuthorizationResult;
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

    #[test]
    fn owner_is_authorized() {
        let policy = CaseAccessPolicy {
            owner_id: "user-1".into(),
        };
        assert!(policy.authorize(&request("user-1")).is_ok());
    }

    #[test]
    fn other_actor_is_denied() {
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
        let request = AuthorizationRequest {
            subject_ref: reference(crate::ResourceType::Party, "party-1"),
            action: reference(crate::ResourceType::Action, "action-1"),
            resource_ref: reference(crate::ResourceType::Document, "doc-1"),
            purpose: "case preparation".into(),
            policy_refs: vec![reference(crate::ResourceType::Other, "policy-1")],
        };
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
}
