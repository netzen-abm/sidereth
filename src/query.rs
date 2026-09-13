use crate::{
    AuthorizationDecision, AuthorizationResult, PersistenceError, ResourceRecord, ResourceRef,
    UnitOfWorkContext, UnitOfWorkFactory,
};

/// Provider-neutral request for a direct canonical resource read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceQueryRequest {
    pub resource_ref: ResourceRef,
    pub authorization: AuthorizationResult,
    pub requested_data_class: Option<String>,
    pub now_epoch_seconds: u64,
}

impl ResourceQueryRequest {
    pub fn new(
        resource_ref: ResourceRef,
        authorization: AuthorizationResult,
        requested_data_class: Option<String>,
        now_epoch_seconds: u64,
    ) -> Self {
        Self {
            resource_ref,
            authorization,
            requested_data_class,
            now_epoch_seconds,
        }
    }

    fn validate(&self) -> Result<(), QueryError> {
        if self.authorization.decision != AuthorizationDecision::Allow {
            return Err(QueryError::AuthorizationDenied);
        }
        if self.authorization.resource_ref != self.resource_ref {
            return Err(QueryError::AuthorizationMismatch);
        }
        if self.authorization.subject_ref.id.is_empty()
            || self.authorization.purpose.trim().is_empty()
        {
            return Err(QueryError::InvalidRequest);
        }
        if self.authorization.evaluated_at_epoch_seconds > self.now_epoch_seconds {
            return Err(QueryError::AuthorizationInvalid);
        }
        if self
            .authorization
            .expires_at_epoch_seconds
            .map(|expires_at| self.now_epoch_seconds >= expires_at)
            .unwrap_or(false)
        {
            return Err(QueryError::AuthorizationExpired);
        }
        if self
            .requested_data_class
            .as_ref()
            .map(|value| value.trim().is_empty())
            .unwrap_or(false)
        {
            return Err(QueryError::InvalidRequest);
        }
        if self.requested_data_class != self.authorization.data_class {
            return Err(QueryError::DataClassificationMismatch);
        }
        Ok(())
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
        let result = uow
            .execute(|context| {
                context
                    .read_resource(&resource_ref)
                    .map_err(|error| match error {
                        crate::UnitOfWorkError::Persistence(error) => {
                            QueryError::Persistence(error)
                        }
                        crate::UnitOfWorkError::InvalidOperation => QueryError::InvalidRequest,
                    })
            })
            .and_then(|record| record.ok_or(QueryError::NotFound));

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
    use crate::{AuthorizationConstraint, ResourceType, Revision, UnitOfWork, UnitOfWorkError};
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

    fn request(resource_ref: ResourceRef) -> ResourceQueryRequest {
        ResourceQueryRequest::new(
            resource_ref.clone(),
            AuthorizationResult {
                request_id: "request-1".into(),
                authorization_ref: ResourceRef::new(ResourceType::Other, "auth-1").unwrap(),
                subject_ref: ResourceRef::new(ResourceType::Case, "case-1").unwrap(),
                action: ResourceRef::new(ResourceType::Other, "resource.read").unwrap(),
                resource_ref,
                purpose: "direct resource retrieval".into(),
                jurisdiction_ref: None,
                data_class: Some("public".into()),
                decision: AuthorizationDecision::Allow,
                constraints: vec![AuthorizationConstraint {
                    key: "scope".into(),
                    value: "exact_resource".into(),
                }],
                policy_refs: vec![],
                evaluated_at_epoch_seconds: 1_000,
                expires_at_epoch_seconds: Some(2_000),
            },
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
        request.authorization.resource_ref = other_ref;
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
    fn data_classification_must_match_authorization() {
        let resource_ref = ResourceRef::new(ResourceType::Case, "case-1").unwrap();
        let mut request = request(resource_ref);
        request.requested_data_class = Some("restricted".into());
        let factory = FakeFactory::default();
        let mut query = UnitOfWorkResourceQuery::new(factory);
        assert_eq!(
            query.get(request),
            Err(QueryError::DataClassificationMismatch)
        );
    }

    #[test]
    fn missing_resource_is_distinguished_from_denial() {
        let resource_ref = ResourceRef::new(ResourceType::Case, "missing").unwrap();
        let factory = FakeFactory::default();
        let mut query = UnitOfWorkResourceQuery::new(factory);
        assert_eq!(query.get(request(resource_ref)), Err(QueryError::NotFound));
    }
}
