use crate::authorization::{
    AuthorizationDecision, AuthorizationRequest, AuthorizationResult,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthorizationValidationError {
    InvalidRequest,
    RequestResultMismatch,
    NotYetEvaluated,
    Expired,
    Denied,
    ConstraintViolation,
}

/// Canonical consumer-side enforcement of an already evaluated authorization
/// result. This is not a second policy evaluator.
pub fn validate_authorization(
    request: &AuthorizationRequest,
    authorization: &AuthorizationResult,
    now_epoch_seconds: u64,
) -> Result<(), AuthorizationValidationError> {
    if request.request_id.is_empty()
        || request.authorization_ref.id.is_empty()
        || request.subject_ref.id.is_empty()
        || request.action.id.is_empty()
        || request.resource_ref.id.is_empty()
        || request.purpose.trim().is_empty()
    {
        return Err(AuthorizationValidationError::InvalidRequest);
    }
    if request.requested_at_epoch_seconds > now_epoch_seconds
        || request.freshness_seconds.is_some_and(|freshness| {
            now_epoch_seconds - request.requested_at_epoch_seconds > freshness
        })
    {
        return Err(AuthorizationValidationError::Expired);
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
        return Err(AuthorizationValidationError::RequestResultMismatch);
    }
    if authorization.evaluated_at_epoch_seconds > now_epoch_seconds {
        return Err(AuthorizationValidationError::NotYetEvaluated);
    }
    if authorization
        .expires_at_epoch_seconds
        .is_some_and(|expires_at| now_epoch_seconds >= expires_at)
    {
        return Err(AuthorizationValidationError::Expired);
    }
    if authorization.decision != AuthorizationDecision::Allow {
        return Err(AuthorizationValidationError::Denied);
    }
    validate_constraints(&authorization.constraints)
}

/// The currently defined constraint vocabulary is deliberately closed. New
/// constraint kinds must be implemented here before a protected consumer can
/// accept them; silently ignoring an unknown constraint would widen authority.
fn validate_constraints(
    constraints: &[crate::authorization::AuthorizationConstraint],
) -> Result<(), AuthorizationValidationError> {
    for constraint in constraints {
        match constraint.key.as_str() {
            "scope" if constraint.value == "exact_resource" || constraint.value == "exact_evidence" => {}
            "access_mode" if constraint.value == "read_only" => {}
            _ => return Err(AuthorizationValidationError::ConstraintViolation),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authorization::AuthorizationConstraint;
    use crate::{ResourceRef, ResourceType, StaticAuthorizationEvaluator};

    fn reference(kind: ResourceType, id: &str) -> ResourceRef {
        ResourceRef::new(kind, id).unwrap()
    }

    fn request() -> AuthorizationRequest {
        AuthorizationRequest {
            request_id: "req-1".into(),
            authorization_ref: reference(ResourceType::Other, "auth-1"),
            subject_ref: reference(ResourceType::Party, "party-1"),
            action: reference(ResourceType::Action, "resource.read"),
            resource_ref: reference(ResourceType::Case, "case-1"),
            purpose: "protected read".into(),
            policy_refs: vec![reference(ResourceType::Other, "policy-1")],
            jurisdiction_ref: None,
            data_class: Some("restricted".into()),
            requested_at_epoch_seconds: 1_000,
            freshness_seconds: Some(100),
        }
    }

    fn allowed() -> AuthorizationResult {
        let request = request();
        AuthorizationResult {
            request_id: request.request_id.clone(),
            authorization_ref: request.authorization_ref.clone(),
            subject_ref: request.subject_ref.clone(),
            action: request.action.clone(),
            resource_ref: request.resource_ref.clone(),
            purpose: request.purpose.clone(),
            jurisdiction_ref: request.jurisdiction_ref.clone(),
            data_class: request.data_class.clone(),
            decision: AuthorizationDecision::Allow,
            constraints: vec![AuthorizationConstraint { key: "scope".into(), value: "exact_resource".into() }],
            policy_refs: request.policy_refs.clone(),
            evaluated_at_epoch_seconds: 1_000,
            expires_at_epoch_seconds: Some(1_100),
        }
    }

    #[test]
    fn valid_allow_is_accepted() {
        assert_eq!(validate_authorization(&request(), &allowed(), 1_050), Ok(()));
    }

    #[test]
    fn exact_expiry_is_rejected() {
        assert_eq!(validate_authorization(&request(), &allowed(), 1_100), Err(AuthorizationValidationError::Expired));
    }

    #[test]
    fn unknown_constraint_fails_closed() {
        let mut result = allowed();
        result.constraints.push(AuthorizationConstraint { key: "future_constraint".into(), value: "x".into() });
        assert_eq!(validate_authorization(&request(), &result, 1_050), Err(AuthorizationValidationError::ConstraintViolation));
    }

    #[test]
    fn deny_is_rejected() {
        let mut result = allowed();
        result.decision = AuthorizationDecision::Deny;
        assert_eq!(validate_authorization(&request(), &result, 1_050), Err(AuthorizationValidationError::Denied));
    }

    #[test]
    fn malformed_request_fails_closed() {
        let mut request = request();
        request.purpose.clear();
        assert_eq!(validate_authorization(&request, &allowed(), 1_050), Err(AuthorizationValidationError::InvalidRequest));
    }

    #[test]
    fn static_evaluator_result_is_consumable() {
        let request = request();
        let evaluator = StaticAuthorizationEvaluator { rules: vec![], now_epoch_seconds: 1_050 };
        let result = evaluator.evaluate(&request);
        assert_eq!(result.decision, AuthorizationDecision::NotApplicable);
        assert_eq!(validate_authorization(&request, &result, 1_050), Err(AuthorizationValidationError::Denied));
    }
}
