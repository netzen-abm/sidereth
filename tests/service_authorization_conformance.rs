use sidereth::authorization::{
    AuthorizationDecision, AuthorizationEvaluator, AuthorizationRequest, AuthorizationResult,
};
use sidereth::persistence::{
    PersistenceError, ResourceRecord, ResourceWrite, ResourceWriteMode, Revision, UnitOfWork,
    UnitOfWorkContext, UnitOfWorkError, UnitOfWorkFactory,
};
use sidereth::{Case, CaseCommand, CaseService, CommandContext, ResourceRef, ResourceType, ServiceError};
use serde_json::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, Default)]
struct State(Rc<RefCell<HashMap<ResourceRef, ResourceRecord>>>);

struct MockContext {
    state: State,
}

impl UnitOfWorkContext for MockContext {
    fn read_resource(
        &mut self,
        resource_ref: &ResourceRef,
    ) -> Result<Option<ResourceRecord>, UnitOfWorkError> {
        Ok(self.state.0.borrow().get(resource_ref).cloned())
    }

    fn write_resource(&mut self, write: ResourceWrite) -> Result<(), UnitOfWorkError> {
        let mut state = self.state.0.borrow_mut();
        let current = state.get(&write.resource_ref).cloned();
        match (write.mode, current, write.expected_revision) {
            (ResourceWriteMode::Insert, Some(_), _) => {
                Err(UnitOfWorkError::Persistence(PersistenceError::Duplicate))
            }
            (ResourceWriteMode::Insert, None, _) => {
                state.insert(
                    write.resource_ref.clone(),
                    ResourceRecord {
                        resource_ref: write.resource_ref,
                        schema_version: write.schema_version,
                        revision: Revision::initial(),
                        payload: write.payload,
                    },
                );
                Ok(())
            }
            (ResourceWriteMode::Upsert, Some(current), Some(expected))
                if current.revision != expected =>
            {
                Err(UnitOfWorkError::Persistence(PersistenceError::Conflict))
            }
            (ResourceWriteMode::Upsert, Some(_), Some(expected)) => {
                state.insert(
                    write.resource_ref.clone(),
                    ResourceRecord {
                        resource_ref: write.resource_ref,
                        schema_version: write.schema_version,
                        revision: expected.next()?,
                        payload: write.payload,
                    },
                );
                Ok(())
            }
            (ResourceWriteMode::Upsert, None, Some(_)) => {
                Err(UnitOfWorkError::Persistence(PersistenceError::Conflict))
            }
            (ResourceWriteMode::Upsert, Some(current), None) => {
                state.insert(
                    write.resource_ref.clone(),
                    ResourceRecord {
                        resource_ref: write.resource_ref,
                        schema_version: write.schema_version,
                        revision: current.revision.next()?,
                        payload: write.payload,
                    },
                );
                Ok(())
            }
            (ResourceWriteMode::Upsert, None, None) => {
                state.insert(
                    write.resource_ref.clone(),
                    ResourceRecord {
                        resource_ref: write.resource_ref,
                        schema_version: write.schema_version,
                        revision: Revision::initial(),
                        payload: write.payload,
                    },
                );
                Ok(())
            }
        }
    }

    fn link_resources(
        &mut self,
        _link: sidereth::persistence::ResourceLink,
    ) -> Result<(), UnitOfWorkError> {
        Ok(())
    }
}

struct MockUow {
    context: MockContext,
    snapshot: HashMap<ResourceRef, ResourceRecord>,
}

impl UnitOfWork for MockUow {
    type Context = MockContext;

    fn execute<R, F>(&mut self, operation: F) -> Result<R, UnitOfWorkError>
    where
        F: FnOnce(&mut Self::Context) -> Result<R, UnitOfWorkError>,
    {
        operation(&mut self.context)
    }

    fn commit(self) -> Result<(), PersistenceError> {
        Ok(())
    }

    fn rollback(self) -> Result<(), PersistenceError> {
        let mut state = self.context.state.0.borrow_mut();
        state.clear();
        state.extend(self.snapshot);
        Ok(())
    }
}

#[derive(Clone, Default)]
struct MockFactory {
    state: State,
    begins: Rc<RefCell<usize>>,
}

impl UnitOfWorkFactory for MockFactory {
    type Uow = MockUow;

