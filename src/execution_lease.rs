use crate::{CapabilityLease, CapabilityLeaseError, CapabilityLeaseState, ResourceRef};

/// Provider-neutral runtime boundary for activating and terminating a canonical lease.
///
/// This type owns runtime orchestration only. It does not evaluate authorization,
/// widen lease scope, select providers, or persist audit records.
pub struct ExecutionLeaseRuntime<A: ExecutionLeaseAdapter> {
    lease: CapabilityLease,
    adapter: A,
    active_handle: Option<A::Handle>,
}

/// Replaceable provider/OS boundary for a leased capability.
pub trait ExecutionLeaseAdapter {
    type Handle;
    type Error;

    fn activate(&mut self, lease: &CapabilityLease) -> Result<Self::Handle, Self::Error>;
    fn release(&mut self, handle: &mut Self::Handle) -> Result<(), Self::Error>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum ExecutionLeaseRuntimeError<E> {
    Lease(CapabilityLeaseError),
    Adapter(E),
    AlreadyActive,
    NotActive,
    CannotReauthorize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionLeaseRuntimeOutcome {
    Activated,
    Released,
    Revoked,
    Cancelled,
    Expired,
    ReleasedWithPlatformLimitation,
}

/// Exact authorization context required to activate a lease.
#[derive(Debug, Clone)]
pub struct ExecutionLeaseActivationContext<'a> {
    pub now_epoch_seconds: u64,
    pub capability_ref: &'a ResourceRef,
    pub resource_ref: Option<&'a ResourceRef>,
    pub subject_ref: &'a ResourceRef,
    pub actor_ref: Option<&'a ResourceRef>,
    pub purpose: &'a str,
    pub purpose_version: Option<&'a str>,
    pub scope: &'a str,
    pub incident_ref: Option<&'a ResourceRef>,
    pub session_ref: Option<&'a ResourceRef>,
}

impl<A: ExecutionLeaseAdapter> ExecutionLeaseRuntime<A> {
    pub fn new(lease: CapabilityLease, adapter: A) -> Result<Self, CapabilityLeaseError> {
        lease.validate()?;
        Ok(Self {
            lease,
            adapter,
            active_handle: None,
        })
    }

    pub fn lease(&self) -> &CapabilityLease {
        &self.lease
    }

    pub fn lease_mut(&mut self) -> &mut CapabilityLease {
        &mut self.lease
    }

    pub fn activate(
        &mut self,
        context: ExecutionLeaseActivationContext<'_>,
    ) -> Result<ExecutionLeaseRuntimeOutcome, ExecutionLeaseRuntimeError<A::Error>> {
        if self.active_handle.is_some() || self.lease.state == CapabilityLeaseState::Active {
            return Err(ExecutionLeaseRuntimeError::AlreadyActive);
        }

        self.lease
            .validate_activation(
                context.now_epoch_seconds,
                context.capability_ref,
                context.resource_ref,
                context.subject_ref,
                context.actor_ref,
                context.purpose,
                context.purpose_version,
                context.scope,
                context.incident_ref,
                context.session_ref,
            )
            .map_err(ExecutionLeaseRuntimeError::Lease)?;

        let handle = self
            .adapter
            .activate(&self.lease)
            .map_err(ExecutionLeaseRuntimeError::Adapter)?;

        self.lease
            .transition(CapabilityLeaseState::Active, context.now_epoch_seconds)
            .map_err(ExecutionLeaseRuntimeError::Lease)?;
        self.active_handle = Some(handle);
        Ok(ExecutionLeaseRuntimeOutcome::Activated)
    }

    pub fn release(
        &mut self,
        now_epoch_seconds: u64,
    ) -> Result<ExecutionLeaseRuntimeOutcome, ExecutionLeaseRuntimeError<A::Error>> {
        let handle = self
            .active_handle
            .as_mut()
            .ok_or(ExecutionLeaseRuntimeError::NotActive)?;

        self.adapter
            .release(handle)
            .map_err(ExecutionLeaseRuntimeError::Adapter)?;

        self.active_handle = None;
        if self.lease.state == CapabilityLeaseState::Active {
            self.lease
                .transition(CapabilityLeaseState::Completed, now_epoch_seconds)
                .map_err(ExecutionLeaseRuntimeError::Lease)?;
            self.lease
                .transition(CapabilityLeaseState::Released, now_epoch_seconds)
                .map_err(ExecutionLeaseRuntimeError::Lease)?;
        }
        Ok(ExecutionLeaseRuntimeOutcome::Released)
    }

