use crate::persistence::{
    PersistenceError, ResourceLink, ResourceWrite, ResourceWriteMode, UnitOfWork, UnitOfWorkError,
    UnitOfWorkFactory,
};
use crate::{Id, ResourceRef, ResourceType};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtomicCommandPlan {
    pub operation_id: Id,
    pub resource_writes: Vec<ResourceWrite>,
    pub resource_links: Vec<ResourceLink>,
}

impl AtomicCommandPlan {
    pub fn new(operation_id: impl Into<Id>) -> Result<Self, UnitOfWorkError> {
        let operation_id = operation_id.into();
        if operation_id.trim().is_empty() {
            return Err(UnitOfWorkError::InvalidOperation);
        }
        Ok(Self {
            operation_id,
            resource_writes: Vec::new(),
            resource_links: Vec::new(),
        })
    }

    pub fn insert_resource(
        &mut self,
        resource_ref: ResourceRef,
        schema_version: u16,
        payload: Value,
    ) -> Result<(), UnitOfWorkError> {
        self.resource_writes.push(ResourceWrite::new(
            resource_ref,
            schema_version,
            payload,
            ResourceWriteMode::Insert,
        )?);
        Ok(())
    }

    pub fn upsert_resource(
        &mut self,
        resource_ref: ResourceRef,
        schema_version: u16,
        payload: Value,
    ) -> Result<(), UnitOfWorkError> {
        self.resource_writes.push(ResourceWrite::new(
            resource_ref,
            schema_version,
            payload,
            ResourceWriteMode::Upsert,
        )?);
        Ok(())
    }

    pub fn link(&mut self, link: ResourceLink) {
        self.resource_links.push(link);
    }

