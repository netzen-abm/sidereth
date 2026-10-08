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
    ) -> Result<ExecutionLeaseRuntimeOutcome, ExecutionLeaseRuntimeError<A::Error>> {
        if self.active_handle.is_some() || self.lease.state == CapabilityLeaseState::Active {
            return Err(ExecutionLeaseRuntimeError::AlreadyActive);
        }

        self.lease
            .validate_activation(
                now_epoch_seconds,
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
            .map_err(ExecutionLeaseRuntimeError::Lease)?;

        let handle = self
            .adapter
            .activate(&self.lease)
            .map_err(ExecutionLeaseRuntimeError::Adapter)?;

        self.lease
            .transition(CapabilityLeaseState::Active, now_epoch_seconds)
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
        runtime
            .activate(
                150,
                &runtime.lease().capability_ref.clone(),
                runtime.lease().resource_ref.as_ref(),
                &runtime.lease().subject_ref.clone(),
                runtime.lease().actor_ref.as_ref(),
                &runtime.lease().purpose.clone(),
                runtime.lease().purpose_version.as_deref(),
                &runtime.lease().scope.clone(),
                runtime.lease().incident_ref.as_ref(),
                runtime.lease().session_ref.as_ref(),
            )
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