    fn begin(&mut self) -> Result<Self::Uow, PersistenceError> {
        *self.begins.borrow_mut() += 1;
        let snapshot = self.state.0.borrow().clone();
        Ok(MockUow {
            context: MockContext {
                state: self.state.clone(),
            },
            snapshot,
        })
    }
}

#[derive(Clone)]
struct FixedEvaluator {
    result: AuthorizationResult,
}

impl AuthorizationEvaluator for FixedEvaluator {
    fn evaluate(&self, _request: &AuthorizationRequest) -> AuthorizationResult {
        self.result.clone()
    }
}

fn reference(resource_type: ResourceType, id: &str) -> ResourceRef {
    ResourceRef::new(resource_type, id).unwrap()
}

fn context() -> CommandContext {
    CommandContext {
        request_id: "req-1".into(),
        authorization_ref: reference(ResourceType::Other, "auth-1"),
        actor_id: "user-1".into(),
        operation_id: "op-1".into(),
        correlation_id: "corr-1".into(),
        purpose: "case management".into(),
        policy_refs: vec![reference(ResourceType::Other, "policy-1")],
        jurisdiction_ref: Some(reference(ResourceType::Jurisdiction, "jurisdiction-1")),
        data_class: Some("case-restricted".into()),
        requested_at_epoch_seconds: 100,
        freshness_seconds: Some(60),
        now_epoch_seconds: 110,
    }
}

fn authorization_for(context: &CommandContext, decision: AuthorizationDecision) -> AuthorizationResult {
    AuthorizationResult {
        request_id: context.request_id.clone(),
        authorization_ref: context.authorization_ref.clone(),
        subject_ref: reference(ResourceType::Party, &context.actor_id),
        action: reference(ResourceType::Action, "case.create"),
        resource_ref: reference(ResourceType::Case, "case-1"),
        purpose: context.purpose.clone(),
        jurisdiction_ref: context.jurisdiction_ref.clone(),
        data_class: context.data_class.clone(),
        decision,
        constraints: Vec::new(),
        policy_refs: context.policy_refs.clone(),
        evaluated_at_epoch_seconds: 100,
        expires_at_epoch_seconds: Some(160),
    }
}

fn create_command() -> CaseCommand {
    CaseCommand::Create {
        case: Case::new("case-1".into()).unwrap(),
    }
}

#[test]
fn allow_preserves_authoritative_case_execution_and_side_effects() {
    let mut factory = MockFactory::default();
    let state = factory.state.clone();
    let evaluator = FixedEvaluator {
        result: authorization_for(&context(), AuthorizationDecision::Allow),
    };
    let mut service = CaseService::new(&mut factory, &evaluator);

    let result = service.execute(context(), create_command()).unwrap();

    assert_eq!(result.case_id, "case-1");
    assert!(state.0.borrow().contains_key(&reference(ResourceType::Case, "case-1")));
    assert!(state.0.borrow().contains_key(&reference(ResourceType::Event, &result.event_id)));
    assert!(state.0.borrow().contains_key(&reference(ResourceType::Audit, "audit-op-1")));
    assert!(state.0.borrow().contains_key(&reference(ResourceType::Provenance, "provenance-op-1")));
    assert!(state.0.borrow().contains_key(&reference(ResourceType::Idempotency, "op-1")));
}

#[test]
fn deny_does_not_begin_unit_of_work_or_claim_idempotency() {
    let mut factory = MockFactory::default();
    let state = factory.state.clone();
    let begins = factory.begins.clone();
    let evaluator = FixedEvaluator {
        result: authorization_for(&context(), AuthorizationDecision::Deny),
    };
    let mut service = CaseService::new(&mut factory, &evaluator);

    assert_eq!(
        service.execute(context(), create_command()),
        Err(ServiceError::AuthorizationDenied)
    );
    assert_eq!(*begins.borrow(), 0);
    assert!(state.0.borrow().is_empty());
}

#[test]
fn not_applicable_fails_closed_before_idempotency() {
    let mut factory = MockFactory::default();
    let begins = factory.begins.clone();
    let evaluator = FixedEvaluator {
        result: authorization_for(&context(), AuthorizationDecision::NotApplicable),
    };
    let mut service = CaseService::new(&mut factory, &evaluator);

    assert_eq!(
        service.execute(context(), create_command()),
        Err(ServiceError::AuthorizationDenied)
    );
    assert_eq!(*begins.borrow(), 0);
}