    pub fn revoke(
        &mut self,
        now_epoch_seconds: u64,
    ) -> Result<ExecutionLeaseRuntimeOutcome, ExecutionLeaseRuntimeError<A::Error>> {
        if self.lease.state == CapabilityLeaseState::Authorized
            || self.lease.state == CapabilityLeaseState::Active
        {
            self.lease
                .transition(CapabilityLeaseState::Revoked, now_epoch_seconds)
                .map_err(ExecutionLeaseRuntimeError::Lease)?;
        }
        self.terminate_active_resource()?;
        Ok(ExecutionLeaseRuntimeOutcome::Revoked)
    }

    pub fn cancel(
        &mut self,
        now_epoch_seconds: u64,
    ) -> Result<ExecutionLeaseRuntimeOutcome, ExecutionLeaseRuntimeError<A::Error>> {
        if self.lease.state == CapabilityLeaseState::Active {
            self.lease
                .transition(CapabilityLeaseState::Cancelled, now_epoch_seconds)
                .map_err(ExecutionLeaseRuntimeError::Lease)?;
        }
        self.terminate_active_resource()?;
        Ok(ExecutionLeaseRuntimeOutcome::Cancelled)
    }

    pub fn expire(
        &mut self,
        now_epoch_seconds: u64,
    ) -> Result<ExecutionLeaseRuntimeOutcome, ExecutionLeaseRuntimeError<A::Error>> {
        if !self.lease.is_expired_at(now_epoch_seconds) {
            return Err(ExecutionLeaseRuntimeError::Lease(
                CapabilityLeaseError::InvalidTransition,
            ));
        }

        if matches!(
            self.lease.state,
            CapabilityLeaseState::Authorized | CapabilityLeaseState::Active
        ) {
            self.lease
                .transition(CapabilityLeaseState::Expired, now_epoch_seconds)
                .map_err(ExecutionLeaseRuntimeError::Lease)?;
        }
        self.terminate_active_resource()?;
        Ok(ExecutionLeaseRuntimeOutcome::Expired)
    }

    /// Replace a terminal lease with a freshly authorized lease.
    /// Authorization itself remains outside this runtime boundary.
    pub fn reauthorize(
        &mut self,
        new_lease: CapabilityLease,
    ) -> Result<(), ExecutionLeaseRuntimeError<A::Error>> {
        if self.active_handle.is_some() || !self.lease.state.is_terminal() {
            return Err(ExecutionLeaseRuntimeError::CannotReauthorize);
        }
        new_lease
            .validate()
            .map_err(ExecutionLeaseRuntimeError::Lease)?;
        if new_lease.state != CapabilityLeaseState::Authorized {
            return Err(ExecutionLeaseRuntimeError::Lease(
                CapabilityLeaseError::NotActivatable,
            ));
        }
        self.lease = new_lease;
        Ok(())
    }

    fn terminate_active_resource(&mut self) -> Result<(), ExecutionLeaseRuntimeError<A::Error>> {
        if let Some(handle) = self.active_handle.as_mut() {
            self.adapter
                .release(handle)
                .map_err(ExecutionLeaseRuntimeError::Adapter)?;
        }
        self.active_handle = None;
        Ok(())
    }
}

/// Provider-neutral lifecycle fact emitted after a runtime state change or
/// provider operation outcome. It is evidence, not an instruction to mutate
/// runtime state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionLeaseLifecycleEvent {
    pub event_id: crate::Id,
    pub lease_ref: ResourceRef,
    pub authorization_ref: ResourceRef,
    pub capability_ref: ResourceRef,
    pub resource_ref: Option<ResourceRef>,
    pub subject_ref: ResourceRef,
    pub actor_ref: Option<ResourceRef>,
    pub operation: String,
    pub from_state: Option<CapabilityLeaseState>,
    pub to_state: CapabilityLeaseState,
    pub occurred_at: String,
    pub correlation_id: crate::Id,
    pub causation_id: Option<crate::Id>,
    pub provider_id: Option<crate::Id>,
    pub purpose: String,
    pub scope: String,
    pub outcome: String,
    pub failure: Option<String>,
}

