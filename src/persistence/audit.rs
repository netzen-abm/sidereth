use serde::Serialize;
use serde_json::Value;

use crate::audit::{AuditProvenanceSink, AuditRecord, AuditSink};
use crate::persistence::{ResourceWrite, ResourceWriteMode, UnitOfWorkError, UnitOfWorkFactory};
use crate::Provenance;

/// Persistence adapter that atomically stores an audit record and its provenance
/// through the existing provider-neutral UnitOfWork contract.
///
/// This type owns only the audit-to-persistence mapping. It does not define
/// audit semantics, provenance semantics, authorization, or database behavior.
pub struct UnitOfWorkAuditProvenanceSink<F: UnitOfWorkFactory> {
    factory: F,
}

impl<F: UnitOfWorkFactory> UnitOfWorkAuditProvenanceSink<F> {
    pub fn new(factory: F) -> Self {
        Self { factory }
    }

    pub fn into_factory(self) -> F {
        self.factory
    }

    fn persist_pair(
        &mut self,
        record: AuditRecord,
        provenance: Provenance,
    ) -> Result<(), &'static str> {
        record.validate()?;
        provenance.validate()?;

        let audit_ref =
            crate::ResourceRef::new(crate::ResourceType::Audit, record.audit_id.clone())
                .map_err(|_| "invalid audit resource reference")?;
        let provenance_ref = crate::ResourceRef::new(
            crate::ResourceType::Provenance,
            provenance.provenance_id.clone(),
        )
        .map_err(|_| "invalid provenance resource reference")?;

        let audit_payload =
            serde_json::to_value(&record).map_err(|_| "cannot serialize audit record")?;
        let provenance_payload =
            serde_json::to_value(&provenance).map_err(|_| "cannot serialize provenance")?;

        let audit_write =
            ResourceWrite::new(audit_ref, 1, audit_payload, ResourceWriteMode::Insert)
                .map_err(|_| "invalid audit resource write")?;
        let provenance_write = ResourceWrite::new(
            provenance_ref,
            1,
            provenance_payload,
            ResourceWriteMode::Insert,
        )
        .map_err(|_| "invalid provenance resource write")?;

        let mut uow = self
            .factory
            .begin()
            .map_err(|_| "cannot begin audit persistence unit of work")?;

        let result = uow.execute(|context| {
            context.write_resource(audit_write)?;
            context.write_resource(provenance_write)?;
            Ok(())
        });

        match result {
            Ok(()) => uow
                .commit()
                .map_err(|_| "cannot commit audit persistence unit of work"),
            Err(error) => {
                let _ = uow.rollback();
                Err(map_unit_of_work_error(error))
            }
        }
    }
}

impl<F: UnitOfWorkFactory> AuditSink for UnitOfWorkAuditProvenanceSink<F> {
    fn record(&mut self, _record: AuditRecord) -> Result<(), &'static str> {
        Err("audit provenance pair is required")
    }
}

impl<F: UnitOfWorkFactory> AuditProvenanceSink for UnitOfWorkAuditProvenanceSink<F> {
    fn record_invocation(
        &mut self,
        record: AuditRecord,
        provenance: Provenance,
    ) -> Result<(), &'static str> {
        self.persist_pair(record, provenance)
    }
}

fn map_unit_of_work_error(error: UnitOfWorkError) -> &'static str {
    match error {
        UnitOfWorkError::InvalidOperation => "invalid audit persistence operation",
        UnitOfWorkError::Persistence(_) => "audit persistence operation failed",
    }
}