#[test]
fn mismatched_subject_is_rejected_even_when_decision_is_allow() {
    let mut factory = MockFactory::default();
    let begins = factory.begins.clone();
    let mut result = authorization_for(&context(), AuthorizationDecision::Allow);
    result.subject_ref = reference(ResourceType::Party, "different-user");
    let evaluator = FixedEvaluator { result };
    let mut service = CaseService::new(&mut factory, &evaluator);

    assert_eq!(
        service.execute(context(), create_command()),
        Err(ServiceError::AuthorizationMismatch)
    );
    assert_eq!(*begins.borrow(), 0);
}

#[test]
fn mismatched_purpose_data_class_and_resource_are_rejected() {
    for mutation in [0, 1, 2] {
        let mut factory = MockFactory::default();
        let begins = factory.begins.clone();
        let mut result = authorization_for(&context(), AuthorizationDecision::Allow);
        match mutation {
            0 => result.purpose = "different-purpose".into(),
            1 => result.data_class = Some("less-restricted".into()),
            2 => result.resource_ref = reference(ResourceType::Case, "different-case"),
            _ => unreachable!(),
        }
        let evaluator = FixedEvaluator { result };
        let mut service = CaseService::new(&mut factory, &evaluator);
        assert_eq!(
            service.execute(context(), create_command()),
            Err(ServiceError::AuthorizationMismatch)
        );
        assert_eq!(*begins.borrow(), 0);
    }
}

#[test]
fn expired_authorization_is_rejected_before_idempotency() {
    let mut factory = MockFactory::default();
    let begins = factory.begins.clone();
    let mut result = authorization_for(&context(), AuthorizationDecision::Allow);
    result.expires_at_epoch_seconds = Some(109);
    let evaluator = FixedEvaluator { result };
    let mut service = CaseService::new(&mut factory, &evaluator);

    assert_eq!(
        service.execute(context(), create_command()),
        Err(ServiceError::AuthorizationExpired)
    );
    assert_eq!(*begins.borrow(), 0);
}

#[test]
fn evaluator_action_binding_for_create_is_canonical_case_create() {
    struct RecordingEvaluator(Rc<RefCell<Option<AuthorizationRequest>>>);

    impl AuthorizationEvaluator for RecordingEvaluator {
        fn evaluate(&self, request: &AuthorizationRequest) -> AuthorizationResult {
            *self.0.borrow_mut() = Some(request.clone());
            AuthorizationResult {
                request_id: request.request_id.clone(),
                authorization_ref: request.authorization_ref.clone(),
                subject_ref: request.subject_ref.clone(),
                action: request.action.clone(),
                resource_ref: request.resource_ref.clone(),
                purpose: request.purpose.clone(),
                jurisdiction_ref: request.jurisdiction_ref.clone(),
                data_class: request.data_class.clone(),
                decision: AuthorizationDecision::Deny,
                constraints: Vec::new(),
                policy_refs: request.policy_refs.clone(),
                evaluated_at_epoch_seconds: 110,
                expires_at_epoch_seconds: None,
            }
        }
    }

    let captured = Rc::new(RefCell::new(None));
    let evaluator = RecordingEvaluator(captured.clone());
    let mut factory = MockFactory::default();
    let mut service = CaseService::new(&mut factory, &evaluator);
    let _ = service.execute(context(), create_command());

    let request = captured.borrow().clone().unwrap();
    assert_eq!(request.action, reference(ResourceType::Action, "case.create"));
    assert_eq!(request.resource_ref, reference(ResourceType::Case, "case-1"));
    assert_eq!(request.subject_ref, reference(ResourceType::Party, "user-1"));
}

#[test]
fn mismatched_authorization_reference_is_rejected() {
    let mut factory = MockFactory::default();
    let begins = factory.begins.clone();
    let mut result = authorization_for(&context(), AuthorizationDecision::Allow);
    result.authorization_ref = reference(ResourceType::Other, "different-auth");
    let evaluator = FixedEvaluator { result };
    let mut service = CaseService::new(&mut factory, &evaluator);

    assert_eq!(
        service.execute(context(), create_command()),
        Err(ServiceError::AuthorizationMismatch)
    );
    assert_eq!(*begins.borrow(), 0);
}

#[test]
fn service_test_fixture_payloads_remain_json_compatible() {
    let case = Case::new("case-1".into()).unwrap();
    let value: Value = serde_json::to_value(case).unwrap();
    assert!(value.is_object());
}
