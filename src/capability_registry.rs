//! Provider-neutral Capability Registry reference implementation.
//!
//! The registry owns discovery, identity, version resolution and lifecycle
//! metadata. It does not grant authorization, approval, trust or execution.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{Id, ResourceRef};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct CapabilityVersion {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

impl CapabilityVersion {
    pub const fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CapabilityLifecycle {
    Proposed,
    Designed,
    Contracted,
    Implemented,
    Tested,
    SecurityReviewed,
    OperationallyVerified,
    Active,
    Deprecated,
    Retired,
}

impl CapabilityLifecycle {
    pub fn selectable_for_execution(self) -> bool {
        matches!(self, Self::Active)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CapabilityRiskClass {
    ReadOnly,
    UserData,
    Mutating,
    HighImpact,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum CapabilityDataClass {
    Public,
    Internal,
    Confidential,
    Restricted,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExecutionMode {
    Sync,
    Async,
    Offline,
    Resumable,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VersionRequirement {
    Exact(CapabilityVersion),
    CompatibleMajor(u64),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityImplementation {
    pub implementation_id: Id,
    pub provider_id: Id,
    pub implementation_version: String,
    pub adapter_refs: Vec<ResourceRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityDependency {
    pub capability_id: Id,
    pub requirement: VersionRequirement,
    pub optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityRegistryEntry {
    pub capability_id: Id,
    pub version: CapabilityVersion,
    pub name: String,
    pub purpose: String,
    pub lifecycle: CapabilityLifecycle,
    pub risk_class: CapabilityRiskClass,
    pub data_classes: BTreeSet<CapabilityDataClass>,
    pub jurisdiction_scope: Vec<String>,
    pub source_requirements: Vec<String>,
    pub approval_required: bool,
    pub contract_ref: ResourceRef,
    pub input_schema_ref: Option<ResourceRef>,
    pub output_schema_ref: Option<ResourceRef>,
    pub execution_modes: BTreeSet<ExecutionMode>,
    pub dependencies: Vec<CapabilityDependency>,
    pub implementations: Vec<CapabilityImplementation>,
    pub observability_refs: Vec<ResourceRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegistryAuditRecord {
    pub change_id: Id,
    pub capability_id: Id,
    pub version: CapabilityVersion,
    pub change_type: String,
    pub actor_ref: Id,
    pub authorization_ref: Id,
    pub previous_lifecycle: Option<CapabilityLifecycle>,
    pub new_lifecycle: Option<CapabilityLifecycle>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegistryCriteria {
    pub capability_id: Option<Id>,
    pub version: Option<CapabilityVersion>,
    pub risk_class: Option<CapabilityRiskClass>,
    pub data_class: Option<CapabilityDataClass>,
    pub jurisdiction: Option<String>,
    pub execution_mode: Option<ExecutionMode>,
    pub lifecycle: Option<CapabilityLifecycle>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapabilityRegistryError {
    EmptyId,
    EmptyName,
    EmptyPurpose,
    InvalidContractReference,
    DuplicateIdentity,
    NotFound,
    UnsupportedVersion,
    Retired,
    InvalidLifecyclePromotion,
    HighImpactApprovalRequired,
    MissingRequiredDependency,
    IncompatibleDependency,
    DependencyCycle,
    DuplicateImplementation,
    ConflictingMetadata,
    AuditContextRequired,
}

impl std::fmt::Display for CapabilityRegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for CapabilityRegistryError {}

#[derive(Debug, Clone, Default)]
pub struct InMemoryCapabilityRegistry {
    entries: BTreeMap<(Id, CapabilityVersion), CapabilityRegistryEntry>,
    audit: Vec<RegistryAuditRecord>,
}

impl InMemoryCapabilityRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn validate(
        &self,
        entry: &CapabilityRegistryEntry,
    ) -> Result<(), CapabilityRegistryError> {
        if entry.capability_id.is_empty() {
            return Err(CapabilityRegistryError::EmptyId);
        }
        if entry.name.is_empty() {
            return Err(CapabilityRegistryError::EmptyName);
        }
        if entry.purpose.is_empty() {
            return Err(CapabilityRegistryError::EmptyPurpose);
        }
        if entry.contract_ref.id.is_empty() {
            return Err(CapabilityRegistryError::InvalidContractReference);
        }
        if entry.risk_class == CapabilityRiskClass::HighImpact && !entry.approval_required {
            return Err(CapabilityRegistryError::HighImpactApprovalRequired);
        }

        let mut implementation_ids = BTreeSet::new();
        for implementation in &entry.implementations {
            if implementation.implementation_id.is_empty()
                || implementation.provider_id.is_empty()
            {
                return Err(CapabilityRegistryError::InvalidContractReference);
            }
            if !implementation_ids.insert(implementation.implementation_id.clone()) {
                return Err(CapabilityRegistryError::DuplicateImplementation);
            }
        }

        if entry.dependencies.iter().any(|dependency| {
            dependency.capability_id == entry.capability_id
                && dependency.requirement == VersionRequirement::Exact(entry.version)
        }) {
            return Err(CapabilityRegistryError::DependencyCycle);
        }

        Ok(())
    }

    pub fn register(
        &mut self,
        entry: CapabilityRegistryEntry,
        audit: RegistryAuditRecord,
    ) -> Result<(), CapabilityRegistryError> {
        self.validate(&entry)?;
        if self
            .entries
            .contains_key(&(entry.capability_id.clone(), entry.version))
        {
            return Err(CapabilityRegistryError::DuplicateIdentity);
        }
        self.validate_dependencies(&entry)?;
        self.entries
            .insert((entry.capability_id.clone(), entry.version), entry);
        self.audit.push(audit);
        Ok(())
    }

    pub fn get(
        &self,
        capability_id: &str,
        version: CapabilityVersion,
    ) -> Result<&CapabilityRegistryEntry, CapabilityRegistryError> {
        self.entries
            .get(&(capability_id.to_owned(), version))
            .ok_or(CapabilityRegistryError::NotFound)
    }

    pub fn resolve(
        &self,
        capability_id: &str,
        requirement: &VersionRequirement,
    ) -> Result<&CapabilityRegistryEntry, CapabilityRegistryError> {
        let candidates = self.entries.iter().filter(|((id, version), entry)| {
            (id == capability_id
                && matches!(requirement, VersionRequirement::Exact(v) if *version == *v))
                || (id == capability_id
                    && matches!(
                        requirement,
                        VersionRequirement::CompatibleMajor(m)
                            if version.major == *m && entry.lifecycle != CapabilityLifecycle::Retired
                    ))
        });
        let result = candidates
            .max_by_key(|((_, version), _)| *version)
            .map(|(_, entry)| entry);
        match result {
            Some(entry) if entry.lifecycle == CapabilityLifecycle::Retired => {
                Err(CapabilityRegistryError::Retired)
            }
            Some(entry) => Ok(entry),
            None => Err(CapabilityRegistryError::UnsupportedVersion),
        }
    }

    pub fn discover(&self, criteria: &RegistryCriteria) -> Vec<&CapabilityRegistryEntry> {
        let mut result: Vec<_> = self
            .entries
            .values()
            .filter(|entry| {
                criteria
                    .capability_id
                    .as_deref()
                    .is_none_or(|id| entry.capability_id == id)
                    && criteria.version.is_none_or(|v| entry.version == v)
                    && criteria.risk_class.is_none_or(|v| entry.risk_class == v)
                    && criteria
                        .data_class
                        .is_none_or(|v| entry.data_classes.contains(&v))
                    && criteria
                        .jurisdiction
                        .as_ref()
                        .is_none_or(|v| entry.jurisdiction_scope.contains(v))
                    && criteria
                        .execution_mode
                        .is_none_or(|v| entry.execution_modes.contains(&v))
                    && criteria.lifecycle.is_none_or(|v| entry.lifecycle == v)
            })
            .collect();
        result.sort_by(|a, b| {
            a.capability_id
                .cmp(&b.capability_id)
                .then(b.version.cmp(&a.version))
        });
        result
    }

    pub fn promote(
        &mut self,
        capability_id: &str,
        version: CapabilityVersion,
        lifecycle: CapabilityLifecycle,
        audit: RegistryAuditRecord,
    ) -> Result<(), CapabilityRegistryError> {
        let entry = self
            .entries
            .get_mut(&(capability_id.to_owned(), version))
            .ok_or(CapabilityRegistryError::NotFound)?;
        let allowed = matches!(
            (entry.lifecycle, lifecycle),
            (CapabilityLifecycle::Proposed, CapabilityLifecycle::Designed)
                | (CapabilityLifecycle::Designed, CapabilityLifecycle::Contracted)
                | (CapabilityLifecycle::Contracted, CapabilityLifecycle::Implemented)
                | (CapabilityLifecycle::Implemented, CapabilityLifecycle::Tested)
                | (CapabilityLifecycle::Tested, CapabilityLifecycle::SecurityReviewed)
                | (CapabilityLifecycle::SecurityReviewed, CapabilityLifecycle::OperationallyVerified)
                | (CapabilityLifecycle::OperationallyVerified, CapabilityLifecycle::Active)
                | (CapabilityLifecycle::Active, CapabilityLifecycle::Deprecated)
                | (CapabilityLifecycle::Deprecated, CapabilityLifecycle::Retired)
        );
        if !allowed {
            return Err(CapabilityRegistryError::InvalidLifecyclePromotion);
        }
        if lifecycle == CapabilityLifecycle::Active && entry.implementations.is_empty() {
            return Err(CapabilityRegistryError::InvalidLifecyclePromotion);
        }
        let mut audit = audit;
        audit.previous_lifecycle = Some(entry.lifecycle);
        audit.new_lifecycle = Some(lifecycle);
        entry.lifecycle = lifecycle;
        self.audit.push(audit);
        Ok(())
    }

    pub fn audit_records(&self) -> &[RegistryAuditRecord] {
        &self.audit
    }

    fn validate_dependencies(
        &self,
        entry: &CapabilityRegistryEntry,
    ) -> Result<(), CapabilityRegistryError> {
        for dependency in &entry.dependencies {
            if dependency.optional {
                continue;
            }
            let key_exists = self.entries.keys().any(|(id, version)| {
                id == &dependency.capability_id
                    && version_matches(*version, &dependency.requirement)
            });
            if !key_exists {
                return Err(CapabilityRegistryError::MissingRequiredDependency);
            }
        }
        Ok(())
    }
}

fn version_matches(version: CapabilityVersion, requirement: &VersionRequirement) -> bool {
    match requirement {
        VersionRequirement::Exact(v) => version == *v,
        VersionRequirement::CompatibleMajor(major) => version.major == *major,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ref_(id: &str) -> ResourceRef {
        ResourceRef::new(crate::ResourceType::Other, id).unwrap()
    }

    fn entry(id: &str, version: CapabilityVersion) -> CapabilityRegistryEntry {
        CapabilityRegistryEntry {
            capability_id: id.into(),
            version,
            name: id.into(),
            purpose: "test capability".into(),
            lifecycle: CapabilityLifecycle::Proposed,
            risk_class: CapabilityRiskClass::ReadOnly,
            data_classes: [CapabilityDataClass::Public].into_iter().collect(),
            jurisdiction_scope: vec!["IN".into()],
            source_requirements: vec![],
            approval_required: false,
            contract_ref: ref_("contract"),
            input_schema_ref: None,
            output_schema_ref: None,
            execution_modes: [ExecutionMode::Sync].into_iter().collect(),
            dependencies: vec![],
            implementations: vec![CapabilityImplementation {
                implementation_id: "impl-a".into(),
                provider_id: "provider-a".into(),
                implementation_version: "1".into(),
                adapter_refs: vec![],
            }],
            observability_refs: vec![],
        }
    }

    fn audit(id: &str, v: CapabilityVersion) -> RegistryAuditRecord {
        RegistryAuditRecord {
            change_id: "change-1".into(),
            capability_id: id.into(),
            version: v,
            change_type: "register".into(),
            actor_ref: "actor".into(),
            authorization_ref: "auth".into(),
            previous_lifecycle: None,
            new_lifecycle: None,
        }
    }

    #[test]
    fn register_and_exact_get_work() {
        let v = CapabilityVersion::new(1, 0, 0);
        let mut r = InMemoryCapabilityRegistry::new();
        r.register(entry("x", v), audit("x", v)).unwrap();
        assert_eq!(r.get("x", v).unwrap().capability_id, "x");
    }

    #[test]
    fn duplicate_identity_is_rejected() {
        let v = CapabilityVersion::new(1, 0, 0);
        let mut r = InMemoryCapabilityRegistry::new();
        r.register(entry("x", v), audit("x", v)).unwrap();
        assert_eq!(
            r.register(entry("x", v), audit("x", v)),
            Err(CapabilityRegistryError::DuplicateIdentity)
        );
    }

    #[test]
    fn exact_resolution_does_not_upgrade_major() {
        let mut r = InMemoryCapabilityRegistry::new();
        let v2 = CapabilityVersion::new(2, 0, 0);
        r.register(entry("x", v2), audit("x", v2)).unwrap();
        assert_eq!(
            r.resolve(
                "x",
                &VersionRequirement::Exact(CapabilityVersion::new(1, 0, 0))
            ),
            Err(CapabilityRegistryError::UnsupportedVersion)
        );
    }

    #[test]
    fn compatible_major_selects_highest_version() {
        let mut r = InMemoryCapabilityRegistry::new();
        for v in [
            CapabilityVersion::new(1, 0, 0),
            CapabilityVersion::new(1, 2, 0),
        ] {
            r.register(entry("x", v), audit("x", v)).unwrap();
        }
        assert_eq!(
            r.resolve("x", &VersionRequirement::CompatibleMajor(1))
                .unwrap()
                .version,
            CapabilityVersion::new(1, 2, 0)
        );
    }

    #[test]
    fn high_impact_requires_approval() {
        let v = CapabilityVersion::new(1, 0, 0);
        let mut e = entry("x", v);
        e.risk_class = CapabilityRiskClass::HighImpact;
        let r = InMemoryCapabilityRegistry::new();
        assert_eq!(
            r.validate(&e),
            Err(CapabilityRegistryError::HighImpactApprovalRequired)
        );
    }

    #[test]
    fn retired_is_not_execution_selectable() {
        assert!(!CapabilityLifecycle::Retired.selectable_for_execution());
    }

    #[test]
    fn discovery_is_deterministic_and_filtered() {
        let v = CapabilityVersion::new(1, 0, 0);
        let mut r = InMemoryCapabilityRegistry::new();
        r.register(entry("b", v), audit("b", v)).unwrap();
        r.register(entry("a", v), audit("a", v)).unwrap();
        let found = r.discover(&RegistryCriteria {
            risk_class: Some(CapabilityRiskClass::ReadOnly),
            ..RegistryCriteria::default()
        });
        assert_eq!(
            found
                .iter()
                .map(|e| e.capability_id.as_str())
                .collect::<Vec<_>>(),
            vec!["a", "b"]
        );
    }

    #[test]
    fn lifecycle_promotion_requires_order() {
        let v = CapabilityVersion::new(1, 0, 0);
        let mut r = InMemoryCapabilityRegistry::new();
        r.register(entry("x", v), audit("x", v)).unwrap();
        assert!(r
            .promote("x", v, CapabilityLifecycle::Active, audit("x", v))
            .is_err());
    }

    #[test]
    fn required_dependency_must_exist() {
        let v = CapabilityVersion::new(1, 0, 0);
        let mut e = entry("x", v);
        e.dependencies.push(CapabilityDependency {
            capability_id: "missing".into(),
            requirement: VersionRequirement::Exact(v),
            optional: false,
        });
        let mut r = InMemoryCapabilityRegistry::new();
        assert_eq!(
            r.register(e, audit("x", v)),
            Err(CapabilityRegistryError::MissingRequiredDependency)
        );
    }
}
