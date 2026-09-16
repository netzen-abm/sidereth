use crate::ResourceRef;
use serde::{Deserialize, Serialize};

/// Canonical lifecycle state for a purpose-bound capability lease.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityLeaseState {
    Requested,
    Authorized,
    Active,
    Completed,
    Released,
    Revoked,
    Cancelled,
    Expired,
}

impl CapabilityLeaseState {
    pub fn can_activate(&self) -> bool {
        matches!(self, Self::Authorized)
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Released | Self::Revoked | Self::Cancelled | Self::Expired
        )
    }

    pub fn can_transition_to(&self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Requested, Self::Authorized)
                | (Self::Authorized, Self::Active)
                | (Self::Authorized, Self::Expired)
                | (Self::Authorized, Self::Revoked)
                | (Self::Active, Self::Completed)
                | (Self::Active, Self::Released)
                | (Self::Active, Self::Revoked)
                | (Self::Active, Self::Cancelled)
                | (Self::Active, Self::Expired)
                | (Self::Completed, Self::Released)
        )
    }
}

/// Provider-neutral identity and lifecycle record for temporary capability use.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityLease {
    pub lease_id: ResourceRef,
    pub schema_version: String,
    pub capability_ref: ResourceRef,
    pub resource_ref: Option<ResourceRef>,
    pub subject_ref: ResourceRef,
    pub actor_ref: Option<ResourceRef>,
    pub purpose: String,
    pub purpose_version: Option<String>,
    pub authorization_ref: ResourceRef,
    pub incident_ref: Option<ResourceRef>,
    pub session_ref: Option<ResourceRef>,
    pub scope: String,
    pub issued_at_epoch_seconds: u64,
    pub expires_at_epoch_seconds: u64,
    pub state: CapabilityLeaseState,
    pub revoked_at_epoch_seconds: Option<u64>,
    pub cancelled_at_epoch_seconds: Option<u64>,
    pub activated_at_epoch_seconds: Option<u64>,
    pub released_at_epoch_seconds: Option<u64>,
    pub adapter_ref: Option<ResourceRef>,
    pub audit_ref: Option<ResourceRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityLeaseError {
    InvalidLease,
    Expired,
    NotActivatable,
    InvalidTransition,
    PurposeMismatch,
    ScopeMismatch,
    SubjectMismatch,
    ActorMismatch,
    SessionMismatch,
    IncidentMismatch,
    CapabilityMismatch,
    ResourceMismatch,
    PurposeVersionMismatch,
}

impl CapabilityLease {
    pub fn validate(&self) -> Result<(), CapabilityLeaseError> {
        if self.lease_id.id.is_empty()
            || self.schema_version.is_empty()
            || self.capability_ref.id.is_empty()
            || self.subject_ref.id.is_empty()
            || self.authorization_ref.id.is_empty()
            || self.purpose.is_empty()
            || self.scope.is_empty()
            || self.expires_at_epoch_seconds <= self.issued_at_epoch_seconds
        {
            return Err(CapabilityLeaseError::InvalidLease);
        }

        if self.state == CapabilityLeaseState::Active && self.activated_at_epoch_seconds.is_none() {
            return Err(CapabilityLeaseError::InvalidLease);
        }

        if matches!(
            self.state,
            CapabilityLeaseState::Released
                | CapabilityLeaseState::Revoked
                | CapabilityLeaseState::Cancelled
                | CapabilityLeaseState::Expired
        ) && self.released_at_epoch_seconds.is_none()
            && !matches!(self.state, CapabilityLeaseState::Revoked | CapabilityLeaseState::Expired)
        {
            return Err(CapabilityLeaseError::InvalidLease);
        }

        Ok(())
    }

    pub fn is_expired_at(&self, now_epoch_seconds: u64) -> bool {
        now_epoch_seconds >= self.expires_at_epoch_seconds
    }

