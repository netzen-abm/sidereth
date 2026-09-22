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
            capability_lease_required: false,
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
        assert!(r
            .promote("x", v, ToolLifecycle::Contracted, audit("x", v, "promote"))
            .is_err());
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