#[allow(dead_code)]
fn _serialization_boundary<T: Serialize>(value: &T) -> Result<Value, &'static str> {
    serde_json::to_value(value).map_err(|_| "cannot serialize persistence value")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::{ResourceRecord, UnitOfWork, UnitOfWorkContext, UnitOfWorkFactory};
    use crate::{Id, ResourceLink, ResourceRef};
    use std::cell::RefCell;
    use std::collections::HashMap;

    #[derive(Default)]
    struct TestContext {
        resources: HashMap<String, ResourceRecord>,
    }

    impl UnitOfWorkContext for TestContext {
        fn read_resource(
            &mut self,
            resource_ref: &ResourceRef,
        ) -> Result<Option<ResourceRecord>, UnitOfWorkError> {
            Ok(self.resources.get(&resource_ref.id).cloned())
        }

        fn write_resource(&mut self, write: ResourceWrite) -> Result<(), UnitOfWorkError> {
            let key = format!(
                "{:?}:{}",
                write.resource_ref.resource_type, write.resource_ref.id
            );
            if write.mode == ResourceWriteMode::Insert && self.resources.contains_key(&key) {
                return Err(UnitOfWorkError::Persistence(
                    crate::PersistenceError::Duplicate,
                ));
            }
            self.resources.insert(
                key,
                ResourceRecord {
                    resource_ref: write.resource_ref,
                    schema_version: write.schema_version,
                    revision: crate::Revision::initial(),
                    payload: write.payload,
                },
            );
            Ok(())
        }

        fn link_resources(&mut self, _link: ResourceLink) -> Result<(), UnitOfWorkError> {
            Ok(())
        }
    }

    struct TestUnitOfWork {
        context: TestContext,
        committed: bool,
        rolled_back: bool,
    }

    impl UnitOfWork for TestUnitOfWork {
        type Context = TestContext;

        fn execute<R, F>(&mut self, operation: F) -> Result<R, UnitOfWorkError>
        where
            F: FnOnce(&mut Self::Context) -> Result<R, UnitOfWorkError>,
        {
            operation(&mut self.context)
        }

        fn commit(mut self) -> Result<(), crate::PersistenceError> {
            self.committed = true;
            Ok(())
        }

        fn rollback(mut self) -> Result<(), crate::PersistenceError> {
            self.rolled_back = true;
            Ok(())
        }
    }

    #[derive(Default)]
    struct TestFactory {
        begins: RefCell<usize>,
    }

    impl UnitOfWorkFactory for TestFactory {
        type Uow = TestUnitOfWork;

        fn begin(&mut self) -> Result<Self::Uow, crate::PersistenceError> {
            *self.begins.borrow_mut() += 1;
            Ok(TestUnitOfWork {
                context: TestContext::default(),
                committed: false,
                rolled_back: false,
            })
        }
    }

    fn record() -> AuditRecord {
        AuditRecord {
            audit_id: "audit-1".into(),
            actor_id: Id::from("actor-1"),
            action: "lease.activate".into(),
            aggregate_type: "other".into(),
            aggregate_id: Id::from("lease-1"),
            occurred_at: "2026-10-08T00:00:00Z".into(),
            correlation_id: None,
            causation_id: None,
            provenance_ref: Some(ResourceRef::new(
                crate::ResourceType::Provenance,
                "prov-1",
            ).unwrap()),
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
            purpose: Some("bounded execution".into()),
            jurisdiction_ref: None,
            data_class: None,
            requested_scope: Some("lease-1".into()),
            execution_mode: Some("sync".into()),
            idempotency_ref: None,
            outcome: Some("activated".into()),
            failure: None,
            input_hash: None,
            output_hash: None,
        }
    }

    fn provenance() -> Provenance {
        Provenance {
            provenance_id: "prov-1".into(),
            actor_ref: None,
            source_refs: Vec::new(),
            input_refs: Vec::new(),
            operation: "execution-lease.activated".into(),
            occurred_at: "2026-10-08T00:00:00Z".into(),
        }
    }

    #[test]
    fn atomic_sink_maps_pair_to_existing_resource_write_boundary() {
        let mut sink = UnitOfWorkAuditProvenanceSink::new(TestFactory::default());
        sink.record_invocation(record(), provenance()).unwrap();
    }

    #[test]
    fn plain_audit_record_is_rejected_to_preserve_atomic_pair_semantics() {
        let mut sink = UnitOfWorkAuditProvenanceSink::new(TestFactory::default());
        assert_eq!(
            sink.record(record()),
            Err("audit provenance pair is required")
        );
    }
}
