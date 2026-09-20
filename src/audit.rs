use serde::{Deserialize, Serialize};

use crate::Id;

/// Immutable record of an observed system operation.
/// Transition-specific context belongs to LifecycleTransition so existing
/// AuditRecord producers remain source-compatible.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditRecord {
    pub audit_id: Id,
    pub actor_id: Id,
    pub action: String,
    pub aggregate_type: String,
    pub aggregate_id: Id,
    pub occurred_at: String,
    /// Optional correlation identifier for the operation that caused this audit record.
    #[serde(default)]
    pub correlation_id: Option<Id>,
    /// Optional causation identifier linking this record to a preceding operation.
    #[serde(default)]
    pub causation_id: Option<Id>,
    /// Canonical provenance reference for the observed operation.
    #[serde(default)]
    pub provenance_ref: Option<crate::ResourceRef>,
    /// Optional invocation identity when this audit records Tool Gateway execution.
    #[serde(default)]
    pub invocation_id: Option<Id>,
    #[serde(default)]
    pub request_id: Option<Id>,
    #[serde(default)]
    pub authorization_ref: Option<crate::ResourceRef>,
    #[serde(default)]
    pub action_ref: Option<crate::ResourceRef>,
    #[serde(default)]
    pub approval_ref: Option<crate::ResourceRef>,
    #[serde(default)]
    pub tool_id: Option<Id>,
    #[serde(default)]
    pub tool_version: Option<String>,
    #[serde(default)]
    pub capability_ref: Option<crate::ResourceRef>,
    #[serde(default)]
    pub function_ref: Option<crate::ResourceRef>,
    #[serde(default)]
    pub provider_id: Option<Id>,
    #[serde(default)]
    pub implementation_id: Option<Id>,
    #[serde(default)]
    pub implementation_version: Option<String>,
    #[serde(default)]
    pub resource_ref: Option<crate::ResourceRef>,
    #[serde(default)]
    pub purpose: Option<String>,
    #[serde(default)]
    pub jurisdiction_ref: Option<crate::ResourceRef>,
    #[serde(default)]
    pub data_class: Option<String>,
    #[serde(default)]
    pub requested_scope: Option<String>,
    #[serde(default)]
    pub execution_mode: Option<String>,
    #[serde(default)]
    pub idempotency_ref: Option<Id>,
    #[serde(default)]
    pub outcome: Option<String>,
    #[serde(default)]
    pub failure: Option<String>,
    #[serde(default)]
    pub input_hash: Option<String>,
    #[serde(default)]
    pub output_hash: Option<String>,
}

impl AuditRecord {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.audit_id.is_empty() {
            return Err("audit id is required");
        }
        if self.actor_id.is_empty() {
            return Err("audit actor is required");
        }
        if self.action.is_empty() {
            return Err("audit action is required");
        }
        if self.aggregate_type.is_empty() || self.aggregate_id.is_empty() {
            return Err("audit aggregate is required");
        }
        if self.occurred_at.is_empty() {
            return Err("audit time is required");
        }
        Ok(())
    }
}

pub trait AuditSink {
    fn record(&mut self, record: AuditRecord) -> Result<(), &'static str>;
}

/// Canonical atomic sink for an invocation audit record and its provenance.
///
/// Implementations must persist the pair atomically or return an error.
/// The Tool Gateway never treats this sink as an authorization mechanism.
pub trait AuditProvenanceSink: AuditSink {
    fn record_invocation(
        &mut self,
        record: AuditRecord,
        provenance: crate::Provenance,
    ) -> Result<(), &'static str>;
}

#[derive(Debug, Default)]
pub struct InMemoryAudit {
    records: Vec<AuditRecord>,
    provenances: Vec<crate::Provenance>,
}

impl AuditSink for InMemoryAudit {
    fn record(&mut self, record: AuditRecord) -> Result<(), &'static str> {
        record.validate()?;
        if self
            .records
            .iter()
            .any(|item| item.audit_id == record.audit_id)
        {
            return Err("audit record already exists");
        }
        self.records.push(record);
        Ok(())
    }
}

impl AuditProvenanceSink for InMemoryAudit {
    fn record_invocation(
        &mut self,
        record: AuditRecord,
        provenance: crate::Provenance,
    ) -> Result<(), &'static str> {
        record.validate()?;
        provenance.validate()?;
        if self
            .records
            .iter()
            .any(|item| item.audit_id == record.audit_id)
        {
            return Err("audit record already exists");
        }
        if self
            .provenances
            .iter()
            .any(|item| item.provenance_id == provenance.provenance_id)
        {
            return Err("provenance record already exists");
        }
        self.records.push(record);
        self.provenances.push(provenance);
        Ok(())
    }
}

impl InMemoryAudit {
    pub fn records(&self) -> &[AuditRecord] {
        &self.records
    }

    pub fn provenances(&self) -> &[crate::Provenance] {
        &self.provenances
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(id: &str) -> AuditRecord {
        AuditRecord {
            audit_id: id.into(),
            actor_id: "user-1".into(),
            action: "case.create".into(),
            aggregate_type: "case".into(),
            aggregate_id: "case-1".into(),
            occurred_at: "2026-09-02T00:00:00Z".into(),
            correlation_id: None,
            causation_id: None,
            provenance_ref: None,
            invocation_id: None,
            request_id: None,
            authorization_ref: None,
            action_ref: None,
            approval_ref: None,
            tool_id: None,
            tool_version: None,
            capability_ref: None,
            function_ref: None,
            provider_id: None,
            implementation_id: None,
            implementation_version: None,
            resource_ref: None,
            purpose: None,
            jurisdiction_ref: None,
            data_class: None,
            requested_scope: None,
            execution_mode: None,
            idempotency_ref: None,
            outcome: None,
            failure: None,
            input_hash: None,
            output_hash: None,
        }
    }

    #[test]
    fn audit_record_is_stored() {
        let mut audit = InMemoryAudit::default();
        audit.record(record("audit-1")).unwrap();
        assert_eq!(audit.records().len(), 1);
    }

    #[test]
    fn duplicate_audit_id_is_rejected() {
        let mut audit = InMemoryAudit::default();
        audit.record(record("audit-1")).unwrap();
        assert_eq!(
            audit.record(record("audit-1")),
            Err("audit record already exists")
        );
    }

    #[test]
    fn audit_record_validation_is_explicit() {
        let mut value = record("audit-1");
        value.action.clear();
        assert_eq!(value.validate(), Err("audit action is required"));
    }
}
