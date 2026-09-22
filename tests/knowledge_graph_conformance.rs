use sidereth::{
    AuthorizationDecision, AuthorizationResult, KnowledgeEdge, KnowledgeGraph,
    KnowledgeGraphAccessContext, KnowledgeLinkClass, KnowledgeNode, ResourceRef, ResourceType,
};
use std::collections::BTreeSet;

fn reference(t: ResourceType, id: &str) -> ResourceRef { ResourceRef::new(t, id).unwrap() }

fn access(resource: &ResourceRef) -> KnowledgeGraphAccessContext {
    KnowledgeGraphAccessContext {
        authorization: AuthorizationResult {
            request_id: "req-conformance".into(),
            authorization_ref: reference(ResourceType::Other, "auth"),
            subject_ref: reference(ResourceType::Party, "researcher"),
            action: reference(ResourceType::Action, "graph"),
            resource_ref: resource.clone(),
            purpose: "knowledge graph".into(),
            jurisdiction_ref: None,
            data_class: Some("research".into()),
            decision: AuthorizationDecision::Allow,
            constraints: vec![],
            policy_refs: vec![reference(ResourceType::Other, "policy")],
            evaluated_at_epoch_seconds: 100,
            expires_at_epoch_seconds: Some(200),
        },
        now_epoch_seconds: 100,
    }
}

#[test]
fn graph_contract_requires_authorized_resource_scope() {
    let doc = reference(ResourceType::Document, "doc-1");
    let other = reference(ResourceType::Document, "doc-2");
    let mut graph = KnowledgeGraph::default();
    let node = KnowledgeNode { resource: doc.clone(), labels: BTreeSet::new() };
    graph.insert_node(node, &access(&doc)).unwrap();
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
    graph.insert_node(
        KnowledgeNode { resource: source.clone(), labels: BTreeSet::new() },
        &access(&source),
    ).unwrap();
    graph.insert_edge(
        KnowledgeEdge::new(
            "edge-1", source.clone(), "federates_to", external.clone(),
            KnowledgeLinkClass::External, provenance,
        ).unwrap(),
        &access(&source),
    ).unwrap();
    assert_eq!(graph.traverse(&source, 1, &access(&source)).unwrap(), vec![source, external]);
}
