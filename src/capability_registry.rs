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

    pub fn validate(&self, entry: &CapabilityRegistryEntry) -> Result<(), CapabilityRegistryError> {
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
            if implementation.implementation_id.is_empty() || implementation.provider_id.is_empty()
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

    fn validate_audit_for_entry(
        &self,
        audit: &RegistryAuditRecord,
        entry: &CapabilityRegistryEntry,
        expected_change_type: &str,
    ) -> Result<(), CapabilityRegistryError> {
        if audit.change_id.is_empty()
            || audit.actor_ref.is_empty()
            || audit.authorization_ref.is_empty()
            || audit.change_type.is_empty()
        {
            return Err(CapabilityRegistryError::AuditContextRequired);
        }
        if audit.capability_id != entry.capability_id || audit.version != entry.version {
            return Err(CapabilityRegistryError::ConflictingMetadata);
        }
        if audit.change_type != expected_change_type {
            return Err(CapabilityRegistryError::ConflictingMetadata);
        }
        Ok(())
    }

    pub fn register(
        &mut self,
        entry: CapabilityRegistryEntry,
        audit: RegistryAuditRecord,
    ) -> Result<(), CapabilityRegistryError> {
        self.validate(&entry)?;
        self.validate_audit_for_entry(&audit, &entry, "register")?;
        if self
            .entries
            .contains_key(&(entry.capability_id.clone(), entry.version))
        {
            return Err(CapabilityRegistryError::DuplicateIdentity);
        }
        self.validate_dependencies(&entry)?;
        if dependency_graph_has_cycle(&self.entries, &entry) {
            return Err(CapabilityRegistryError::DependencyCycle);
        }
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
        match requirement {
            VersionRequirement::Exact(version) => {
                let entry = self
                    .entries
                    .get(&(capability_id.to_owned(), *version))
                    .ok_or(CapabilityRegistryError::UnsupportedVersion)?;
                if entry.lifecycle == CapabilityLifecycle::Retired {
                    return Err(CapabilityRegistryError::Retired);
                }
                Ok(entry)
            }
            VersionRequirement::CompatibleMajor(major) => {
                let mut matching = self
                    .entries
                    .iter()
                    .filter(|((id, version), _)| id == capability_id && version.major == *major);
                let has_match = matching.next().is_some();
                let active = self
                    .entries
                    .iter()
                    .filter(|((id, version), entry)| {
                        id == capability_id
                            && version.major == *major
                            && entry.lifecycle != CapabilityLifecycle::Retired
                    })
                    .max_by_key(|((_, version), _)| *version)
                    .map(|(_, entry)| entry);
                match active {
                    Some(entry) => Ok(entry),
                    None if has_match => Err(CapabilityRegistryError::Retired),
                    None => Err(CapabilityRegistryError::UnsupportedVersion),
                }
            }
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
        mut audit: RegistryAuditRecord,
    ) -> Result<(), CapabilityRegistryError> {
        let entry = self
            .entries
            .get_mut(&(capability_id.to_owned(), version))
            .ok_or(CapabilityRegistryError::NotFound)?;
        if audit.capability_id != entry.capability_id || audit.version != entry.version {
            return Err(CapabilityRegistryError::ConflictingMetadata);
        }
        if audit.change_id.is_empty()
            || audit.actor_ref.is_empty()
            || audit.authorization_ref.is_empty()
            || audit.change_type.is_empty()
        {
            return Err(CapabilityRegistryError::AuditContextRequired);
        }
        if audit.change_type != "promote" {
            return Err(CapabilityRegistryError::ConflictingMetadata);
        }
        let allowed = matches!(
            (entry.lifecycle, lifecycle),
            (CapabilityLifecycle::Proposed, CapabilityLifecycle::Designed)
                | (
                    CapabilityLifecycle::Designed,
                    CapabilityLifecycle::Contracted
                )
                | (
                    CapabilityLifecycle::Contracted,
                    CapabilityLifecycle::Implemented
                )
                | (
                    CapabilityLifecycle::Implemented,
                    CapabilityLifecycle::Tested
                )
                | (
                    CapabilityLifecycle::Tested,
                    CapabilityLifecycle::SecurityReviewed
                )
                | (
                    CapabilityLifecycle::SecurityReviewed,
                    CapabilityLifecycle::OperationallyVerified
                )
                | (
                    CapabilityLifecycle::OperationallyVerified,
                    CapabilityLifecycle::Active
                )
                | (CapabilityLifecycle::Active, CapabilityLifecycle::Deprecated)
                | (
                    CapabilityLifecycle::Deprecated,
                    CapabilityLifecycle::Retired
                )
        );
        if !allowed {
            return Err(CapabilityRegistryError::InvalidLifecyclePromotion);
        }
        if lifecycle == CapabilityLifecycle::Active && entry.implementations.is_empty() {
            return Err(CapabilityRegistryError::InvalidLifecyclePromotion);
        }
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

fn dependency_graph_has_cycle(
    entries: &BTreeMap<(Id, CapabilityVersion), CapabilityRegistryEntry>,
    candidate: &CapabilityRegistryEntry,
) -> bool {
    fn visit(
        id: &Id,
        version: CapabilityVersion,
        entries: &BTreeMap<(Id, CapabilityVersion), CapabilityRegistryEntry>,
        candidate: &CapabilityRegistryEntry,
        visiting: &mut BTreeSet<(Id, CapabilityVersion)>,
        visited: &mut BTreeSet<(Id, CapabilityVersion)>,
    ) -> bool {
        let key = (id.clone(), version);
        if !visiting.insert(key.clone()) {
            return true;
        }
        if visited.contains(&key) {
            visiting.remove(&key);
            return false;
        }
        let entry = if id == &candidate.capability_id && version == candidate.version {
            candidate
        } else if let Some(existing) = entries.get(&key) {
            existing
        } else {
            visiting.remove(&key);
            visited.insert(key);
            return false;
        };
        for dependency in &entry.dependencies {
            if dependency.optional {
                continue;
            }
            let versions: Vec<_> = entries
                .keys()
                .filter(|(dep_id, dep_version)| {
                    dep_id == &dependency.capability_id
                        && version_matches(*dep_version, &dependency.requirement)
                })
                .map(|(_, dep_version)| *dep_version)
                .chain(std::iter::once(candidate.version).filter(|dep_version| {
                    candidate.capability_id == dependency.capability_id
                        && version_matches(*dep_version, &dependency.requirement)
                }))
                .collect();
            for dep_version in versions {
                if visit(
                    &dependency.capability_id,
                    dep_version,
                    entries,
                    candidate,
                    visiting,
                    visited,
                ) {
                    return true;
                }
            }
        }
        visiting.remove(&key);
        visited.insert(key);
        false
    }

    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    visit(
        &candidate.capability_id,
        candidate.version,
        entries,
        candidate,
        &mut visiting,
        &mut visited,
    )
}
#[cfg(test)]
#[path = "capability_registry_tests.rs"]
mod tests;