impl<A: ExecutionLeaseAdapter> ExecutionLeaseRuntime<A> {
    /// Build canonical lifecycle evidence from the runtime's current lease truth.
    ///
    /// This constructs evidence only; it does not persist it or mutate runtime state.
    pub fn lifecycle_event(
        &self,
        context: ExecutionLeaseEvidenceContext<'_>,
    ) -> Result<ExecutionLeaseLifecycleEvent, &'static str> {
        let event = ExecutionLeaseLifecycleEvent {
            event_id: context.event_id.to_owned(),
            lease_ref: self.lease.lease_id.clone(),
            authorization_ref: self.lease.authorization_ref.clone(),
            capability_ref: self.lease.capability_ref.clone(),
            resource_ref: self.lease.resource_ref.clone(),
            subject_ref: self.lease.subject_ref.clone(),
            actor_ref: self.lease.actor_ref.clone(),
            operation: context.operation.to_owned(),
            from_state: context.from_state,
            to_state: self.lease.state,
            occurred_at: context.occurred_at.to_owned(),
            correlation_id: context.correlation_id.to_owned(),
            causation_id: context.causation_id.map(str::to_owned),
            provider_id: context.provider_id.map(str::to_owned),
            purpose: self.lease.purpose.clone(),
            scope: self.lease.scope.clone(),
            outcome: context.outcome.to_owned(),
            failure: context.failure.map(str::to_owned),
        };
        event.validate()?;
        Ok(event)
    }
}

/// Exact external context required to construct lifecycle evidence.
#[derive(Debug, Clone, Copy)]
pub struct ExecutionLeaseEvidenceContext<'a> {
    pub event_id: &'a str,
    pub operation: &'a str,
    pub from_state: Option<CapabilityLeaseState>,
    pub occurred_at: &'a str,
    pub correlation_id: &'a str,
    pub causation_id: Option<&'a str>,
    pub provider_id: Option<&'a str>,
    pub outcome: &'a str,
    pub failure: Option<&'a str>,
}

