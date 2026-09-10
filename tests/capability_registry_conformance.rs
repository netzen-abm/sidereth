use sidereth_core::{
    CapabilityDataClass, CapabilityDependency, CapabilityImplementation, CapabilityLifecycle,
    CapabilityRegistryEntry, CapabilityRegistryError, CapabilityRegistry, CapabilityRiskClass,
    CapabilityVersion, ExecutionMode, InMemoryCapabilityRegistry, RegistryAuditRecord,
    RegistryCriteria, ResourceRef, ResourceType, VersionRequirement,
};

fn resource(id: &str) -> ResourceRef {
    ResourceRef::new(ResourceType::Other, id).unwrap()
}

fn entry(id: &str, version: CapabilityVersion) -> CapabilityRegistryEntry {
    CapabilityRegistryEntry {
        capability_id: id.into(),
        version,
        name: id.into(),
        purpose: "conformance fixture".into(),
        lifecycle: CapabilityLifecycle::Proposed,
        risk_class: CapabilityRiskClass::ReadOnly,
        data_classes: [CapabilityDataClass::Public].into_iter().collect(),
        jurisdiction_scope: vec!["IN".into()],
        source_requirements: vec![],
        approval_required: false,
        contract_ref: resource("contract"),
        input_schema_ref: None,
        output_schema_ref: None,
        execution_modes: [ExecutionMode::Sync].into_iter().collect(),
        dependencies: vec![],
        implementations: vec![CapabilityImplementation {
            implementation_id: format!("impl-{id}"),
            provider_id: format!("provider-{id}"),
            implementation_version: "1".into(),
            adapter_refs: vec![],
        }],
        observability_refs: vec![],
    }
}

fn audit(id: &str, version: CapabilityVersion) -> RegistryAuditRecord {
    RegistryAuditRecord {
        change_id: format!("change-{id}"),
        capability_id: id.into(),
        version,
        change_type: "register".into(),
        actor_ref: "actor".into(),
        authorization_ref: "authorization".into(),
        previous_lifecycle: None,
        new_lifecycle: None,
    }
}

fn register(registry: &mut InMemoryCapabilityRegistry, id: &str, version: CapabilityVersion) {
    registry.register(entry(id, version), audit(id, version)).unwrap();
}

#[test]
fn audit_context_must_match_registered_identity() {
    let v = CapabilityVersion::new(1, 0, 0);
    let mut registry = InMemoryCapabilityRegistry::new();
    let mut record = audit("different", v);
    record.capability_id = "different".into();
    assert_eq!(
        registry.register(entry("actual", v), record),
        Err(CapabilityRegistryError::AuditContextRequired)
    );
}

#[test]
fn audit_context_must_be_nonempty() {
    let v = CapabilityVersion::new(1, 0, 0);
    let mut registry = InMemoryCapabilityRegistry::new();
    let mut record = audit("x", v);
    record.change_id.clear();
    assert_eq!(
        registry.register(entry("x", v), record),
        Err(CapabilityRegistryError::AuditContextRequired)
    );
}

#[test]
fn promotion_audit_context_must_match_target() {
    let v = CapabilityVersion::new(1, 0, 0);
    let mut registry = InMemoryCapabilityRegistry::new();
    register(&mut registry, "x", v);
    let mut record = audit("other", v);
    record.capability_id = "other".into();
    assert_eq!(
        registry.promote("x", v, CapabilityLifecycle::Designed, record),
        Err(CapabilityRegistryError::AuditContextRequired)
    );
}

#[test]
fn multi_node_dependency_cycle_is_rejected() {
    let v = CapabilityVersion::new(1, 0, 0);
    let mut registry = InMemoryCapabilityRegistry::new();
    let mut a = entry("a", v);
    a.dependencies.push(CapabilityDependency {
        capability_id: "b".into(),
        requirement: VersionRequirement::Exact(v),
        optional: false,
    });
    let mut b = entry("b", v);
    b.dependencies.push(CapabilityDependency {
        capability_id: "a".into(),
        requirement: VersionRequirement::Exact(v),
        optional: false,
    });
    register(&mut registry, "b", v);
    assert_eq!(
        registry.register(a, audit("a", v)),
        Err(CapabilityRegistryError::DependencyCycle)
    );
    assert!(registry.get("b", v).is_ok());
}

#[test]
fn required_dependency_registration_is_not_allowed_to_create_an_existing_cycle() {
    let v = CapabilityVersion::new(1, 0, 0);
    let mut registry = InMemoryCapabilityRegistry::new();
    register(&mut registry, "base", v);
    let mut child = entry("child", v);
    child.dependencies.push(CapabilityDependency {
        capability_id: "base".into(),
        requirement: VersionRequirement::Exact(v),
        optional: false,
    });
    registry.register(child, audit("child", v)).unwrap();
    assert!(registry.get("child", v).is_ok());
}

#[test]
fn compatible_resolution_returns_retired_when_only_compatible_version_is_retired() {
    let v = CapabilityVersion::new(1, 0, 0);
    let mut registry = InMemoryCapabilityRegistry::new();
    let mut retired = entry("x", v);
    retired.lifecycle = CapabilityLifecycle::Retired;
    registry.register(retired, audit("x", v)).unwrap();
    assert_eq!(
        registry.resolve("x", &VersionRequirement::CompatibleMajor(1)),
        Err(CapabilityRegistryError::Retired)
    );
}

#[test]
fn discovery_does_not_imply_execution_permission() {
    let v = CapabilityVersion::new(1, 0, 0);
    let mut registry = InMemoryCapabilityRegistry::new();
    register(&mut registry, "x", v);
    let found = registry.discover(&RegistryCriteria::default());
    assert_eq!(found.len(), 1);
    assert!(!found[0].lifecycle.selectable_for_execution());
}

#[test]
fn high_impact_registration_still_requires_explicit_approval_metadata() {
    let v = CapabilityVersion::new(1, 0, 0);
    let mut e = entry("high", v);
    e.risk_class = CapabilityRiskClass::HighImpact;
    assert_eq!(
        InMemoryCapabilityRegistry::new().validate(&e),
        Err(CapabilityRegistryError::HighImpactApprovalRequired)
    );
}

#[test]
fn provider_replacement_does_not_change_capability_identity() {
    let v = CapabilityVersion::new(1, 0, 0);
    let mut e = entry("x", v);
    e.implementations[0].provider_id = "provider-b".into();
    let mut registry = InMemoryCapabilityRegistry::new();
    registry.register(e, audit("x", v)).unwrap();
    assert_eq!(registry.get("x", v).unwrap().capability_id, "x");
    assert_eq!(registry.get("x", v).unwrap().version, v);
}

#[test]
fn optional_missing_dependency_does_not_block_registration() {
    let v = CapabilityVersion::new(1, 0, 0);
    let mut e = entry("x", v);
    e.dependencies.push(CapabilityDependency {
        capability_id: "optional".into(),
        requirement: VersionRequirement::Exact(v),
        optional: true,
    });
    let mut registry = InMemoryCapabilityRegistry::new();
    registry.register(e, audit("x", v)).unwrap();
}

#[test]
fn failed_registration_has_no_side_effect_or_audit_record() {
    let v = CapabilityVersion::new(1, 0, 0);
    let mut registry = InMemoryCapabilityRegistry::new();
    let mut e = entry("x", v);
    e.name.clear();
    assert!(registry.register(e, audit("x", v)).is_err());
    assert!(registry.get("x", v).is_err());
    assert!(registry.audit_records().is_empty());
}
