    use crate::capability_registry::*;

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

    fn promotion_audit(id: &str, v: CapabilityVersion) -> RegistryAuditRecord {
        RegistryAuditRecord {
            change_id: "change-promote".into(),
            capability_id: id.into(),
            version: v,
            change_type: "promote".into(),
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
        let mut r = InMemoryCapabilityRegistry::new();
        for v in [
            CapabilityVersion::new(1, 0, 0),
            CapabilityVersion::new(1, 1, 0),
        ] {
            r.register(entry("x", v), audit("x", v)).unwrap();
        }
        let found = r.discover(&RegistryCriteria {
            capability_id: Some("x".into()),
            ..RegistryCriteria::default()
        });
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].version, CapabilityVersion::new(1, 1, 0));
    }

    #[test]
    fn lifecycle_promotion_requires_order() {
        let v = CapabilityVersion::new(1, 0, 0);
        let mut r = InMemoryCapabilityRegistry::new();
        r.register(entry("x", v), audit("x", v)).unwrap();
        assert!(r
            .promote(
                "x",
                v,
                CapabilityLifecycle::Contracted,
                promotion_audit("x", v)
            )
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

    #[test]
    fn register_rejects_orphan_or_mismatched_audit_context() {
        let v = CapabilityVersion::new(1, 0, 0);
        let mut r = InMemoryCapabilityRegistry::new();
        let mut mismatched = audit("other", v);
        assert_eq!(
            r.register(entry("x", v), mismatched.clone()),
            Err(CapabilityRegistryError::ConflictingMetadata)
        );
        mismatched.capability_id = "x".into();
        mismatched.change_type.clear();
        assert_eq!(
            r.register(entry("x", v), mismatched),
            Err(CapabilityRegistryError::AuditContextRequired)
        );
    }

    #[test]
    fn promotion_audit_context_is_bound_to_entry() {
        let v = CapabilityVersion::new(1, 0, 0);
        let mut r = InMemoryCapabilityRegistry::new();
        r.register(entry("x", v), audit("x", v)).unwrap();
        let mut a = promotion_audit("other", v);
        assert_eq!(
            r.promote("x", v, CapabilityLifecycle::Designed, a.clone()),
            Err(CapabilityRegistryError::ConflictingMetadata)
        );
        a.capability_id = "x".into();
        a.change_type.clear();
        assert_eq!(
            r.promote("x", v, CapabilityLifecycle::Designed, a),
            Err(CapabilityRegistryError::AuditContextRequired)
        );
    }

    #[test]
    fn compatible_major_returns_retired_when_all_matching_versions_are_retired() {
        let v = CapabilityVersion::new(1, 0, 0);
        let mut e = entry("x", v);
        e.lifecycle = CapabilityLifecycle::Retired;
        let mut r = InMemoryCapabilityRegistry::new();
        r.register(e, audit("x", v)).unwrap();
        assert_eq!(
            r.resolve("x", &VersionRequirement::CompatibleMajor(1)),
            Err(CapabilityRegistryError::Retired)
        );
    }

    #[test]
    fn compatible_major_prefers_active_over_retired_version() {
        let v1 = CapabilityVersion::new(1, 0, 0);
        let v2 = CapabilityVersion::new(1, 1, 0);
        let mut r = InMemoryCapabilityRegistry::new();
        let mut retired = entry("x", v2);
        retired.lifecycle = CapabilityLifecycle::Retired;
        r.register(entry("x", v1), audit("x", v1)).unwrap();
        r.register(retired, audit("x", v2)).unwrap();
        assert_eq!(
            r.resolve("x", &VersionRequirement::CompatibleMajor(1))
                .unwrap()
                .version,
            v1
        );
    }

    #[test]
    fn multiple_providers_are_metadata_only_and_allowed() {
        let v = CapabilityVersion::new(1, 0, 0);
        let mut e = entry("x", v);
        e.implementations.push(CapabilityImplementation {
            implementation_id: "impl-b".into(),
            provider_id: "provider-b".into(),
            implementation_version: "2".into(),
            adapter_refs: vec![],
        });
        let mut r = InMemoryCapabilityRegistry::new();
        r.register(e, audit("x", v)).unwrap();
        assert_eq!(r.get("x", v).unwrap().implementations.len(), 2);
    }

    #[test]
    fn provider_replacement_does_not_change_capability_identity() {
        let v = CapabilityVersion::new(1, 0, 0);
        let mut first = entry("x", v);
        first.implementations[0].provider_id = "provider-a".into();
        let mut second = first.clone();
        second.implementations[0].provider_id = "provider-b".into();
        let r = InMemoryCapabilityRegistry::new();
        assert_eq!(first.capability_id, second.capability_id);
        assert_eq!(first.version, second.version);
        assert_eq!(r.validate(&second), Ok(()));
    }

    #[test]
    fn optional_dependency_does_not_block_registration() {
        let v = CapabilityVersion::new(1, 0, 0);
        let mut e = entry("x", v);
        e.dependencies.push(CapabilityDependency {
            capability_id: "optional".into(),
            requirement: VersionRequirement::Exact(v),
            optional: true,
        });
        let mut r = InMemoryCapabilityRegistry::new();
        assert_eq!(r.register(e, audit("x", v)), Ok(()));
    }

    #[test]
    fn registry_discovery_does_not_grant_execution_authority() {
        let v = CapabilityVersion::new(1, 0, 0);
        let mut r = InMemoryCapabilityRegistry::new();
        r.register(entry("x", v), audit("x", v)).unwrap();
        let found = r.discover(&RegistryCriteria::default());
        assert_eq!(found.len(), 1);
        assert!(!found[0].lifecycle.selectable_for_execution());
        assert!(!found[0].approval_required);
    }

    #[test]
    fn multi_node_dependency_registration_is_deterministic() {
        let v = CapabilityVersion::new(1, 0, 0);
        let b = entry("b", v);
        let mut a = entry("a", v);
        a.dependencies.push(CapabilityDependency {
            capability_id: "b".into(),
            requirement: VersionRequirement::Exact(v),
            optional: false,
        });
        let mut r = InMemoryCapabilityRegistry::new();
        r.register(b, audit("b", v)).unwrap();
        assert_eq!(r.register(a, audit("a", v)), Ok(()));
        assert_eq!(r.discover(&RegistryCriteria::default()).len(), 2);
    }
}