impl ExecutionLeaseLifecycleEvent {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.event_id.is_empty() {
            return Err("execution lease event id is required");
        }
        if self.lease_ref.id.is_empty() {
            return Err("execution lease reference is required");
        }
        if self.authorization_ref.id.is_empty() {
            return Err("execution lease authorization reference is required");
        }
        if self.capability_ref.id.is_empty() {
            return Err("execution lease capability reference is required");
        }
        if self.subject_ref.id.is_empty() {
            return Err("execution lease subject reference is required");
        }
        if self.operation.is_empty() {
            return Err("execution lease operation is required");
        }
        if self.occurred_at.is_empty() {
            return Err("execution lease event time is required");
        }
        if self.correlation_id.is_empty() {
            return Err("execution lease correlation id is required");
        }
        if self.purpose.is_empty() {
            return Err("execution lease purpose is required");
        }
        if self.scope.is_empty() {
            return Err("execution lease scope is required");
        }
        if self.outcome.is_empty() {
            return Err("execution lease outcome is required");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ResourceType;
    use std::sync::{Arc, Mutex};
    use std::thread;

    #[derive(Default, Clone)]
    struct TestAdapter {
        activations: usize,
        releases: usize,
        fail_release: bool,
    }

    impl ExecutionLeaseAdapter for TestAdapter {
        type Handle = usize;
        type Error = &'static str;

        fn activate(&mut self, _lease: &CapabilityLease) -> Result<Self::Handle, Self::Error> {
            self.activations += 1;
            Ok(self.activations)
        }

        fn release(&mut self, _handle: &mut Self::Handle) -> Result<(), Self::Error> {
            self.releases += 1;
            if self.fail_release {
                return Err("release failed");
            }
            Ok(())
        }
    }

    fn reference(resource_type: ResourceType, id: &str) -> ResourceRef {
        ResourceRef::new(resource_type, id).unwrap()
    }

    fn lease() -> CapabilityLease {
        CapabilityLease {
            lease_id: reference(ResourceType::Other, "lease-1"),
            schema_version: "0.1".into(),
            capability_ref: reference(ResourceType::Other, "camera"),
            resource_ref: Some(reference(ResourceType::Other, "camera-1")),
            subject_ref: reference(ResourceType::Party, "subject-1"),
            actor_ref: Some(reference(ResourceType::Other, "surface-1")),
            purpose: "bounded capture".into(),
            purpose_version: Some("1".into()),
            authorization_ref: reference(ResourceType::Other, "auth-1"),
            incident_ref: None,
            session_ref: Some(reference(ResourceType::Other, "session-1")),
            scope: "camera-1:frame".into(),
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

    fn activate(runtime: &mut ExecutionLeaseRuntime<TestAdapter>) {
        let capability_ref = runtime.lease().capability_ref.clone();
        let resource_ref = runtime.lease().resource_ref.clone();
        let subject_ref = runtime.lease().subject_ref.clone();
        let actor_ref = runtime.lease().actor_ref.clone();
        let purpose = runtime.lease().purpose.clone();
        let purpose_version = runtime.lease().purpose_version.clone();
        let scope = runtime.lease().scope.clone();
        let incident_ref = runtime.lease().incident_ref.clone();
        let session_ref = runtime.lease().session_ref.clone();

        runtime
            .activate(ExecutionLeaseActivationContext {
                now_epoch_seconds: 150,
                capability_ref: &capability_ref,
                resource_ref: resource_ref.as_ref(),
                subject_ref: &subject_ref,
                actor_ref: actor_ref.as_ref(),
                purpose: &purpose,
                purpose_version: purpose_version.as_deref(),
                scope: &scope,
                incident_ref: incident_ref.as_ref(),
                session_ref: session_ref.as_ref(),
            })
            .unwrap();
    }

    #[test]
    fn activation_and_release_form_one_runtime_boundary() {
        let mut runtime = ExecutionLeaseRuntime::new(lease(), TestAdapter::default()).unwrap();
        activate(&mut runtime);
        assert_eq!(runtime.lease().state, CapabilityLeaseState::Active);
        assert_eq!(
            runtime.release(160).unwrap(),
            ExecutionLeaseRuntimeOutcome::Released
        );
        assert_eq!(runtime.lease().state, CapabilityLeaseState::Released);
    }

    #[test]
    fn expiry_is_fail_closed_and_terminal() {
        let mut runtime = ExecutionLeaseRuntime::new(lease(), TestAdapter::default()).unwrap();
        activate(&mut runtime);
        assert_eq!(
            runtime.expire(200).unwrap(),
            ExecutionLeaseRuntimeOutcome::Expired
        );
        assert_eq!(runtime.lease().state, CapabilityLeaseState::Expired);
    }

    #[test]
    fn failed_release_retains_runtime_handle_and_allows_retry() {
        let mut runtime = ExecutionLeaseRuntime::new(
            lease(),
            TestAdapter {
                fail_release: true,
                ..TestAdapter::default()
            },
        )
        .unwrap();
        activate(&mut runtime);

        assert_eq!(
            runtime.release(160),
            Err(ExecutionLeaseRuntimeError::Adapter("release failed"))
        );
        assert_eq!(runtime.lease().state, CapabilityLeaseState::Active);

        runtime.adapter.fail_release = false;
        assert_eq!(
            runtime.release(161).unwrap(),
            ExecutionLeaseRuntimeOutcome::Released
        );
        assert_eq!(runtime.lease().state, CapabilityLeaseState::Released);
        assert_eq!(runtime.adapter.releases, 2);
    }

    #[test]
    fn revocation_prevents_reuse() {
        let mut runtime = ExecutionLeaseRuntime::new(lease(), TestAdapter::default()).unwrap();
        activate(&mut runtime);
        runtime.revoke(160).unwrap();
        assert_eq!(runtime.lease().state, CapabilityLeaseState::Revoked);
        assert!(matches!(
            runtime.release(161),
            Err(ExecutionLeaseRuntimeError::NotActive)
        ));
    }

    #[test]
    fn reauthorization_requires_a_fresh_authorized_lease() {
        let mut runtime = ExecutionLeaseRuntime::new(lease(), TestAdapter::default()).unwrap();
        runtime.revoke(160).unwrap();
        let mut fresh = lease();
        fresh.lease_id = reference(ResourceType::Other, "lease-2");
        fresh.authorization_ref = reference(ResourceType::Other, "auth-2");
        fresh.expires_at_epoch_seconds = 300;
        runtime.reauthorize(fresh).unwrap();
        assert_eq!(runtime.lease().state, CapabilityLeaseState::Authorized);
        assert_eq!(runtime.lease().lease_id.id, "lease-2");
    }

    #[test]
    fn shared_runtime_requires_explicit_synchronization_for_concurrent_callers() {
        let runtime = Arc::new(Mutex::new(
            ExecutionLeaseRuntime::new(lease(), TestAdapter::default()).unwrap(),
        ));
        let left = Arc::clone(&runtime);
        let right = Arc::clone(&runtime);
        let a = thread::spawn(move || left.lock().unwrap().lease().state);
        let b = thread::spawn(move || right.lock().unwrap().lease().state);
        assert_eq!(a.join().unwrap(), CapabilityLeaseState::Authorized);
        assert_eq!(b.join().unwrap(), CapabilityLeaseState::Authorized);
    }
}
