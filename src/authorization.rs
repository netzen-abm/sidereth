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

/// Canonical authorization decision. This is not human approval, legal
/// authority, evidence authenticity, or execution permission by itself.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuthorizationDecision {
    Allow,
    Deny,
    NotApplicable,
}

/// Canonical request presented to the authorization boundary.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthorizationRequest {
    pub request_id: Id,
    pub authorization_ref: ResourceRef,
    pub subject_ref: ResourceRef,
    pub action: ResourceRef,
    pub resource_ref: ResourceRef,
    pub purpose: String,
    pub policy_refs: Vec<ResourceRef>,
    pub jurisdiction_ref: Option<ResourceRef>,
    pub data_class: Option<String>,
    pub requested_at_epoch_seconds: u64,
    pub freshness_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthorizationConstraint {
    pub key: String,
    pub value: String,
}

/// Canonical result bound to the exact request context that was evaluated.
/// The context fields are carried forward so a consuming boundary can
/// compare invocation scope with the evaluated authorization without
/// substituting a different subject, action, resource, purpose, jurisdiction,
/// or data class. This is structural binding, not cryptographic attestation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthorizationResult {
    pub request_id: Id,
    pub authorization_ref: ResourceRef,
    pub subject_ref: ResourceRef,
    pub action: ResourceRef,
    pub resource_ref: ResourceRef,
    pub purpose: String,
    pub jurisdiction_ref: Option<ResourceRef>,
    pub data_class: Option<String>,
    pub decision: AuthorizationDecision,
    pub constraints: Vec<AuthorizationConstraint>,
    pub policy_refs: Vec<ResourceRef>,
    pub evaluated_at_epoch_seconds: u64,
    pub expires_at_epoch_seconds: Option<u64>,
}

impl AuthorizationResult {
    fn from_request(
        request: &AuthorizationRequest,
        decision: AuthorizationDecision,
        constraints: Vec<AuthorizationConstraint>,
        policy_refs: Vec<ResourceRef>,
        evaluated_at_epoch_seconds: u64,
        expires_at_epoch_seconds: Option<u64>,
    ) -> Self {
        Self {
            request_id: request.request_id.clone(),
            authorization_ref: request.authorization_ref.clone(),
            subject_ref: request.subject_ref.clone(),
            action: request.action.clone(),
            resource_ref: request.resource_ref.clone(),
            purpose: request.purpose.clone(),
            jurisdiction_ref: request.jurisdiction_ref.clone(),
            data_class: request.data_class.clone(),
            decision,
            constraints,
            policy_refs,
            evaluated_at_epoch_seconds,
            expires_at_epoch_seconds,
        }
    }
}

pub trait AuthorizationEvaluator {
    fn evaluate(&self, request: &AuthorizationRequest) -> AuthorizationResult;
}

/// A deliberately small, provider-neutral rule representation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthorizationRule {
    pub policy_ref: ResourceRef,
    pub subject_ref: ResourceRef,
    pub action: ResourceRef,
    pub resource_ref: ResourceRef,
    pub purpose: String,
    pub jurisdiction_ref: Option<ResourceRef>,
    pub data_class: Option<String>,
    pub decision: AuthorizationDecision,
    pub constraints: Vec<AuthorizationConstraint>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StaticAuthorizationEvaluator {
    pub rules: Vec<AuthorizationRule>,
    pub now_epoch_seconds: u64,
}

impl StaticAuthorizationEvaluator {
    fn fail_closed(&self, request: &AuthorizationRequest) -> AuthorizationResult {
        AuthorizationResult::from_request(
            request,
            AuthorizationDecision::Deny,
            Vec::new(),
            request.policy_refs.clone(),
            self.now_epoch_seconds,
            None,
        )
    }

    fn valid_request(&self, request: &AuthorizationRequest) -> bool {
        !request.request_id.is_empty()
            && !request.authorization_ref.id.is_empty()
            && !request.purpose.is_empty()
            && !request.subject_ref.id.is_empty()
            && !request.action.id.is_empty()
            && !request.resource_ref.id.is_empty()
            && request
                .freshness_seconds
                .map(|freshness| {
                    self.now_epoch_seconds >= request.requested_at_epoch_seconds
                        && self.now_epoch_seconds - request.requested_at_epoch_seconds <= freshness
                })
                .unwrap_or(true)
    }

