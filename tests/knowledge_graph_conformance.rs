use sidereth_core::authorization::AuthorizationEvaluator;
use sidereth_core::{
    AuthorizationDecision, AuthorizationRequest, AuthorizationResult, AuthorizedKnowledgeGraph,
    KnowledgeEdge, KnowledgeLinkClass, KnowledgeNode, ResourceRef, ResourceType,
};
use std::collections::BTreeSet;
use std::time::{SystemTime, UNIX_EPOCH};

fn reference(t: ResourceType, id: &str) -> ResourceRef {
    ResourceRef::new(t, id).unwrap()
}

fn now_epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn request(resource: &ResourceRef) -> AuthorizationRequest {
    AuthorizationRequest {
        request_id: format!("req-{}", resource.id),
        authorization_ref: reference(ResourceType::Other, "auth"),
        subject_ref: reference(ResourceType::Party, "researcher"),
        action: reference(ResourceType::Action, "caller-intent-only"),
        resource_ref: resource.clone(),
        purpose: "knowledge graph".into(),
        policy_refs: vec![reference(ResourceType::Other, "policy")],
        jurisdiction_ref: None,
        data_class: Some("research".into()),
        requested_at_epoch_seconds: now_epoch_seconds(),
        freshness_seconds: Some(300),
    }
}

#[derive(Default)]
struct TestEvaluator {
    denied_resource: Option<ResourceRef>,
    denied_action: Option<String>,
    expired: bool,
}

impl AuthorizationEvaluator for TestEvaluator {
    fn evaluate(&self, request: &AuthorizationRequest) -> AuthorizationResult {
        let now = now_epoch_seconds();
        let denied = self.denied_resource.as_ref() == Some(&request.resource_ref)
            || self.denied_action.as_deref() == Some(request.action.id.as_str());
        AuthorizationResult {
            request_id: request.request_id.clone(),
            authorization_ref: request.authorization_ref.clone(),
            subject_ref: request.subject_ref.clone(),
            action: request.action.clone(),
            resource_ref: request.resource_ref.clone(),
            purpose: request.purpose.clone(),
            jurisdiction_ref: request.jurisdiction_ref.clone(),
            data_class: request.data_class.clone(),
            decision: if denied {
                AuthorizationDecision::Deny
            } else {
                AuthorizationDecision::Allow
            },
            constraints: vec![sidereth_core::authorization::AuthorizationConstraint {
                key: "scope".into(),
                value: "exact_resource".into(),
            }],
            policy_refs: request.policy_refs.clone(),
            evaluated_at_epoch_seconds: now,
            expires_at_epoch_seconds: Some(if self.expired { now } else { now + 300 }),
        }
    }
}

fn node(resource: ResourceRef) -> KnowledgeNode {
    KnowledgeNode {
        resource,
        labels: BTreeSet::new(),
    }
}

#[test]
fn graph_read_is_bound_to_exact_resource() {
    let doc = reference(ResourceType::Document, "doc-1");
    let other = reference(ResourceType::Document, "doc-2");
    let evaluator = TestEvaluator {
        denied_resource: Some(other.clone()),
        ..Default::default()
    };
    let mut graph = AuthorizedKnowledgeGraph::new(&evaluator);
    graph.insert_node(&request(&doc), node(doc.clone())).unwrap();

    assert_eq!(
        graph.get_node(&request(&doc), &other),
        Err("knowledge graph read is unauthorized")
    );
}

#[test]
fn graph_contract_preserves_external_federation_reference() {
    let source = reference(ResourceType::Document, "doc-1");
    let external = reference(ResourceType::Other, "external-record");
    let provenance = reference(ResourceType::Provenance, "prov-1");
    let evaluator = TestEvaluator::default();
    let mut graph = AuthorizedKnowledgeGraph::new(&evaluator);
    graph.insert_node(&request(&source), node(source.clone())).unwrap();
    graph
        .insert_edge(
            &request(&source),
            KnowledgeEdge::new(
                "edge-1",
                source.clone(),
                "federates_to",
                external.clone(),
                KnowledgeLinkClass::External,
                provenance,
            )
            .unwrap(),
        )
        .unwrap();

    assert_eq!(
        graph.traverse(&request(&source), &source, 1).unwrap(),
        vec![source, external]
    );
}

#[test]
fn graph_traversal_filters_an_unauthorized_target() {
    let source = reference(ResourceType::Document, "doc-source");
    let target = reference(ResourceType::Document, "doc-private");
    let evaluator = TestEvaluator {
        denied_resource: Some(target.clone()),
        ..Default::default()
    };
    let mut graph = AuthorizedKnowledgeGraph::new(&evaluator);
    graph.insert_node(&request(&source), node(source.clone())).unwrap();
    graph
        .insert_edge(
            &request(&source),
            KnowledgeEdge::new(
                "edge-private",
                source.clone(),
                "references",
                target,
                KnowledgeLinkClass::Forward,
                reference(ResourceType::Provenance, "prov-private"),
            )
            .unwrap(),
        )
        .unwrap();

    assert_eq!(
        graph.traverse(&request(&source), &source, 1).unwrap(),
        vec![source]
    );
}

