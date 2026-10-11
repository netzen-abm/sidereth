use sidereth_core::{
    AuthorizationDecision, AuthorizationRequest, AuthorizationResult, KnowledgeEdge,
    KnowledgeGraph, KnowledgeGraphAccessContext, KnowledgeGraphResourceAuthorization,
    KnowledgeLinkClass, KnowledgeNode, ResourceRef, ResourceType,
};
use std::collections::BTreeSet;

fn reference(t: ResourceType, id: &str) -> ResourceRef {
    ResourceRef::new(t, id).unwrap()
}

fn access_many(resources: &[ResourceRef], action_id: &str) -> KnowledgeGraphAccessContext {
    KnowledgeGraphAccessContext {
        authorizations: resources
            .iter()
            .map(|resource| {
                let request = AuthorizationRequest {
                    request_id: format!("req-{}", resource.id),
                    authorization_ref: reference(ResourceType::Other, "auth"),
                    subject_ref: reference(ResourceType::Party, "researcher"),
                    action: reference(ResourceType::Action, action_id),
                    resource_ref: resource.clone(),
                    purpose: "knowledge graph".into(),
                    policy_refs: vec![reference(ResourceType::Other, "policy")],
                    jurisdiction_ref: None,
                    data_class: Some("research".into()),
                    requested_at_epoch_seconds: 100,
                    freshness_seconds: Some(100),
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
                    constraints: vec![],
                    policy_refs: request.policy_refs.clone(),
                    evaluated_at_epoch_seconds: 100,
                    expires_at_epoch_seconds: Some(200),
                };
                KnowledgeGraphResourceAuthorization {
                    request,
                    result,
                    now_epoch_seconds: 100,
                }
            })
            .collect(),
    }
}

fn access(resource: &ResourceRef) -> KnowledgeGraphAccessContext {
    access_many(std::slice::from_ref(resource), "graph.read")
}

fn write_access(resources: &[ResourceRef]) -> KnowledgeGraphAccessContext {
    access_many(resources, "graph.write")
}

#[test]
fn graph_contract_requires_authorized_resource_scope() {
    let doc = reference(ResourceType::Document, "doc-1");
    let other = reference(ResourceType::Document, "doc-2");
    let mut graph = KnowledgeGraph::default();
    graph
        .insert_node(
            KnowledgeNode {
                resource: doc.clone(),
                labels: BTreeSet::new(),
            },
            &write_access(std::slice::from_ref(&doc)),
        )
        .unwrap();
    assert_eq!(
        graph.get_node(&other, &access(&doc)),
        Err("knowledge graph read is unauthorized")
    );
}

#[test]
fn graph_contract_preserves_external_federation_reference() {
    let source = reference(ResourceType::Document, "doc-1");
    let external = reference(ResourceType::Other, "external-record");
    let provenance = reference(ResourceType::Provenance, "prov-1");
    let mut graph = KnowledgeGraph::default();
    graph
        .insert_node(
            KnowledgeNode {
                resource: source.clone(),
                labels: BTreeSet::new(),
            },
            &write_access(std::slice::from_ref(&source)),
        )
        .unwrap();
    graph
        .insert_edge(
            KnowledgeEdge::new(
                "edge-1",
                source.clone(),
                "federates_to",
                external.clone(),
                KnowledgeLinkClass::External,
                provenance,
            )
            .unwrap(),
            &write_access(&[source.clone(), external.clone()]),
        )
        .unwrap();
    assert_eq!(
        graph
            .traverse(
                &source,
                1,
                &access_many(&[source.clone(), external.clone()], "graph.read")
            )
            .unwrap(),
        vec![source, external]
    );
}

#[test]
fn graph_traversal_rejects_an_unauthorized_target() {
    let source = reference(ResourceType::Document, "doc-source");
    let target = reference(ResourceType::Document, "doc-private");
    let provenance = reference(ResourceType::Provenance, "prov-private");
    let mut graph = KnowledgeGraph::default();
    graph
        .insert_node(
            KnowledgeNode {
                resource: source.clone(),
                labels: BTreeSet::new(),
            },
            &write_access(std::slice::from_ref(&source)),
        )
        .unwrap();
    graph
        .insert_edge(
            KnowledgeEdge::new(
                "edge-private",
                source.clone(),
                "references",
                target.clone(),
                KnowledgeLinkClass::Forward,
                provenance,
            )
            .unwrap(),
            &write_access(&[source.clone(), target.clone()]),
        )
        .unwrap();

    assert_eq!(
        graph.traverse(&source, 1, &access(&source)).unwrap(),
        vec![source]
    );
}

#[test]
fn graph_access_rejects_expired_canonical_authorization() {
    let resource = reference(ResourceType::Document, "doc-expired");
    let mut context = access(&resource);
    context.authorizations[0].now_epoch_seconds = 200;
    assert!(!context.permits(&resource, "graph.read"));
}

#[test]
fn edge_mutation_rejects_an_unauthorized_target() {
    let source = reference(ResourceType::Document, "doc-source");
    let target = reference(ResourceType::Document, "doc-private");
    let provenance = reference(ResourceType::Provenance, "prov-private");
    let mut graph = KnowledgeGraph::default();
    graph
        .insert_node(
            KnowledgeNode {
                resource: source.clone(),
                labels: BTreeSet::new(),
            },
            &write_access(std::slice::from_ref(&source)),
        )
        .unwrap();

    assert_eq!(
        graph.insert_edge(
            KnowledgeEdge::new(
                "edge-private",
                source.clone(),
                "references",
                target,
                KnowledgeLinkClass::Forward,
                provenance,
            )
            .unwrap(),
            &write_access(std::slice::from_ref(&source)),
        ),
        Err("knowledge graph edge mutation is unauthorized")
    );
}