    /// Validate the exact context needed before activating or using a lease.
    pub fn validate_use(
        &self,
        now_epoch_seconds: u64,
        capability_ref: &ResourceRef,
        resource_ref: Option<&ResourceRef>,
        subject_ref: &ResourceRef,
        actor_ref: Option<&ResourceRef>,
        purpose: &str,
        purpose_version: Option<&str>,
        scope: &str,
        incident_ref: Option<&ResourceRef>,
        session_ref: Option<&ResourceRef>,
    ) -> Result<(), CapabilityLeaseError> {
        self.validate()?;

        if self.is_expired_at(now_epoch_seconds) || self.state != CapabilityLeaseState::Active {
            return Err(if self.is_expired_at(now_epoch_seconds) {
                CapabilityLeaseError::Expired
            } else {
                CapabilityLeaseError::NotActivatable
            });
        }
        self.validate_context(
            capability_ref,
            resource_ref,
            subject_ref,
            actor_ref,
            purpose,
            purpose_version,
            scope,
            incident_ref,
            session_ref,
        )
    }

    pub fn validate_activation(
        &self,
        now_epoch_seconds: u64,
        capability_ref: &ResourceRef,
        resource_ref: Option<&ResourceRef>,
        subject_ref: &ResourceRef,
        actor_ref: Option<&ResourceRef>,
        purpose: &str,
        purpose_version: Option<&str>,
        scope: &str,
        incident_ref: Option<&ResourceRef>,
        session_ref: Option<&ResourceRef>,
    ) -> Result<(), CapabilityLeaseError> {
        self.validate()?;
        if self.is_expired_at(now_epoch_seconds) {
            return Err(CapabilityLeaseError::Expired);
        }
        if !self.state.can_activate() {
            return Err(CapabilityLeaseError::NotActivatable);
        }
        self.validate_context(
            capability_ref,
            resource_ref,
            subject_ref,
            actor_ref,
            purpose,
            purpose_version,
            scope,
            incident_ref,
            session_ref,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn validate_context(
        &self,
        capability_ref: &ResourceRef,
        resource_ref: Option<&ResourceRef>,
        subject_ref: &ResourceRef,
        actor_ref: Option<&ResourceRef>,
        purpose: &str,
        purpose_version: Option<&str>,
        scope: &str,
        incident_ref: Option<&ResourceRef>,
        session_ref: Option<&ResourceRef>,
    ) -> Result<(), CapabilityLeaseError> {
        if &self.capability_ref != capability_ref {
            return Err(CapabilityLeaseError::CapabilityMismatch);
        }
        if self.resource_ref.as_ref() != resource_ref {
            return Err(CapabilityLeaseError::ResourceMismatch);
        }
        if &self.subject_ref != subject_ref {
            return Err(CapabilityLeaseError::SubjectMismatch);
        }
        if self.actor_ref.as_ref() != actor_ref {
            return Err(CapabilityLeaseError::ActorMismatch);
        }
        if self.purpose != purpose {
            return Err(CapabilityLeaseError::PurposeMismatch);
        }
        if self.purpose_version.as_deref() != purpose_version {
            return Err(CapabilityLeaseError::PurposeVersionMismatch);
        }
        if self.scope != scope {
            return Err(CapabilityLeaseError::ScopeMismatch);
        }
        if self.incident_ref.as_ref() != incident_ref {
            return Err(CapabilityLeaseError::IncidentMismatch);
        }
        if self.session_ref.as_ref() != session_ref {
            return Err(CapabilityLeaseError::SessionMismatch);
        }
        Ok(())
    }

    pub fn transition(
        &mut self,
        next: CapabilityLeaseState,
        at_epoch_seconds: u64,
    ) -> Result<(), CapabilityLeaseError> {
        if !self.state.can_transition_to(next) {
            return Err(CapabilityLeaseError::InvalidTransition);
        }

        if next != CapabilityLeaseState::Expired && at_epoch_seconds >= self.expires_at_epoch_seconds {
            return Err(CapabilityLeaseError::Expired);
        }

        self.state = next;
        match next {
            CapabilityLeaseState::Active => self.activated_at_epoch_seconds = Some(at_epoch_seconds),
            CapabilityLeaseState::Revoked => self.revoked_at_epoch_seconds = Some(at_epoch_seconds),
            CapabilityLeaseState::Cancelled => self.cancelled_at_epoch_seconds = Some(at_epoch_seconds),
            CapabilityLeaseState::Released => self.released_at_epoch_seconds = Some(at_epoch_seconds),
            CapabilityLeaseState::Expired => self.released_at_epoch_seconds = None,
            _ => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ResourceType;

    fn reference(resource_type: ResourceType, id: &str) -> ResourceRef {
        ResourceRef::new(resource_type, id).unwrap()
    }

    fn lease() -> CapabilityLease {
        CapabilityLease {
            lease_id: reference(ResourceType::Other, "lease-1"),
            schema_version: "0.1".into(),
            capability_ref: reference(ResourceType::Other, "microphone"),
            resource_ref: Some(reference(ResourceType::Other, "device-mic-1")),
            subject_ref: reference(ResourceType::Party, "party-1"),
            actor_ref: Some(reference(ResourceType::Other, "surface-1")),
            purpose: "safety incident evidence capture".into(),
            purpose_version: Some("1".into()),
            authorization_ref: reference(ResourceType::Other, "auth-1"),
            incident_ref: Some(reference(ResourceType::Incident, "incident-1")),
            session_ref: Some(reference(ResourceType::Other, "session-1")),
            scope: "incident-1:audio".into(),
            issued_at_epoch_seconds: 100,
            expires_at_epoch_seconds: 200,
            state: CapabilityLeaseState::Authorized,
            revoked_at_epoch_seconds: None,
            cancelled_at_epoch_seconds: None,
            activated_at_epoch_seconds: None,
            released_at_epoch_seconds: None,
            adapter_ref: None,
            audit_ref: None,
        }
    }

    #[test]
    fn valid_lease_activates_only_in_exact_context() {
        let mut lease = lease();
        let capability = lease.capability_ref.clone();
        let resource = lease.resource_ref.clone();
        let subject = lease.subject_ref.clone();
        let actor = lease.actor_ref.clone();
        let incident = lease.incident_ref.clone();
        let session = lease.session_ref.clone();

        assert!(lease.validate_activation(
            150,
            &capability,
            resource.as_ref(),
            &subject,
            actor.as_ref(),
            "safety incident evidence capture",
            Some("1"),
            "incident-1:audio",
            incident.as_ref(),
            session.as_ref(),
        ).is_ok());

        lease.transition(CapabilityLeaseState::Active, 150).unwrap();
        assert!(lease.validate_use(
            150,
            &capability,
            resource.as_ref(),
            &subject,
            actor.as_ref(),
            "safety incident evidence capture",
            Some("1"),
            "incident-1:audio",
            incident.as_ref(),
            session.as_ref(),
        ).is_ok());
    }

    #[test]
    fn exact_expiry_is_fail_closed() {
        let lease = lease();
        assert_eq!(lease.is_expired_at(200), true);
        assert_eq!(
            lease.validate_activation(
                200,
                &lease.capability_ref,
                lease.resource_ref.as_ref(),
                &lease.subject_ref,
                lease.actor_ref.as_ref(),
                &lease.purpose,
                lease.purpose_version.as_deref(),
                &lease.scope,
                lease.incident_ref.as_ref(),
                lease.session_ref.as_ref(),
            ),
            Err(CapabilityLeaseError::Expired)
        );
    }

    #[test]
    fn purpose_scope_and_subject_are_exact() {
        let lease = lease();
        let mut bad_purpose = lease.purpose.clone();
        bad_purpose.push_str("-different");
        assert_eq!(
            lease.validate_activation(
                150,
                &lease.capability_ref,
                lease.resource_ref.as_ref(),
                &lease.subject_ref,
                lease.actor_ref.as_ref(),
                &bad_purpose,
                lease.purpose_version.as_deref(),
                &lease.scope,
                lease.incident_ref.as_ref(),
                lease.session_ref.as_ref(),
            ),
            Err(CapabilityLeaseError::PurposeMismatch)
        );
        assert_eq!(
            lease.validate_activation(
                150,
                &lease.capability_ref,
                lease.resource_ref.as_ref(),
                &lease.subject_ref,
                lease.actor_ref.as_ref(),
                &lease.purpose,
                lease.purpose_version.as_deref(),
                "incident-1:audio-wide",
                lease.incident_ref.as_ref(),
                lease.session_ref.as_ref(),
            ),
            Err(CapabilityLeaseError::ScopeMismatch)
        );
        let other_subject = reference(ResourceType::Party, "party-2");
        assert_eq!(
            lease.validate_activation(
                150,
                &lease.capability_ref,
                lease.resource_ref.as_ref(),
                &other_subject,
                lease.actor_ref.as_ref(),
                &lease.purpose,
                lease.purpose_version.as_deref(),
                &lease.scope,
                lease.incident_ref.as_ref(),
                lease.session_ref.as_ref(),
            ),
            Err(CapabilityLeaseError::SubjectMismatch)
        );
    }

    #[test]
    fn terminal_lease_cannot_reactivate() {
        let mut lease = lease();
        lease.transition(CapabilityLeaseState::Active, 150).unwrap();
        lease.transition(CapabilityLeaseState::Completed, 160).unwrap();
        lease.transition(CapabilityLeaseState::Released, 161).unwrap();
        assert!(lease.state.is_terminal());
        assert_eq!(
            lease.transition(CapabilityLeaseState::Active, 170),
            Err(CapabilityLeaseError::InvalidTransition)
        );
    }

    #[test]
    fn revocation_and_cancellation_are_terminal_for_use() {
        let mut revoked = lease();
        revoked.transition(CapabilityLeaseState::Active, 150).unwrap();
        revoked.transition(CapabilityLeaseState::Revoked, 155).unwrap();
        assert_eq!(
            revoked.validate_use(
                156,
                &revoked.capability_ref,
                revoked.resource_ref.as_ref(),
                &revoked.subject_ref,
                revoked.actor_ref.as_ref(),
                &revoked.purpose,
                revoked.purpose_version.as_deref(),
                &revoked.scope,
                revoked.incident_ref.as_ref(),
                revoked.session_ref.as_ref(),
            ),
            Err(CapabilityLeaseError::NotActivatable)
        );

        let mut cancelled = lease();
        cancelled.transition(CapabilityLeaseState::Active, 150).unwrap();
        cancelled.transition(CapabilityLeaseState::Cancelled, 155).unwrap();
        assert!(cancelled.state.is_terminal());
    }

    #[test]
    fn release_is_recorded_and_old_lease_is_not_reusable() {
        let mut lease = lease();
        lease.transition(CapabilityLeaseState::Active, 150).unwrap();
        lease.transition(CapabilityLeaseState::Completed, 160).unwrap();
        lease.transition(CapabilityLeaseState::Released, 161).unwrap();
        assert_eq!(lease.released_at_epoch_seconds, Some(161));
        assert_eq!(
            lease.validate_use(
                162,
                &lease.capability_ref,
                lease.resource_ref.as_ref(),
                &lease.subject_ref,
                lease.actor_ref.as_ref(),
                &lease.purpose,
                lease.purpose_version.as_deref(),
                &lease.scope,
                lease.incident_ref.as_ref(),
                lease.session_ref.as_ref(),
            ),
            Err(CapabilityLeaseError::NotActivatable)
        );
    }
}