#[test]
fn expired_evaluator_decision_fails_closed() {
    let resource = reference(ResourceType::Document, "doc-expired");
    let evaluator = TestEvaluator {
        expired: true,
        ..Default::default()
    };
    let mut graph = AuthorizedKnowledgeGraph::new(&evaluator);
    assert_eq!(
        graph.insert_node(&request(&resource), node(resource)),
        Err("knowledge graph node mutation is unauthorized")
    );
}

#[test]
fn edge_mutation_requires_authorization_for_both_endpoints() {
    let source = reference(ResourceType::Document, "doc-source");
    let target = reference(ResourceType::Document, "doc-private");
    let evaluator = TestEvaluator {
        denied_resource: Some(target.clone()),
        ..Default::default()
    };
    let mut graph = AuthorizedKnowledgeGraph::new(&evaluator);
    graph.insert_node(&request(&source), node(source.clone())).unwrap();

    assert_eq!(
        graph.insert_edge(
            &request(&source),
            KnowledgeEdge::new(
                "edge-private",
                source,
                "references",
                target,
                KnowledgeLinkClass::Forward,
                reference(ResourceType::Provenance, "prov-private"),
            )
            .unwrap(),
        ),
        Err("knowledge graph edge mutation is unauthorized")
    );
}

#[test]
fn read_authorization_cannot_mutate_graph() {
    let resource = reference(ResourceType::Document, "doc-read-only");
    let evaluator = TestEvaluator {
        denied_action: Some("graph.write".into()),
        ..Default::default()
    };
    let mut graph = AuthorizedKnowledgeGraph::new(&evaluator);

    assert_eq!(
        graph.insert_node(&request(&resource), node(resource.clone())),
        Err("knowledge graph node mutation is unauthorized")
    );
    // A read under a separate action remains possible but returns no node.
    assert!(graph.get_node(&request(&resource), &resource).unwrap().is_none());
}

#[test]
fn write_authorization_cannot_read_graph() {
    let resource = reference(ResourceType::Document, "doc-write-only");
    let evaluator = TestEvaluator {
        denied_action: Some("graph.read".into()),
        ..Default::default()
    };
    let mut graph = AuthorizedKnowledgeGraph::new(&evaluator);
    graph.insert_node(&request(&resource), node(resource.clone())).unwrap();

    assert_eq!(
        graph.get_node(&request(&resource), &resource),
        Err("knowledge graph read is unauthorized")
    );
}

#[test]
fn malformed_edge_is_rejected_before_graph_mutation() {
    let source = reference(ResourceType::Document, "doc-source");
    let target = reference(ResourceType::Document, "doc-target");
    let evaluator = TestEvaluator::default();
    let mut graph = AuthorizedKnowledgeGraph::new(&evaluator);
    graph.insert_node(&request(&source), node(source.clone())).unwrap();

    let malformed = KnowledgeEdge {
        edge_id: "edge-retry".into(),
        source: source.clone(),
        predicate: "   ".into(),
        target: target.clone(),
        class: KnowledgeLinkClass::Forward,
        provenance: reference(ResourceType::Provenance, "prov-retry"),
    };
    assert_eq!(
        graph.insert_edge(&request(&source), malformed),
        Err("knowledge edge contains an incomplete canonical field")
    );

    // Reusing the same ID with a valid edge proves the rejected attempt did not
    // leave a partially inserted edge behind.
    graph
        .insert_edge(
            &request(&source),
            KnowledgeEdge::new(
                "edge-retry",
                source.clone(),
                "references",
                target,
                KnowledgeLinkClass::Forward,
                reference(ResourceType::Provenance, "prov-retry"),
            )
            .unwrap(),
        )
        .unwrap();
}

#[test]
fn strong_edge_rejects_missing_target_without_mutating_graph() {
    let source = reference(ResourceType::Document, "doc-source");
    let target = reference(ResourceType::Document, "doc-missing");
    let evaluator = TestEvaluator::default();
    let mut graph = AuthorizedKnowledgeGraph::new(&evaluator);
    graph.insert_node(&request(&source), node(source.clone())).unwrap();

    assert_eq!(
        graph.insert_edge(
            &request(&source),
            KnowledgeEdge::new(
                "edge-strong",
                source.clone(),
                "requires",
                target,
                KnowledgeLinkClass::Strong,
                reference(ResourceType::Provenance, "prov-strong"),
            )
            .unwrap(),
        ),
        Err("strong knowledge link target must exist")
    );
    assert!(graph.get_node(&request(&source), &source).unwrap().is_some());
}
