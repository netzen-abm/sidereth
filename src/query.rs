use crate::authorization::{AuthorizationEvaluator, AuthorizationRequest};
use crate::persistence::ResourceRecord;
use crate::{
    validate_authorization, AuthorizationResult, PersistenceError, ResourceRef, ResourceType,
    UnitOfWork, UnitOfWorkContext, UnitOfWorkFactory,
};

/// Provider-neutral request for a direct canonical resource read.
/// The caller supplies intent, not a pre-fabricated authorization decision or clock value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceQueryRequest {
    pub resource_ref: ResourceRef,
    pub authorization_request: AuthorizationRequest,
    pub requested_data_class: Option<String>,
}

impl ResourceQueryRequest {
    pub fn new(
        resource_ref: ResourceRef,
        authorization_request: AuthorizationRequest,
        requested_data_class: Option<String>,
    ) -> Self {
        Self {
            resource_ref,
            authorization_request,
            requested_data_class,
        }
    }

    fn validate_context(&self) -> Result<(), QueryError> {
        let request = &self.authorization_request;
        if request.resource_ref != self.resource_ref
            || request.data_class != self.requested_data_class
            || request.action.resource_type != ResourceType::Action
            || request.action.id != "resource.read"
        {
            return Err(QueryError::AuthorizationMismatch);
        }
        if self
            .requested_data_class
            .as_ref()
            .map(|value| value.trim().is_empty())
            .unwrap_or(false)
        {
            return Err(QueryError::InvalidRequest);
        }
        Ok(())
    }

