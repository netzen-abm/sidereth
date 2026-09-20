//! Canonical, provider-neutral data-access boundary for Tool Gateway executions.
//!
//! This module does not grant authority. Authorization, capability leases and the
//! Execution Gate remain authoritative. It converts an already-authorized
//! invocation into a bounded data-access grant so a provider cannot silently
//! widen resource, purpose, scope or classification at the Tool Gateway boundary.

use crate::{tool_registry::ToolDataClass, Id, ResourceRef};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolDataAccessRequest {
    pub resource_ref: ResourceRef,
    pub purpose: String,
    pub data_class: ToolDataClass,
    pub scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolDataAccessGrant {
    pub invocation_id: Id,
    pub authorization_ref: ResourceRef,
    pub resource_ref: ResourceRef,
    pub purpose: String,
    pub data_class: ToolDataClass,
    pub scope: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolDataAccessError {
    ResourceMismatch,
    PurposeMismatch,
    DataClassExceedsAuthorization,
    ScopeMismatch,
    InvalidRequest,
}

impl ToolDataAccessGrant {
    pub fn from_invocation(
        invocation_id: Id,
        authorization_ref: ResourceRef,
        resource_ref: ResourceRef,
        purpose: String,
        data_class: ToolDataClass,
        scope: String,
    ) -> Result<Self, ToolDataAccessError> {
        if invocation_id.is_empty()
            || authorization_ref.id.is_empty()
            || resource_ref.id.is_empty()
            || purpose.trim().is_empty()
            || scope.trim().is_empty()
        {
            return Err(ToolDataAccessError::InvalidRequest);
        }

        Ok(Self {
            invocation_id,
            authorization_ref,
            resource_ref,
            purpose,
            data_class,
            scope,
        })
    }

    /// Authorize an exact data request against this invocation-bound grant.
    ///
    /// Classification is ordered from Public to Restricted. A provider may
    /// request the granted class or a less-sensitive class, never a broader one.
    /// Scope is intentionally exact in v0.1; broader selectors require a
    /// separately contracted capability rather than ad-hoc interpretation.
    pub fn authorize(
        &self,
        request: &ToolDataAccessRequest,
    ) -> Result<(), ToolDataAccessError> {
        if request.resource_ref != self.resource_ref {
            return Err(ToolDataAccessError::ResourceMismatch);
        }
        if request.purpose != self.purpose {
            return Err(ToolDataAccessError::PurposeMismatch);
        }
        if request.scope != self.scope {
            return Err(ToolDataAccessError::ScopeMismatch);
        }
        if data_class_rank(request.data_class) > data_class_rank(self.data_class) {
            return Err(ToolDataAccessError::DataClassExceedsAuthorization);
        }
        Ok(())
    }
}

fn data_class_rank(value: ToolDataClass) -> u8 {
    match value {
        ToolDataClass::Public => 0,
        ToolDataClass::Internal => 1,
        ToolDataClass::Confidential => 2,
        ToolDataClass::Restricted => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ResourceType;

    fn reference(resource_type: ResourceType, id: &str) -> ResourceRef {
        ResourceRef::new(resource_type, id).unwrap()
    }

    fn grant() -> ToolDataAccessGrant {
        ToolDataAccessGrant::from_invocation(
            "invocation-1".into(),
            reference(ResourceType::Other, "auth-1"),
            reference(ResourceType::Case, "case-1"),
            "case preparation".into(),
            ToolDataClass::Confidential,
            "case-1".into(),
        )
        .unwrap()
    }

    fn request(data_class: ToolDataClass) -> ToolDataAccessRequest {
        ToolDataAccessRequest {
            resource_ref: reference(ResourceType::Case, "case-1"),
            purpose: "case preparation".into(),
            data_class,
            scope: "case-1".into(),
        }
    }

    #[test]
    fn exact_authorized_request_passes() {
        assert!(grant().authorize(&request(ToolDataClass::Confidential)).is_ok());
    }

    #[test]
    fn less_sensitive_request_is_allowed() {
        assert!(grant().authorize(&request(ToolDataClass::Public)).is_ok());
    }

    #[test]
    fn broader_classification_is_rejected() {
        assert_eq!(
            grant().authorize(&request(ToolDataClass::Restricted)),
            Err(ToolDataAccessError::DataClassExceedsAuthorization)
        );
    }

    #[test]
    fn resource_cannot_be_widened() {
        let mut value = request(ToolDataClass::Confidential);
        value.resource_ref = reference(ResourceType::Case, "case-2");
        assert_eq!(
            grant().authorize(&value),
            Err(ToolDataAccessError::ResourceMismatch)
        );
    }

    #[test]
    fn purpose_cannot_be_changed() {
        let mut value = request(ToolDataClass::Confidential);
        value.purpose = "different-purpose".into();
        assert_eq!(
            grant().authorize(&value),
            Err(ToolDataAccessError::PurposeMismatch)
        );
    }

    #[test]
    fn scope_cannot_be_widened() {
        let mut value = request(ToolDataClass::Confidential);
        value.scope = "all-cases".into();
        assert_eq!(
            grant().authorize(&value),
            Err(ToolDataAccessError::ScopeMismatch)
        );
    }

    #[test]
    fn empty_grant_context_is_rejected() {
        assert_eq!(
            ToolDataAccessGrant::from_invocation(
                String::new(),
                reference(ResourceType::Other, "auth-1"),
                reference(ResourceType::Case, "case-1"),
                "purpose".into(),
                ToolDataClass::Public,
                "case-1".into(),
            ),
            Err(ToolDataAccessError::InvalidRequest)
        );
    }
}