    fn matches(rule: &AuthorizationRule, request: &AuthorizationRequest) -> bool {
        rule.subject_ref == request.subject_ref
            && rule.action == request.action
            && rule.resource_ref == request.resource_ref
            && rule.purpose == request.purpose
            && rule.jurisdiction_ref == request.jurisdiction_ref
            && rule.data_class == request.data_class
            && (request.policy_refs.is_empty() || request.policy_refs.contains(&rule.policy_ref))
    }
}

impl AuthorizationEvaluator for StaticAuthorizationEvaluator {
    fn evaluate(&self, request: &AuthorizationRequest) -> AuthorizationResult {
        if !self.valid_request(request) {
            return self.fail_closed(request);
        }

        let matches: Vec<&AuthorizationRule> = self
            .rules
            .iter()
            .filter(|rule| Self::matches(rule, request))
            .collect();

        if matches.is_empty() {
            return AuthorizationResult::from_request(
                request,
                AuthorizationDecision::NotApplicable,
                Vec::new(),
                request.policy_refs.clone(),
                self.now_epoch_seconds,
                None,
            );
        }

        let first_decision = matches[0].decision;
        if matches.iter().any(|rule| rule.decision != first_decision) {
            return self.fail_closed(request);
        }

        let mut constraints = Vec::new();
        let mut policy_refs = Vec::new();
        for rule in matches {
            if !policy_refs.contains(&rule.policy_ref) {
                policy_refs.push(rule.policy_ref.clone());
            }
            constraints.extend(rule.constraints.clone());
        }

        AuthorizationResult::from_request(
            request,
            first_decision,
            constraints,
            policy_refs,
            self.now_epoch_seconds,
            request
                .freshness_seconds
                .map(|freshness| self.now_epoch_seconds.saturating_add(freshness)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ResourceType;

    fn reference(resource_type: ResourceType, id: &str) -> ResourceRef {
        ResourceRef::new(resource_type, id).unwrap()
    }

    fn canonical_request() -> AuthorizationRequest {
        AuthorizationRequest {
            request_id: "req-1".into(),
            authorization_ref: reference(ResourceType::Other, "auth-1"),
            subject_ref: reference(ResourceType::Party, "party-1"),
            action: reference(ResourceType::Action, "read-document"),
            resource_ref: reference(ResourceType::Document, "doc-1"),
            purpose: "case preparation".into(),
            policy_refs: vec![reference(ResourceType::Other, "policy-1")],
            jurisdiction_ref: Some(reference(ResourceType::Jurisdiction, "jurisdiction-1")),
            data_class: Some("case-restricted".into()),
            requested_at_epoch_seconds: 100,
            freshness_seconds: Some(60),
        }
    }

    fn rule(decision: AuthorizationDecision) -> AuthorizationRule {
        let request = canonical_request();
        AuthorizationRule {
            policy_ref: reference(ResourceType::Other, "policy-1"),
            subject_ref: request.subject_ref,
            action: request.action,
            resource_ref: request.resource_ref,
            purpose: request.purpose,
            jurisdiction_ref: request.jurisdiction_ref,
            data_class: request.data_class,
            decision,
            constraints: vec![AuthorizationConstraint {
                key: "access_mode".into(),
                value: "read_only".into(),
            }],
        }
    }

    #[test]
    fn owner_is_authorized() {
        let policy = CaseAccessPolicy {
            owner_id: "user-1".into(),
        };
        let request = AccessRequest {
            actor_id: "user-1".into(),
            case_id: "case-1".into(),
            action: AccessAction::Read,
        };
        assert!(policy.authorize(&request).is_ok());
    }

    #[test]
    fn other_actor_is_denied() {
        let policy = CaseAccessPolicy {
            owner_id: "user-1".into(),
        };
        let request = AccessRequest {
            actor_id: "user-2".into(),
            case_id: "case-1".into(),
            action: AccessAction::Read,
        };
        assert_eq!(policy.authorize(&request), Err("case access denied"));
    }

    #[test]
    fn authorization_wire_contract_is_stable() {
        let request = canonical_request();
        let json = serde_json::to_string(&request).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["authorization_ref"]["id"], "auth-1");
        assert_eq!(value["subject_ref"]["resource_type"], "party");
        assert_eq!(value["purpose"], "case preparation");
        assert_eq!(value["data_class"], "case-restricted");
        let decoded: AuthorizationRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, request);
    }

    #[test]
    fn matching_rule_allows_with_constraints() {
        let evaluator = StaticAuthorizationEvaluator {
            rules: vec![rule(AuthorizationDecision::Allow)],
            now_epoch_seconds: 110,
        };
        let result = evaluator.evaluate(&canonical_request());
        assert_eq!(result.authorization_ref.id, "auth-1");
        assert_eq!(result.decision, AuthorizationDecision::Allow);
        assert_eq!(result.constraints.len(), 1);
        assert_eq!(result.policy_refs.len(), 1);
        assert_eq!(result.expires_at_epoch_seconds, Some(170));
    }

    #[test]
    fn authorization_result_preserves_evaluated_request_context() {
        let request = canonical_request();
        let evaluator = StaticAuthorizationEvaluator {
            rules: vec![rule(AuthorizationDecision::Allow)],
            now_epoch_seconds: 110,
        };

        let result = evaluator.evaluate(&request);

        assert_eq!(result.request_id, request.request_id);
        assert_eq!(result.authorization_ref, request.authorization_ref);
        assert_eq!(result.subject_ref, request.subject_ref);
        assert_eq!(result.action, request.action);
        assert_eq!(result.resource_ref, request.resource_ref);
        assert_eq!(result.purpose, request.purpose);
        assert_eq!(result.jurisdiction_ref, request.jurisdiction_ref);
        assert_eq!(result.data_class, request.data_class);
    }

    #[test]
    fn fail_closed_result_still_preserves_request_context() {
        let mut request = canonical_request();
        request.purpose.clear();
        let evaluator = StaticAuthorizationEvaluator {
            rules: vec![rule(AuthorizationDecision::Allow)],
            now_epoch_seconds: 110,
        };

        let result = evaluator.evaluate(&request);

        assert_eq!(result.decision, AuthorizationDecision::Deny);
        assert_eq!(result.request_id, request.request_id);
        assert_eq!(result.subject_ref, request.subject_ref);
        assert_eq!(result.action, request.action);
        assert_eq!(result.resource_ref, request.resource_ref);
        assert_eq!(result.purpose, request.purpose);
        assert_eq!(result.jurisdiction_ref, request.jurisdiction_ref);
        assert_eq!(result.data_class, request.data_class);
    }

    #[test]
    fn no_applicable_policy_is_not_allow() {
        let evaluator = StaticAuthorizationEvaluator {
            rules: vec![],
            now_epoch_seconds: 110,
        };
        let result = evaluator.evaluate(&canonical_request());
        assert_eq!(result.decision, AuthorizationDecision::NotApplicable);
    }

    #[test]
    fn malformed_request_fails_closed() {
        let evaluator = StaticAuthorizationEvaluator {
            rules: vec![rule(AuthorizationDecision::Allow)],
            now_epoch_seconds: 110,
        };
        let mut request = canonical_request();
        request.purpose.clear();
        assert_eq!(
            evaluator.evaluate(&request).decision,
            AuthorizationDecision::Deny
        );
    }

    #[test]
    fn stale_request_fails_closed() {
        let evaluator = StaticAuthorizationEvaluator {
            rules: vec![rule(AuthorizationDecision::Allow)],
            now_epoch_seconds: 200,
        };
        assert_eq!(
            evaluator.evaluate(&canonical_request()).decision,
            AuthorizationDecision::Deny
        );
    }

    #[test]
    fn conflicting_rules_fail_closed() {
        let evaluator = StaticAuthorizationEvaluator {
            rules: vec![
                rule(AuthorizationDecision::Allow),
                rule(AuthorizationDecision::Deny),
            ],
            now_epoch_seconds: 110,
        };
        assert_eq!(
            evaluator.evaluate(&canonical_request()).decision,
            AuthorizationDecision::Deny
        );
    }

    #[test]
    fn authorization_does_not_execute() {
        let evaluator = StaticAuthorizationEvaluator {
            rules: vec![rule(AuthorizationDecision::Allow)],
            now_epoch_seconds: 110,
        };
        let result = evaluator.evaluate(&canonical_request());
        assert_eq!(result.decision, AuthorizationDecision::Allow);
    }

    #[test]
    fn authorization_decision_is_not_approval_or_legal_authority() {
        let request = canonical_request();
        let result = AuthorizationResult::from_request(
            &request,
            AuthorizationDecision::Allow,
            vec![AuthorizationConstraint {
                key: "access_mode".into(),
                value: "read_only".into(),
            }],
            vec![reference(ResourceType::Other, "policy-1")],
            100,
            Some(160),
        );
        assert_eq!(result.decision, AuthorizationDecision::Allow);
        assert_eq!(result.constraints[0].value, "read_only");
    }
}
