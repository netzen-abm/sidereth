//! Provider-neutral Tool Registry reference implementation.
//!
//! The registry owns tool identity, contract metadata, discovery and lifecycle
//! state. It does not grant authorization, human approval, trust or execution.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{Id, ResourceRef};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ToolVersion {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

impl ToolVersion {
    pub const fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ToolLifecycle {
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

impl ToolLifecycle {
    pub fn selectable_for_execution(self) -> bool {
        matches!(self, Self::Active)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ToolRiskClass {
    ReadOnly,
    UserData,
    Mutating,
    HighImpact,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ToolDataClass {
    Public,
    Internal,
    Confidential,
    Restricted,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ToolExecutionMode {
    Sync,
    Async,
    Offline,
    Resumable,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ToolVersionRequirement {
    Exact(ToolVersion),
    CompatibleMajor(u64),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolImplementation {
    pub implementation_id: Id,
    pub provider_id: Id,
    pub implementation_version: String,
    pub adapter_refs: Vec<ResourceRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolDependency {
    pub tool_id: Id,
    pub requirement: ToolVersionRequirement,
    pub optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolRegistryEntry {
    pub tool_id: Id,
    pub version: ToolVersion,
    pub name: String,
    pub purpose: String,
    pub lifecycle: ToolLifecycle,
    pub risk_class: ToolRiskClass,
    pub data_classes: BTreeSet<ToolDataClass>,
    pub capability_ref: ResourceRef,
    pub function_ref: Option<ResourceRef>,
    pub jurisdiction_scope: Vec<String>,
    pub permission_requirements: Vec<String>,
    pub approval_required: bool,
    pub input_schema_ref: Option<ResourceRef>,
    pub output_schema_ref: Option<ResourceRef>,
    pub execution_modes: BTreeSet<ToolExecutionMode>,
    pub dependencies: Vec<ToolDependency>,
    pub implementations: Vec<ToolImplementation>,
    pub provenance_requirements: Vec<String>,
    pub observability_refs: Vec<ResourceRef>,
    pub documentation_refs: Vec<ResourceRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolRegistryAuditRecord {
    pub change_id: Id,
    pub tool_id: Id,
    pub version: ToolVersion,
    pub change_type: String,
    pub actor_ref: Id,
    pub authorization_ref: Id,
    pub timestamp: String,
    pub previous_lifecycle: Option<ToolLifecycle>,
    pub new_lifecycle: Option<ToolLifecycle>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolRegistryCriteria {
    pub tool_id: Option<Id>,
    pub version: Option<ToolVersion>,
    pub capability_ref: Option<ResourceRef>,
    pub risk_class: Option<ToolRiskClass>,
    pub data_class: Option<ToolDataClass>,
    pub jurisdiction: Option<String>,
    pub execution_mode: Option<ToolExecutionMode>,
    pub lifecycle: Option<ToolLifecycle>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolRegistryError {
    EmptyId,
    EmptyName,
    EmptyPurpose,
    InvalidCapabilityReference,
    InvalidFunctionReference,
    InvalidSchemaReference,
    InvalidVersion,
    DuplicateIdentity,
    NotFound,
    UnsupportedVersion,
    Retired,
    InvalidLifecyclePromotion,
    HighImpactApprovalRequired,
    MissingRequiredDependency,
    DependencyCycle,
    DuplicateImplementation,
    ConflictingMetadata,
    AuditContextRequired,
    InvalidMetadata,
}

impl std::fmt::Display for ToolRegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ToolRegistryError {}

#[derive(Debug, Clone, Default)]
pub struct InMemoryToolRegistry {
    entries: BTreeMap<(Id, ToolVersion), ToolRegistryEntry>,
    audit: Vec<ToolRegistryAuditRecord>,
}

impl InMemoryToolRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn validate(&self, entry: &ToolRegistryEntry) -> Result<(), ToolRegistryError> {
        if entry.tool_id.trim().is_empty() {
            return Err(ToolRegistryError::EmptyId);
        }
        if entry.name.trim().is_empty() {
            return Err(ToolRegistryError::EmptyName);
        }
        if entry.purpose.trim().is_empty() {
            return Err(ToolRegistryError::EmptyPurpose);
        }
        if entry.capability_ref.id.trim().is_empty() {
            return Err(ToolRegistryError::InvalidCapabilityReference);
        }
        if let Some(function) = &entry.function_ref {
            if function.id.trim().is_empty() {
                return Err(ToolRegistryError::InvalidFunctionReference);
            }
        }
        if entry.version.major == 0 && entry.version.minor == 0 && entry.version.patch == 0 {
            return Err(ToolRegistryError::InvalidVersion);
        }
        if entry.risk_class == ToolRiskClass::HighImpact && !entry.approval_required {
            return Err(ToolRegistryError::HighImpactApprovalRequired);
        }
        for schema in [&entry.input_schema_ref, &entry.output_schema_ref] {
            if let Some(reference) = schema {
                if reference.id.trim().is_empty() {
                    return Err(ToolRegistryError::InvalidSchemaReference);
                }
            }
        }
        let mut ids = BTreeSet::new();
        for implementation in &entry.implementations {
            if implementation.implementation_id.trim().is_empty()
                || implementation.provider_id.trim().is_empty()
            {
                return Err(ToolRegistryError::InvalidMetadata);
            }
            if !ids.insert(&implementation.implementation_id) {
                return Err(ToolRegistryError::DuplicateImplementation);
            }
        }
        if entry.dependencies.iter().any(|d| {
            d.tool_id == entry.tool_id
                && matches!(
                    d.requirement,
                    ToolVersionRequirement::Exact(v) if v == entry.version
                )
        }) {
            return Err(ToolRegistryError::DependencyCycle);
        }
        Ok(())
    }

    fn validate_audit(
        &self,
        audit: &ToolRegistryAuditRecord,
        entry: &ToolRegistryEntry,
        kind: &str,
    ) -> Result<(), ToolRegistryError> {
        if audit.change_id.trim().is_empty()
            || audit.actor_ref.trim().is_empty()
            || audit.authorization_ref.trim().is_empty()
            || audit.change_type.trim().is_empty()
            || audit.timestamp.trim().is_empty()
        {
            return Err(ToolRegistryError::AuditContextRequired);
        }
        if audit.tool_id != entry.tool_id
            || audit.version != entry.version
            || audit.change_type != kind
        {
            return Err(ToolRegistryError::ConflictingMetadata);
        }
        Ok(())
    }

    pub fn register(
        &mut self,
        entry: ToolRegistryEntry,
        audit: ToolRegistryAuditRecord,
    ) -> Result<(), ToolRegistryError> {
        self.validate(&entry)?;
        self.validate_audit(&audit, &entry, "register")?;
        if self
            .entries
            .contains_key(&(entry.tool_id.clone(), entry.version))
        {
            return Err(ToolRegistryError::DuplicateIdentity);
        }
        self.validate_dependencies(&entry)?;
        if self.dependency_cycle(&entry) {
            return Err(ToolRegistryError::DependencyCycle);
        }
        self.entries
            .insert((entry.tool_id.clone(), entry.version), entry);
        self.audit.push(audit);
        Ok(())
    }

    pub fn get(
        &self,
        tool_id: &str,
        version: ToolVersion,
    ) -> Result<&ToolRegistryEntry, ToolRegistryError> {
        self.entries
            .get(&(tool_id.to_owned(), version))
            .ok_or(ToolRegistryError::NotFound)
    }

    pub fn resolve(
        &self,
        tool_id: &str,
        requirement: &ToolVersionRequirement,
    ) -> Result<&ToolRegistryEntry, ToolRegistryError> {
        match requirement {
            ToolVersionRequirement::Exact(version) => {
                let entry = self
                    .entries
                    .get(&(tool_id.to_owned(), *version))
                    .ok_or(ToolRegistryError::UnsupportedVersion)?;
                if entry.lifecycle == ToolLifecycle::Retired {
                    return Err(ToolRegistryError::Retired);
                }
                Ok(entry)
            }
            ToolVersionRequirement::CompatibleMajor(major) => {
                let matching = self
                    .entries
                    .iter()
                    .filter(|((id, v), _)| id == tool_id && v.major == *major);
                let has_match = matching.clone().next().is_some();
                let selected = matching
                    .filter(|(_, entry)| entry.lifecycle != ToolLifecycle::Retired)
                    .max_by_key(|((_, v), _)| *v)
                    .map(|(_, entry)| entry);
                selected.ok_or(if has_match {
                    ToolRegistryError::Retired
                } else {
                    ToolRegistryError::UnsupportedVersion
                })
            }
        }
    }

    pub fn discover(&self, criteria: &ToolRegistryCriteria) -> Vec<&ToolRegistryEntry> {
        let mut result: Vec<_> = self
            .entries
            .values()
            .filter(|entry| {
                criteria
                    .tool_id
                    .as_deref()
                    .is_none_or(|id| entry.tool_id == id)
                    && criteria.version.is_none_or(|v| entry.version == v)
                    && criteria
                        .capability_ref
                        .as_ref()
                        .is_none_or(|r| entry.capability_ref == *r)
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
        result.sort_by(|a, b| a.tool_id.cmp(&b.tool_id).then(b.version.cmp(&a.version)));
        result
    }

    pub fn promote(
        &mut self,
        tool_id: &str,
        version: ToolVersion,
        lifecycle: ToolLifecycle,
        mut audit: ToolRegistryAuditRecord,
    ) -> Result<(), ToolRegistryError> {
        let entry = self
            .entries
            .get_mut(&(tool_id.to_owned(), version))
            .ok_or(ToolRegistryError::NotFound)?;
        if audit.tool_id != entry.tool_id || audit.version != entry.version {
            return Err(ToolRegistryError::ConflictingMetadata);
        }
        if audit.change_id.trim().is_empty()
            || audit.actor_ref.trim().is_empty()
            || audit.authorization_ref.trim().is_empty()
            || audit.change_type.trim().is_empty()
            || audit.timestamp.trim().is_empty()
        {
            return Err(ToolRegistryError::AuditContextRequired);
        }
        if audit.change_type != "promote" {
            return Err(ToolRegistryError::ConflictingMetadata);
        }
        let allowed = matches!(
            (entry.lifecycle, lifecycle),
            (ToolLifecycle::Proposed, ToolLifecycle::Designed)
                | (ToolLifecycle::Designed, ToolLifecycle::Contracted)
                | (ToolLifecycle::Contracted, ToolLifecycle::Implemented)
                | (ToolLifecycle::Implemented, ToolLifecycle::Tested)
                | (ToolLifecycle::Tested, ToolLifecycle::SecurityReviewed)
                | (
                    ToolLifecycle::SecurityReviewed,
                    ToolLifecycle::OperationallyVerified
                )
                | (ToolLifecycle::OperationallyVerified, ToolLifecycle::Active)
                | (ToolLifecycle::Active, ToolLifecycle::Deprecated)
                | (ToolLifecycle::Deprecated, ToolLifecycle::Retired)
        );
        if !allowed {
            return Err(ToolRegistryError::InvalidLifecyclePromotion);
        }
        if lifecycle == ToolLifecycle::Active && entry.implementations.is_empty() {
            return Err(ToolRegistryError::InvalidLifecyclePromotion);
        }
        audit.previous_lifecycle = Some(entry.lifecycle);
        audit.new_lifecycle = Some(lifecycle);
        entry.lifecycle = lifecycle;
        self.audit.push(audit);
        Ok(())
    }

    pub fn audit_records(&self) -> &[ToolRegistryAuditRecord] {
        &self.audit
    }

    fn validate_dependencies(&self, entry: &ToolRegistryEntry) -> Result<(), ToolRegistryError> {
        for dependency in &entry.dependencies {
            if dependency.optional {
                continue;
            }
            if !self.entries.keys().any(|(id, version)| {
                id == &dependency.tool_id && version_matches(*version, &dependency.requirement)
            }) {
                return Err(ToolRegistryError::MissingRequiredDependency);
            }
        }
        Ok(())
    }

    fn dependency_cycle(&self, candidate: &ToolRegistryEntry) -> bool {
        fn visit(
            id: &Id,
            version: ToolVersion,
            registry: &InMemoryToolRegistry,
            candidate: &ToolRegistryEntry,
            visiting: &mut BTreeSet<(Id, ToolVersion)>,
            visited: &mut BTreeSet<(Id, ToolVersion)>,
        ) -> bool {
            let key = (id.clone(), version);
            if !visiting.insert(key.clone()) {
                return true;
            }
            if visited.contains(&key) {
                visiting.remove(&key);
                return false;
            }
            let entry = if id == &candidate.tool_id && version == candidate.version {
                candidate
            } else if let Some(e) = registry.entries.get(&key) {
                e
            } else {
                visiting.remove(&key);
                visited.insert(key);
                return false;
            };
            for dep in &entry.dependencies {
                if dep.optional {
                    continue;
                }
                let versions: Vec<_> = registry
                    .entries
                    .keys()
                    .filter(|(dep_id, dep_version)| {
                        dep_id == &dep.tool_id
                            && version_matches(*dep_version, &dep.requirement)
                    })
                    .map(|(_, v)| *v)
                    .chain(std::iter::once(candidate.version).filter(|v| {
                        candidate.tool_id == dep.tool_id
                            && version_matches(*v, &dep.requirement)
                    }))
                    .collect();
                for v in versions {
                    if visit(&dep.tool_id, v, registry, candidate, visiting, visited) {
                        return true;
                    }
                }
            }
            visiting.remove(&key);
            visited.insert(key);
            false
        }
        visit(
            &candidate.tool_id,
            candidate.version,
            self,
            candidate,
            &mut BTreeSet::new(),
            &mut BTreeSet::new(),
        )
    }
}

fn version_matches(version: ToolVersion, requirement: &ToolVersionRequirement) -> bool {
    match requirement {
        ToolVersionRequirement::Exact(v) => version == *v,
        ToolVersionRequirement::CompatibleMajor(major) => version.major == *major,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ResourceRef, ResourceType};

    fn ref_(id: &str) -> ResourceRef {
        ResourceRef::new(ResourceType::Other, id).unwrap()
    }

    fn entry(id: &str, version: ToolVersion) -> ToolRegistryEntry {
        ToolRegistryEntry {
            tool_id: id.into(),
            version,
            name: id.into(),
            purpose: "test tool".into(),
            lifecycle: ToolLifecycle::Proposed,
            risk_class: ToolRiskClass::ReadOnly,
            data_classes: [ToolDataClass::Public].into_iter().collect(),
            capability_ref: ref_("capability"),
            function_ref: None,
            jurisdiction_scope: vec!["IN".into()],
            permission_requirements: vec![],
            approval_required: false,
            input_schema_ref: None,
            output_schema_ref: None,
            execution_modes: [ToolExecutionMode::Sync].into_iter().collect(),
            dependencies: vec![],
            implementations: vec![ToolImplementation {
                implementation_id: "impl-a".into(),
                provider_id: "provider-a".into(),
                implementation_version: "1".into(),
                adapter_refs: vec![],
            }],
            provenance_requirements: vec![],
            observability_refs: vec![],
            documentation_refs: vec![],
        }
    }

    fn audit(id: &str, v: ToolVersion, kind: &str) -> ToolRegistryAuditRecord {
        ToolRegistryAuditRecord {
            change_id: "change".into(),
            tool_id: id.into(),
            version: v,
            change_type: kind.into(),
            actor_ref: "actor".into(),
            authorization_ref: "auth".into(),
            timestamp: "2026-09-10T00:00:00Z".into(),
            previous_lifecycle: None,
            new_lifecycle: None,
        }
    }

    #[test]
    fn register_and_get_work() {
        let v = ToolVersion::new(1, 0, 0);
        let mut r = InMemoryToolRegistry::new();
        r.register(entry("x", v), audit("x", v, "register"))
            .unwrap();
        assert_eq!(r.get("x", v).unwrap().tool_id, "x");
    }

    #[test]
    fn duplicate_identity_rejected() {
        let v = ToolVersion::new(1, 0, 0);
        let mut r = InMemoryToolRegistry::new();
        r.register(entry("x", v), audit("x", v, "register"))
            .unwrap();
        assert_eq!(
            r.register(entry("x", v), audit("x", v, "register")),
            Err(ToolRegistryError::DuplicateIdentity)
        );
    }

    #[test]
    fn exact_resolution_never_crosses_major() {
        let v = ToolVersion::new(2, 0, 0);
        let mut r = InMemoryToolRegistry::new();
        r.register(entry("x", v), audit("x", v, "register"))
            .unwrap();
        assert_eq!(
            r.resolve(
                "x",
                &ToolVersionRequirement::Exact(ToolVersion::new(1, 0, 0))
            ),
            Err(ToolRegistryError::UnsupportedVersion)
        );
    }

    #[test]
    fn compatible_resolution_is_deterministic() {
        let mut r = InMemoryToolRegistry::new();
        for v in [ToolVersion::new(1, 0, 0), ToolVersion::new(1, 2, 0)] {
            r.register(entry("x", v), audit("x", v, "register"))
                .unwrap();
        }
        assert_eq!(
            r.resolve("x", &ToolVersionRequirement::CompatibleMajor(1))
                .unwrap()
                .version,
            ToolVersion::new(1, 2, 0)
        );
    }

    #[test]
    fn high_impact_requires_approval() {
        let v = ToolVersion::new(1, 0, 0);
        let mut e = entry("x", v);
        e.risk_class = ToolRiskClass::HighImpact;
        let r = InMemoryToolRegistry::new();
        assert_eq!(
            r.validate(&e),
            Err(ToolRegistryError::HighImpactApprovalRequired)
        );
    }

    #[test]
    fn retired_not_selectable() {
        assert!(!ToolLifecycle::Retired.selectable_for_execution());
    }

    #[test]
    fn discovery_is_filtered_and_deterministic() {
        let mut r = InMemoryToolRegistry::new();
        for v in [ToolVersion::new(1, 0, 0), ToolVersion::new(1, 1, 0)] {
            r.register(entry("x", v), audit("x", v, "register"))
                .unwrap();
        }
        let f = r.discover(&ToolRegistryCriteria {
            tool_id: Some("x".into()),
            ..Default::default()
        });
        assert_eq!(f.len(), 2);
        assert_eq!(f[0].version, ToolVersion::new(1, 1, 0));
    }

    #[test]
    fn lifecycle_order_is_enforced() {
        let v = ToolVersion::new(1, 0, 0);
        let mut r = InMemoryToolRegistry::new();
        r.register(entry("x", v), audit("x", v, "register"))
            .unwrap();
        assert!(
            r.promote("x", v, ToolLifecycle::Contracted, audit("x", v, "promote"))
                .is_err()
        );
    }

    #[test]
    fn required_dependency_must_exist() {
        let v = ToolVersion::new(1, 0, 0);
        let mut e = entry("x", v);
        e.dependencies.push(ToolDependency {
            tool_id: "missing".into(),
            requirement: ToolVersionRequirement::Exact(v),
            optional: false,
        });
        let mut r = InMemoryToolRegistry::new();
        assert_eq!(
            r.register(e, audit("x", v, "register")),
            Err(ToolRegistryError::MissingRequiredDependency)
        );
    }

    #[test]
    fn optional_dependency_does_not_block() {
        let v = ToolVersion::new(1, 0, 0);
        let mut e = entry("x", v);
        e.dependencies.push(ToolDependency {
            tool_id: "optional".into(),
            requirement: ToolVersionRequirement::Exact(v),
            optional: true,
        });
        let mut r = InMemoryToolRegistry::new();
        assert_eq!(r.register(e, audit("x", v, "register")), Ok(()));
    }

    #[test]
    fn multiple_implementations_are_allowed() {
        let v = ToolVersion::new(1, 0, 0);
        let mut e = entry("x", v);
        e.implementations.push(ToolImplementation {
            implementation_id: "impl-b".into(),
            provider_id: "provider-b".into(),
            implementation_version: "2".into(),
            adapter_refs: vec![],
        });
        let mut r = InMemoryToolRegistry::new();
        r.register(e, audit("x", v, "register")).unwrap();
        assert_eq!(r.get("x", v).unwrap().implementations.len(), 2);
    }

    #[test]
    fn audit_context_is_required() {
        let v = ToolVersion::new(1, 0, 0);
        let mut a = audit("x", v, "register");
        a.timestamp.clear();
        let mut r = InMemoryToolRegistry::new();
        assert_eq!(
            r.register(entry("x", v), a),
            Err(ToolRegistryError::AuditContextRequired)
        );
    }

    #[test]
    fn provider_swap_preserves_tool_identity() {
        let v = ToolVersion::new(1, 0, 0);
        let mut a = entry("x", v);
        a.implementations[0].provider_id = "a".into();
        let mut b = a.clone();
        b.implementations[0].provider_id = "b".into();
        assert_eq!(a.tool_id, b.tool_id);
        assert_eq!(a.version, b.version);
    }
}
