use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::error::{PersistenceError, UnitOfWorkError};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResourceWriteMode {
    Insert,
    Upsert,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceWrite {
    pub resource_ref: ResourceRef,
    pub schema_version: u16,
    pub payload: Value,
    pub mode: ResourceWriteMode,
    pub expected_revision: Option<Revision>,
}

impl ResourceWrite {
    pub fn new(
        resource_ref: ResourceRef,
        schema_version: u16,
        payload: Value,
        mode: ResourceWriteMode,
    ) -> Result<Self, UnitOfWorkError> {
        if schema_version == 0 {
            return Err(UnitOfWorkError::InvalidOperation);
        }
        Ok(Self {
            resource_ref,
            schema_version,
            payload,
            mode,
            expected_revision: None,
        })
    }

    pub fn with_expected_revision(mut self, revision: Revision) -> Self {
        self.expected_revision = Some(revision);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceRecord {
    pub resource_ref: ResourceRef,
    pub schema_version: u16,
    pub revision: Revision,
    pub payload: Value,
}

/// Semantic class of a newly authored ResourceLink.
///
/// `Legacy` exists only for backward-compatible decoding of the pre-class wire
/// representation. It MUST NOT be inferred to mean Strong, Forward, or External.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResourceLinkClass {
    Strong,
    Forward,
    External,
    Legacy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResourceLink {
    pub source_ref: ResourceRef,
    pub relation: String,
    pub target_ref: ResourceRef,
    /// `None` is the backward-compatible representation of a pre-class link.
    /// New links should use `new_with_class` and therefore carry `Some(...)`.
    #[serde(default)]
    pub class: Option<ResourceLinkClass>,
}

impl ResourceLink {
    /// Backward-compatible constructor for callers still producing the legacy
    /// class-less representation. It never assigns Strong semantics implicitly.
    pub fn new(
        source_ref: ResourceRef,
        relation: impl Into<String>,
        target_ref: ResourceRef,
    ) -> Result<Self, UnitOfWorkError> {
        Self::new_with_class(source_ref, relation, target_ref, ResourceLinkClass::Legacy)
    }

    pub fn new_with_class(
        source_ref: ResourceRef,
        relation: impl Into<String>,
        target_ref: ResourceRef,
        class: ResourceLinkClass,
    ) -> Result<Self, UnitOfWorkError> {
        let relation = relation.into();
        if relation.trim().is_empty() {
            return Err(UnitOfWorkError::InvalidOperation);
        }
        if class == ResourceLinkClass::Legacy {
            return Ok(Self {
                source_ref,
                relation,
                target_ref,
                class: None,
            });
        }
        Ok(Self {
            source_ref,
            relation,
            target_ref,
            class: Some(class),
        })
    }

    pub fn semantic_class(&self) -> Option<ResourceLinkClass> {
        self.class
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Revision {
    pub value: u64,
}

impl Revision {
    pub fn initial() -> Self {
        Self { value: 0 }
    }

    pub fn next(self) -> Result<Self, PersistenceError> {
        self.value
            .checked_add(1)
            .map(|value| Self { value })
            .ok_or(PersistenceError::Conflict)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Persisted<T> {
    pub schema_version: u16,
    pub revision: Revision,
    pub value: T,
}

impl<T> Persisted<T> {
    pub fn new(schema_version: u16, value: T) -> Result<Self, PersistenceError> {
        if schema_version == 0 {
            return Err(PersistenceError::ValidationFailure);
        }
        Ok(Self {
            schema_version,
            revision: Revision::initial(),
            value,
        })
    }
}
