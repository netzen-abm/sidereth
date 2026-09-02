use crate::Id;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditRecord {
    pub audit_id: Id,
    pub actor_id: Id,
    pub action: String,
    pub aggregate_type: String,
    pub aggregate_id: Id,
    pub occurred_at: String,
}

pub trait AuditSink {
    fn record(&mut self, record: AuditRecord) -> Result<(), &'static str>;
}

#[derive(Debug, Default)]
pub struct InMemoryAudit {
    records: Vec<AuditRecord>,
}

impl AuditSink for InMemoryAudit {
    fn record(&mut self, record: AuditRecord) -> Result<(), &'static str> {
        if record.audit_id.is_empty() {
            return Err("audit id is required");
        }
        if record.actor_id.is_empty() {
            return Err("audit actor is required");
        }
        if record.aggregate_id.is_empty() {
            return Err("audit aggregate is required");
        }
        if self.records.iter().any(|item| item.audit_id == record.audit_id) {
            return Err("audit record already exists");
        }
        self.records.push(record);
        Ok(())
    }
}

impl InMemoryAudit {
    pub fn records(&self) -> &[AuditRecord] {
        &self.records
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
        assert_eq!(audit.record(record("audit-1")), Err("audit record already exists"));
    }
}
