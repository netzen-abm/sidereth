use serde::{Deserialize, Serialize};

use crate::{Id, ResourceRef};

/// Immutable record of an observed system transition or operation.
/// Audit is an observability primitive; it does not replace the domain object,
/// authorization decision, legal authority, or action execution record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditRecord {
    pub audit_id: Id,
    pub actor_id: Id,
    pub action: String,
    pub aggregate_type: String,
    pub aggregate_id: Id,
    pub occurred_at: String,
    pub before_state: Option<String>,
    pub after_state: Option<String>,
    pub reason: Option<String>,
    pub correlation_id: Id,
    pub causation_id: Option<Id>,
    pub provenance_ref: Option<ResourceRef>,
}

impl AuditRecord {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.audit_id.is_empty() { return Err("audit id is required"); }
        if self.actor_id.is_empty() { return Err("audit actor is required"); }
        if self.action.is_empty() { return Err("audit action is required"); }
        if self.aggregate_type.is_empty() || self.aggregate_id.is_empty() {
            return Err("audit aggregate is required");
        }
        if self.occurred_at.is_empty() { return Err("audit time is required"); }
        if self.correlation_id.is_empty() { return Err("audit correlation id is required"); }
        Ok(())
    }
}

pub trait AuditSink {
    fn record(&mut self, record: AuditRecord) -> Result<(), &'static str>;
}

#[derive(Debug, Default)]
pub struct InMemoryAudit { records: Vec<AuditRecord> }

impl AuditSink for InMemoryAudit {
    fn record(&mut self, record: AuditRecord) -> Result<(), &'static str> {
        record.validate()?;
        if self.records.iter().any(|item| item.audit_id == record.audit_id) {
            return Err("audit record already exists");
        }
        self.records.push(record);
        Ok(())
    }
}

impl InMemoryAudit { pub fn records(&self) -> &[AuditRecord] { &self.records } }

#[cfg(test)]
mod tests {
    use super::*;
    fn record(id: &str) -> AuditRecord {
        AuditRecord {
            audit_id: id.into(), actor_id: "user-1".into(), action: "case.create".into(),
            aggregate_type: "case".into(), aggregate_id: "case-1".into(),
            occurred_at: "2026-09-02T00:00:00Z".into(), before_state: None,
            after_state: Some("active".into()), reason: Some("created".into()),
            correlation_id: "corr-1".into(), causation_id: None, provenance_ref: None,
        }
    }
    #[test] fn audit_record_is_stored() {
        let mut audit = InMemoryAudit::default(); audit.record(record("audit-1")).unwrap();
        assert_eq!(audit.records().len(), 1);
    }
    #[test] fn duplicate_audit_id_is_rejected() {
        let mut audit = InMemoryAudit::default(); audit.record(record("audit-1")).unwrap();
        assert_eq!(audit.record(record("audit-1")), Err("audit record already exists"));
    }
    #[test] fn audit_record_has_transition_context() {
        let value = record("audit-1");
        assert_eq!(value.before_state, None);
        assert_eq!(value.after_state.as_deref(), Some("active"));
        assert_eq!(value.correlation_id, "corr-1");
    }
}