    pub fn claim_operation(&mut self) -> Result<(), UnitOfWorkError> {
        let resource_ref = ResourceRef::new(ResourceType::Idempotency, self.operation_id.clone())
            .map_err(|_| UnitOfWorkError::InvalidOperation)?;
        self.insert_resource(
            resource_ref,
            1,
            serde_json::json!({ "operation_id": self.operation_id }),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthoritativeCommandError {
    Persistence(PersistenceError),
    InvalidOperation,
    RollbackFailure {
        operation: Box<Self>,
        rollback: PersistenceError,
    },
}

impl From<UnitOfWorkError> for AuthoritativeCommandError {
    fn from(error: UnitOfWorkError) -> Self {
        match error {
            UnitOfWorkError::Persistence(error) => Self::Persistence(error),
            UnitOfWorkError::InvalidOperation => Self::InvalidOperation,
        }
    }
}

pub fn execute_authoritative_command<F, R, Build>(
    factory: &mut F,
    plan: AtomicCommandPlan,
    build: Build,
) -> Result<R, AuthoritativeCommandError>
where
    F: UnitOfWorkFactory,
    Build: FnOnce(
        &mut <F::Uow as UnitOfWork>::Context,
        &AtomicCommandPlan,
    ) -> Result<R, UnitOfWorkError>,
{
    let mut uow = factory
        .begin()
        .map_err(AuthoritativeCommandError::Persistence)?;
    let result = uow.execute(|context| {
        apply_plan(context, &plan)?;
        build(context, &plan)
    });
    match result {
        Ok(value) => {
            uow.commit()
                .map_err(AuthoritativeCommandError::Persistence)?;
            Ok(value)
        }
        Err(error) => {
            let operation = AuthoritativeCommandError::from(error);
            match uow.rollback() {
                Ok(()) => Err(operation),
                Err(rollback) => Err(AuthoritativeCommandError::RollbackFailure {
                    operation: Box::new(operation),
                    rollback,
                }),
            }
        }
    }
}

pub fn apply_plan<C: crate::persistence::UnitOfWorkContext>(
    context: &mut C,
    plan: &AtomicCommandPlan,
) -> Result<(), UnitOfWorkError> {
    for write in plan.resource_writes.iter().cloned() {
        context.write_resource(write)?;
    }
    for link in plan.resource_links.iter().cloned() {
        context.link_resources(link)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::{ResourceRecord, UnitOfWorkContext};

    #[derive(Default)]
    struct MockContext {
        writes: Vec<ResourceWrite>,
        links: Vec<ResourceLink>,
    }

    impl UnitOfWorkContext for MockContext {
        fn read_resource(
            &mut self,
            _resource_ref: &ResourceRef,
        ) -> Result<Option<ResourceRecord>, UnitOfWorkError> {
            Ok(None)
        }

        fn write_resource(&mut self, write: ResourceWrite) -> Result<(), UnitOfWorkError> {
            self.writes.push(write);
            Ok(())
        }

        fn link_resources(&mut self, link: ResourceLink) -> Result<(), UnitOfWorkError> {
            self.links.push(link);
            Ok(())
        }
    }

    struct MockUow {
        context: MockContext,
        fail_rollback: bool,
        committed: bool,
        rolled_back: bool,
    }

    impl UnitOfWork for MockUow {
        type Context = MockContext;

        fn execute<R, F>(&mut self, operation: F) -> Result<R, UnitOfWorkError>
        where
            F: FnOnce(&mut Self::Context) -> Result<R, UnitOfWorkError>,
        {
            operation(&mut self.context)
        }

        fn commit(mut self) -> Result<(), PersistenceError> {
            self.committed = true;
            Ok(())
        }

        fn rollback(mut self) -> Result<(), PersistenceError> {
            self.rolled_back = true;
            if self.fail_rollback {
                Err(PersistenceError::Unavailable)
            } else {
                Ok(())
            }
        }
    }

    #[derive(Default)]
    struct MockFactory {
        fail_rollback: bool,
    }

    impl UnitOfWorkFactory for MockFactory {
        type Uow = MockUow;

        fn begin(&mut self) -> Result<Self::Uow, PersistenceError> {
            Ok(MockUow {
                context: MockContext::default(),
                fail_rollback: self.fail_rollback,
                committed: false,
                rolled_back: false,
            })
        }
    }

    #[test]
    fn plan_contains_idempotency_and_domain_side_effects() {
        let mut plan = AtomicCommandPlan::new("op-1").unwrap();
        plan.claim_operation().unwrap();
        plan.insert_resource(
            ResourceRef::new(ResourceType::Case, "case-1").unwrap(),
            1,
            serde_json::json!({ "state": "active" }),
        )
        .unwrap();
        plan.insert_resource(
            ResourceRef::new(ResourceType::Event, "event-1").unwrap(),
            1,
            serde_json::json!({ "event_type": "case.created" }),
        )
        .unwrap();
        plan.insert_resource(
            ResourceRef::new(ResourceType::Audit, "audit-1").unwrap(),
            1,
            serde_json::json!({ "action": "case.created" }),
        )
        .unwrap();
        plan.insert_resource(
            ResourceRef::new(ResourceType::Provenance, "prov-1").unwrap(),
            1,
            serde_json::json!({ "operation": "case.create" }),
        )
        .unwrap();
        assert_eq!(plan.resource_writes.len(), 5);
        assert_eq!(
            plan.resource_writes[0].resource_ref.resource_type,
            ResourceType::Idempotency
        );
    }

    #[test]
    fn apply_plan_writes_every_side_effect_through_one_context() {
        let mut plan = AtomicCommandPlan::new("op-2").unwrap();
        plan.claim_operation().unwrap();
        plan.insert_resource(
            ResourceRef::new(ResourceType::Case, "case-2").unwrap(),
            1,
            serde_json::json!({ "state": "draft" }),
        )
        .unwrap();
        plan.link(
            ResourceLink::new(
                ResourceRef::new(ResourceType::Case, "case-2").unwrap(),
                "has_event",
                ResourceRef::new(ResourceType::Event, "event-2").unwrap(),
            )
            .unwrap(),
        );
        let mut context = MockContext::default();
        apply_plan(&mut context, &plan).unwrap();
        assert_eq!(context.writes.len(), 2);
        assert_eq!(context.links.len(), 1);
    }

    #[test]
    fn executor_rolls_back_when_build_fails() {
        let plan = AtomicCommandPlan::new("op-3").unwrap();
        let mut factory = MockFactory::default();
        let result = execute_authoritative_command(&mut factory, plan, |_context, _| {
            Err(UnitOfWorkError::InvalidOperation)
        });
        assert_eq!(result, Err(AuthoritativeCommandError::InvalidOperation));
    }
}
