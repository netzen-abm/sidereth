use crate::{
    AuditProvenanceSink, AuditRecord, ExecutionLeaseLifecycleEvent, Provenance, ResourceRef,
    ResourceType,
};

/// Trust-boundary adapter from runtime lifecycle evidence to the canonical
/// audit/provenance persistence contract.
///
/// It translates evidence; it does not own runtime state or persistence semantics.
pub struct AuditProvenanceLifecycleSink<S> {
    sink: S,
}

impl<S> AuditProvenanceLifecycleSink<S> {
    pub fn new(sink: S) -> Self {
        Self { sink }
    }

    pub fn into_inner(self) -> S {
        self.sink
    }
}

impl<S: AuditProvenanceSink> AuditProvenanceLifecycleSink<S> {
    pub fn record(&mut self, event: ExecutionLeaseLifecycleEvent) -> Result<(), &'static str> {
        event.validate()?;
        let audit_id = format!("{}:audit", event.event_id);
        let provenance_id = format!("{}:provenance", event.event_id);

        let provenance = Provenance {
            provenance_id: provenance_id.clone(),
            actor_ref: event.actor_ref.clone(),
            source_refs: vec![event.lease_ref.clone()],
            input_refs: Vec::new(),
            operation: event.operation.clone(),
            occurred_at: event.occurred_at.clone(),
        };

        let audit = AuditRecord {
            audit_id,
            actor_id: event
                .actor_ref
                .as_ref()
                .map(|reference| reference.id.clone())
                .unwrap_or_else(|| event.subject_ref.id.clone()),
            action: event.operation,
            aggregate_type: "execution_lease".into(),
            aggregate_id: event.lease_ref.id.clone(),
            occurred_at: event.occurred_at,
            correlation_id: Some(event.correlation_id),
            causation_id: event.causation_id,
            provenance_ref: Some(
                ResourceRef::new(ResourceType::Provenance, provenance_id)
                    .map_err(|_| "invalid lifecycle provenance reference")?,
            ),
            invocation_id: None,
            request_id: None,
            authorization_ref: Some(event.authorization_ref),
            action_ref: None,
            approval_ref: None,
            tool_id: None,
            tool_version: None,
            capability_ref: Some(event.capability_ref),
            function_ref: None,
            provider_id: event.provider_id,
            implementation_id: None,
            implementation_version: None,
            resource_ref: event.resource_ref,
            purpose: Some(event.purpose),
            jurisdiction_ref: None,
            data_class: None,
            requested_scope: Some(event.scope),
            execution_mode: Some("execution_lease_runtime".into()),
            idempotency_ref: None,
            outcome: Some(event.outcome),
            failure: event.failure,
            input_hash: None,
            output_hash: None,
        };

        self.sink.record_invocation(audit, provenance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CapabilityLeaseState, InMemoryAudit};

    fn reference(resource_type: ResourceType, id: &str) -> ResourceRef {
        ResourceRef::new(resource_type, id).unwrap()
    }

    fn event() -> ExecutionLeaseLifecycleEvent {
        ExecutionLeaseLifecycleEvent {
            event_id: "lease-event-1".into(),
            lease_ref: reference(ResourceType::Other, "lease-1"),
            authorization_ref: reference(ResourceType::Other, "auth-1"),
            capability_ref: reference(ResourceType::Other, "camera"),
            resource_ref: Some(reference(ResourceType::Other, "camera-1")),
            subject_ref: reference(ResourceType::Party, "subject-1"),
            actor_ref: Some(reference(ResourceType::Other, "surface-1")),
            operation: "execution_lease.release".into(),
            from_state: Some(CapabilityLeaseState::Active),
            to_state: CapabilityLeaseState::Released,
            occurred_at: "2026-09-19T10:00:00Z".into(),
            correlation_id: "corr-1".into(),
            causation_id: Some("cause-1".into()),
            provider_id: Some("provider-1".into()),
            purpose: "bounded capture".into(),
            scope: "camera-1:frame".into(),
            outcome: "released".into(),
            failure: None,
        }
    }

    #[test]
    fn lifecycle_event_maps_to_existing_atomic_audit_provenance_contract() {
        let mut sink = AuditProvenanceLifecycleSink::new(InMemoryAudit::default());
        sink.record(event()).unwrap();

        let audit = sink.into_inner();
        assert_eq!(audit.records().len(), 1);
        assert_eq!(audit.provenances().len(), 1);
        assert_eq!(
            audit.records()[0]
                .authorization_ref
                .as_ref()
                .unwrap()
                .id,
            "auth-1"
        );
        assert_eq!(audit.records()[0].correlation_id.as_deref(), Some("corr-1"));
        assert_eq!(audit.records()[0].outcome.as_deref(), Some("released"));
    }

    #[test]
    fn invalid_lifecycle_event_is_rejected_before_persistence() {
        let mut value = event();
        value.event_id.clear();
        assert_eq!(
            value.validate(),
            Err("execution lease event id is required")
        );
    }
}