    fn validate_result(
        &self,
        authorization: &AuthorizationResult,
        now_epoch_seconds: u64,
    ) -> Result<(), QueryError> {
        validate_authorization(
            &self.authorization_request,
            authorization,
            now_epoch_seconds,
        )
        .map_err(|error| match error {
            crate::AuthorizationValidationError::Denied => QueryError::AuthorizationDenied,
            crate::AuthorizationValidationError::Expired => QueryError::AuthorizationExpired,
            crate::AuthorizationValidationError::RequestResultMismatch => {
                QueryError::AuthorizationMismatch
            }
            crate::AuthorizationValidationError::NotYetEvaluated
            | crate::AuthorizationValidationError::InvalidRequest
            | crate::AuthorizationValidationError::ConstraintViolation => {
                QueryError::AuthorizationInvalid
            }
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryError {
    AuthorizationDenied,
    AuthorizationMismatch,
    AuthorizationInvalid,
    AuthorizationExpired,
    DataClassificationMismatch,
    InvalidRequest,
    NotFound,
    Persistence(PersistenceError),
}

/// Canonical provider-neutral direct read boundary.
/// Implementations return canonical ResourceRecord values and must not reinterpret
/// payloads into stronger legal, evidentiary, epistemic, or responsibility claims.
pub trait ResourceQuery {
    fn get(&mut self, request: ResourceQueryRequest) -> Result<ResourceRecord, QueryError>;
}

/// Query adapter backed by the existing transactional persistence boundary.
/// This deliberately reuses the existing Unit-of-Work context.
pub struct UnitOfWorkResourceQuery<'a, F: UnitOfWorkFactory, E: AuthorizationEvaluator> {
    factory: F,
    evaluator: &'a E,
}

impl<'a, F: UnitOfWorkFactory, E: AuthorizationEvaluator> UnitOfWorkResourceQuery<'a, F, E> {
    pub fn new(factory: F, evaluator: &'a E) -> Self {
        Self { factory, evaluator }
    }

    pub fn into_inner(self) -> F {
        self.factory
    }
}

impl<F: UnitOfWorkFactory, E: AuthorizationEvaluator> ResourceQuery
    for UnitOfWorkResourceQuery<'_, F, E>
{
    fn get(&mut self, request: ResourceQueryRequest) -> Result<ResourceRecord, QueryError> {
        request.validate_context()?;
        let authorization = self.evaluator.evaluate(&request.authorization_request);
        let now_epoch_seconds = crate::authorization_enforcement::current_epoch_seconds()
            .ok_or(QueryError::AuthorizationInvalid)?;
        request.validate_result(&authorization, now_epoch_seconds)?;

        let resource_ref = request.resource_ref;
        let mut uow = self.factory.begin().map_err(QueryError::Persistence)?;
        let result = match uow.execute(|context| context.read_resource(&resource_ref)) {
            Ok(Some(record)) => Ok(record),
            Ok(None) => Err(QueryError::NotFound),
            Err(crate::UnitOfWorkError::Persistence(error)) => Err(QueryError::Persistence(error)),
            Err(crate::UnitOfWorkError::InvalidOperation) => Err(QueryError::InvalidRequest),
        };

        match result {
            Ok(record) => {
                uow.commit().map_err(QueryError::Persistence)?;
                Ok(record)
            }
            Err(error) => {
                let _ = uow.rollback();
                Err(error)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        authorization::{AuthorizationConstraint, AuthorizationDecision},
        ResourceRef, Revision, UnitOfWorkError,
    };
    use serde_json::Value;

    #[derive(Default)]
    struct FakeContext {
        record: Option<ResourceRecord>,
    }

    impl UnitOfWorkContext for FakeContext {
        fn read_resource(
            &mut self,
            resource_ref: &ResourceRef,
        ) -> Result<Option<ResourceRecord>, UnitOfWorkError> {
            Ok(self
                .record
                .clone()
                .filter(|record| &record.resource_ref == resource_ref))
        }

        fn write_resource(&mut self, _write: crate::ResourceWrite) -> Result<(), UnitOfWorkError> {
            Ok(())
        }

        fn link_resources(&mut self, _link: crate::ResourceLink) -> Result<(), UnitOfWorkError> {
            Ok(())
        }
    }

    struct FakeUow {
        context: FakeContext,
        committed: bool,
    }

    impl UnitOfWork for FakeUow {
        type Context = FakeContext;

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

        fn rollback(self) -> Result<(), PersistenceError> {
            Ok(())
        }
    }

    #[derive(Default)]
    struct FakeFactory {
        context: FakeContext,
    }

    impl UnitOfWorkFactory for FakeFactory {
        type Uow = FakeUow;

        fn begin(&mut self) -> Result<Self::Uow, PersistenceError> {
            Ok(FakeUow {
                context: std::mem::take(&mut self.context),
                committed: false,
            })
        }
    }

    struct TestEvaluator {
        decision: AuthorizationDecision,
        expires_in_seconds: u64,
        expired: bool,
        mismatched_purpose: bool,
    }

    impl Default for TestEvaluator {
        fn default() -> Self {
            Self {
                decision: AuthorizationDecision::Allow,
                expires_in_seconds: 3_600,
                expired: false,
                mismatched_purpose: false,
            }
        }
    }

    impl AuthorizationEvaluator for TestEvaluator {
        fn evaluate(&self, request: &AuthorizationRequest) -> AuthorizationResult {
            let now = crate::authorization_enforcement::current_epoch_seconds().unwrap();
            let mut result = AuthorizationResult {
                request_id: request.request_id.clone(),
                authorization_ref: request.authorization_ref.clone(),
                subject_ref: request.subject_ref.clone(),
                action: request.action.clone(),
                resource_ref: request.resource_ref.clone(),
                purpose: request.purpose.clone(),
                jurisdiction_ref: request.jurisdiction_ref.clone(),
                data_class: request.data_class.clone(),
                decision: self.decision,
                constraints: vec![AuthorizationConstraint {
                    key: "scope".into(),
                    value: "exact_resource".into(),
                }],
                policy_refs: request.policy_refs.clone(),
                evaluated_at_epoch_seconds: now,
                expires_at_epoch_seconds: Some(if self.expired {
                    now.saturating_sub(1)
                } else {
                    now.saturating_add(self.expires_in_seconds)
                }),
            };
            if self.mismatched_purpose {
                result.purpose = "different purpose".into();
            }
            result
        }
    }

    fn authorization_request(resource_ref: &ResourceRef) -> AuthorizationRequest {
        let now = crate::authorization_enforcement::current_epoch_seconds().unwrap();
        AuthorizationRequest {
            request_id: "request-1".into(),
            authorization_ref: ResourceRef::new(ResourceType::Other, "auth-1").unwrap(),
            subject_ref: ResourceRef::new(ResourceType::Party, "party-1").unwrap(),
            action: ResourceRef::new(ResourceType::Action, "resource.read").unwrap(),
            resource_ref: resource_ref.clone(),
            purpose: "direct resource retrieval".into(),
            policy_refs: vec![ResourceRef::new(ResourceType::Other, "policy-1").unwrap()],
            jurisdiction_ref: None,
            data_class: Some("public".into()),
            requested_at_epoch_seconds: now,
            freshness_seconds: Some(1_000),
        }
    }

    fn request(resource_ref: ResourceRef) -> ResourceQueryRequest {
        ResourceQueryRequest::new(
            resource_ref.clone(),
            authorization_request(&resource_ref),
            Some("public".into()),
        )
    }

    fn query<'a>(
        factory: FakeFactory,
        evaluator: &'a TestEvaluator,
    ) -> UnitOfWorkResourceQuery<'a, FakeFactory, TestEvaluator> {
        UnitOfWorkResourceQuery::new(factory, evaluator)
    }

    #[test]
    fn exact_resource_is_read_through_existing_uow_boundary() {
        let resource_ref = ResourceRef::new(ResourceType::Case, "case-1").unwrap();
        let factory = FakeFactory {
            context: FakeContext {
                record: Some(ResourceRecord {
                    resource_ref: resource_ref.clone(),
                    schema_version: 1,
                    revision: Revision { value: 3 },
                    payload: Value::String("canonical".into()),
                }),
            },
        };
        let evaluator = TestEvaluator::default();
        let record = query(factory, &evaluator).get(request(resource_ref)).unwrap();
        assert_eq!(record.revision, Revision { value: 3 });
        assert_eq!(record.payload, Value::String("canonical".into()));
    }

    #[test]
    fn denied_authorization_is_rejected_before_persistence_read() {
        let resource_ref = ResourceRef::new(ResourceType::Case, "case-1").unwrap();
        let evaluator = TestEvaluator {
            decision: AuthorizationDecision::Deny,
            ..TestEvaluator::default()
        };
        assert_eq!(
            query(FakeFactory::default(), &evaluator).get(request(resource_ref)),
            Err(QueryError::AuthorizationDenied)
        );
    }

    #[test]
    fn authorization_must_bind_to_exact_resource() {
        let resource_ref = ResourceRef::new(ResourceType::Case, "case-1").unwrap();
        let other_ref = ResourceRef::new(ResourceType::Case, "case-2").unwrap();
        let mut request = request(resource_ref);
        request.authorization_request.resource_ref = other_ref;
        let evaluator = TestEvaluator::default();
        assert_eq!(
            query(FakeFactory::default(), &evaluator).get(request),
            Err(QueryError::AuthorizationMismatch)
        );
    }

    #[test]
    fn evaluator_result_must_match_exact_request_context() {
        let resource_ref = ResourceRef::new(ResourceType::Case, "case-1").unwrap();
        let evaluator = TestEvaluator {
            mismatched_purpose: true,
            ..TestEvaluator::default()
        };
        assert_eq!(
            query(FakeFactory::default(), &evaluator).get(request(resource_ref)),
            Err(QueryError::AuthorizationMismatch)
        );
    }

    #[test]
    fn wrong_action_cannot_read_a_resource() {
        let resource_ref = ResourceRef::new(ResourceType::Case, "case-1").unwrap();
        let mut request = request(resource_ref);
        request.authorization_request.action =
            ResourceRef::new(ResourceType::Action, "case.delete").unwrap();
        let evaluator = TestEvaluator::default();
        assert_eq!(
            query(FakeFactory::default(), &evaluator).get(request),
            Err(QueryError::AuthorizationMismatch)
        );
    }

    #[test]
    fn expired_authorization_is_rejected_before_persistence_read() {
        let resource_ref = ResourceRef::new(ResourceType::Case, "case-1").unwrap();
        let evaluator = TestEvaluator {
            expired: true,
            ..TestEvaluator::default()
        };
        assert_eq!(
            query(FakeFactory::default(), &evaluator).get(request(resource_ref)),
            Err(QueryError::AuthorizationExpired)
        );
    }

    #[test]
    fn stale_request_is_rejected_before_persistence_read() {
        let resource_ref = ResourceRef::new(ResourceType::Case, "case-1").unwrap();
        let mut request = request(resource_ref);
        request.authorization_request.requested_at_epoch_seconds =
            crate::authorization_enforcement::current_epoch_seconds()
                .unwrap()
                .saturating_sub(1_000);
        request.authorization_request.freshness_seconds = Some(100);
        let evaluator = TestEvaluator::default();
        assert_eq!(
            query(FakeFactory::default(), &evaluator).get(request),
            Err(QueryError::AuthorizationExpired)
        );
    }

    #[test]
    fn data_classification_must_match_authorization_request() {
        let resource_ref = ResourceRef::new(ResourceType::Case, "case-1").unwrap();
        let mut request = request(resource_ref);
        request.requested_data_class = Some("restricted".into());
        let evaluator = TestEvaluator::default();
        assert_eq!(
            query(FakeFactory::default(), &evaluator).get(request),
            Err(QueryError::AuthorizationMismatch)
        );
    }

    #[test]
    fn missing_resource_is_distinguished_from_denial() {
        let resource_ref = ResourceRef::new(ResourceType::Case, "missing").unwrap();
        let evaluator = TestEvaluator::default();
        assert_eq!(
            query(FakeFactory::default(), &evaluator).get(request(resource_ref)),
            Err(QueryError::NotFound)
        );
    }
}
