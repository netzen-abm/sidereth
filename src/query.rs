use crate::authorization::AuthorizationRequest;
use crate::persistence::ResourceRecord;
use crate::{
    validate_authorization, AuthorizationResult, PersistenceError, ResourceRef, UnitOfWork,
    UnitOfWorkContext, UnitOfWorkFactory,
};

/// Provider-neutral request for a direct canonical resource read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceQueryRequest {
    pub resource_ref: ResourceRef,
    pub authorization_request: AuthorizationRequest,
    pub authorization: AuthorizationResult,
    pub requested_data_class: Option<String>,
    pub now_epoch_seconds: u64,
}

impl ResourceQueryRequest {
    pub fn new(
        resource_ref: ResourceRef,
        authorization_request: AuthorizationRequest,
        authorization: AuthorizationResult,
        requested_data_class: Option<String>,
        now_epoch_seconds: u64,
    ) -> Self {
        Self {
            resource_ref,
            authorization_request,
            authorization,
            requested_data_class,
            now_epoch_seconds,
        }
    }

    fn validate(&self) -> Result<(), QueryError> {
        let request = &self.authorization_request;
        if request.resource_ref != self.resource_ref
            || request.data_class != self.requested_data_class
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

        validate_authorization(request, &self.authorization, self.now_epoch_seconds).map_err(
            |error| match error {
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
            },
        )
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
///
/// Implementations return canonical `ResourceRecord` values and must not
/// reinterpret payloads into stronger legal, evidentiary, epistemic, or
/// responsibility claims.
pub trait ResourceQuery {
    fn get(&mut self, request: ResourceQueryRequest) -> Result<ResourceRecord, QueryError>;
}

/// Query adapter backed by the existing transactional persistence boundary.
///
/// This intentionally does not introduce another repository or persistence
/// abstraction. The same provider-neutral UnitOfWorkContext used by mutation
/// workflows remains the source of canonical resource reads.
pub struct UnitOfWorkResourceQuery<F: UnitOfWorkFactory> {
    factory: F,
}

impl<F: UnitOfWorkFactory> UnitOfWorkResourceQuery<F> {
    pub fn new(factory: F) -> Self {
        Self { factory }
    }

    pub fn into_inner(self) -> F {
        self.factory
    }
}

impl<F: UnitOfWorkFactory> ResourceQuery for UnitOfWorkResourceQuery<F> {
    fn get(&mut self, request: ResourceQueryRequest) -> Result<ResourceRecord, QueryError> {
        request.validate()?;
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
        ResourceType, Revision, UnitOfWorkError,
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

    fn authorization_context(
        resource_ref: &ResourceRef,
    ) -> (AuthorizationRequest, AuthorizationResult) {
        let request = AuthorizationRequest {
            request_id: "request-1".into(),
            authorization_ref: ResourceRef::new(ResourceType::Other, "auth-1").unwrap(),
            subject_ref: ResourceRef::new(ResourceType::Party, "party-1").unwrap(),
            action: ResourceRef::new(ResourceType::Action, "resource.read").unwrap(),
            resource_ref: resource_ref.clone(),
            purpose: "direct resource retrieval".into(),
            policy_refs: vec![ResourceRef::new(ResourceType::Other, "policy-1").unwrap()],
            jurisdiction_ref: None,
            data_class: Some("public".into()),
            requested_at_epoch_seconds: 1_000,
            freshness_seconds: Some(1_000),
        };
        let result = AuthorizationResult {
            request_id: request.request_id.clone(),
            authorization_ref: request.authorization_ref.clone(),
            subject_ref: request.subject_ref.clone(),
            action: request.action.clone(),
            resource_ref: request.resource_ref.clone(),
            purpose: request.purpose.clone(),
            jurisdiction_ref: request.jurisdiction_ref.clone(),
            data_class: request.data_class.clone(),
            decision: AuthorizationDecision::Allow,
            constraints: vec![AuthorizationConstraint {
                key: "scope".into(),
                value: "exact_resource".into(),
            }],
            policy_refs: request.policy_refs.clone(),
            evaluated_at_epoch_seconds: 1_000,
            expires_at_epoch_seconds: Some(2_000),
        };
        (request, result)
    }

    fn request(resource_ref: ResourceRef) -> ResourceQueryRequest {
        let (authorization_request, authorization) = authorization_context(&resource_ref);
        ResourceQueryRequest::new(
            resource_ref,
            authorization_request,
            authorization,
            Some("public".into()),
            1_500,
        )
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
        let mut query = UnitOfWorkResourceQuery::new(factory);
        let record = query.get(request(resource_ref)).unwrap();
        assert_eq!(record.revision, Revision { value: 3 });
        assert_eq!(record.payload, Value::String("canonical".into()));
    }

    #[test]
    fn denied_authorization_is_rejected_before_persistence_read() {
        let resource_ref = ResourceRef::new(ResourceType::Case, "case-1").unwrap();
        let mut request = request(resource_ref);
        request.authorization.decision = AuthorizationDecision::Deny;
        let factory = FakeFactory::default();
        let mut query = UnitOfWorkResourceQuery::new(factory);
        assert_eq!(query.get(request), Err(QueryError::AuthorizationDenied));
    }

    #[test]
    fn authorization_must_bind_to_exact_resource() {
        let resource_ref = ResourceRef::new(ResourceType::Case, "case-1").unwrap();
        let other_ref = ResourceRef::new(ResourceType::Case, "case-2").unwrap();
        let mut request = request(resource_ref);
        request.authorization_request.resource_ref = other_ref;
        let factory = FakeFactory::default();
        let mut query = UnitOfWorkResourceQuery::new(factory);
        assert_eq!(query.get(request), Err(QueryError::AuthorizationMismatch));
    }

    #[test]
    fn authorization_result_must_match_exact_request_context() {
        let resource_ref = ResourceRef::new(ResourceType::Case, "case-1").unwrap();
        let mut request = request(resource_ref);
        request.authorization.purpose = "different purpose".into();
        let factory = FakeFactory::default();
        let mut query = UnitOfWorkResourceQuery::new(factory);
        assert_eq!(query.get(request), Err(QueryError::AuthorizationMismatch));
    }

    #[test]
    fn expired_authorization_is_rejected_before_persistence_read() {
        let resource_ref = ResourceRef::new(ResourceType::Case, "case-1").unwrap();
        let mut request = request(resource_ref);
        request.authorization.expires_at_epoch_seconds = Some(1_500);
        let factory = FakeFactory::default();
        let mut query = UnitOfWorkResourceQuery::new(factory);
        assert_eq!(query.get(request), Err(QueryError::AuthorizationExpired));
    }

    #[test]
    fn stale_request_is_rejected_before_persistence_read() {
        let resource_ref = ResourceRef::new(ResourceType::Case, "case-1").unwrap();
        let mut request = request(resource_ref);
        request.authorization_request.requested_at_epoch_seconds = 1_000;
        request.authorization_request.freshness_seconds = Some(100);
        let factory = FakeFactory::default();
        let mut query = UnitOfWorkResourceQuery::new(factory);
        assert_eq!(query.get(request), Err(QueryError::AuthorizationExpired));
    }

    #[test]
    fn data_classification_must_match_authorization_request() {
        let resource_ref = ResourceRef::new(ResourceType::Case, "case-1").unwrap();
        let mut request = request(resource_ref);
        request.requested_data_class = Some("restricted".into());
        let factory = FakeFactory::default();
        let mut query = UnitOfWorkResourceQuery::new(factory);
        assert_eq!(query.get(request), Err(QueryError::AuthorizationMismatch));
    }

    #[test]
    fn missing_resource_is_distinguished_from_denial() {
        let resource_ref = ResourceRef::new(ResourceType::Case, "missing").unwrap();
        let factory = FakeFactory::default();
        let mut query = UnitOfWorkResourceQuery::new(factory);
        assert_eq!(query.get(request(resource_ref)), Err(QueryError::NotFound));
    }
}
