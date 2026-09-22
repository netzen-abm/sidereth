use crate::{AuthorizationDecision, AuthorizationResult, Id, ResourceRef};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Semantic relationship strength from the canonical resource-link contract.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeLinkClass {
    Strong,
    Forward,
    External,
}

/// A typed node in the derived knowledge-graph projection.
///
/// The graph is intentionally provider-neutral and storage-neutral. It is a
/// projection over canonical resources; it is not a second source of truth.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnowledgeNode {
    pub resource: ResourceRef,
    pub labels: BTreeSet<String>,
}

/// A typed, attributable relationship between two resources.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnowledgeEdge {
    pub edge_id: Id,
    pub source: ResourceRef,
    pub predicate: String,
    pub target: ResourceRef,
    pub class: KnowledgeLinkClass,
    pub provenance: ResourceRef,
}

impl KnowledgeEdge {
    pub fn new(
        edge_id: impl Into<Id>,
        source: ResourceRef,
        predicate: impl Into<String>,
        target: ResourceRef,
        class: KnowledgeLinkClass,
        provenance: ResourceRef,
    ) -> Result<Self, &'static str> {
        let edge_id = edge_id.into();
        let predicate = predicate.into();
        if edge_id.is_empty() || predicate.is_empty() {
            return Err("knowledge edge id and predicate are required");
        }
        Ok(Self {
            edge_id,
            source,
            predicate,
            target,
            class,
            provenance,
        })
    }
}

/// Explicit invocation context for graph mutation/read operations.
///
/// The graph never evaluates policy itself. It consumes the frozen
/// authorization result and fails closed when the result is absent, denied,
/// expired, or bound to a different resource.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnowledgeGraphAccessContext {
    pub authorization: AuthorizationResult,
    pub now_epoch_seconds: u64,
}

impl KnowledgeGraphAccessContext {
    pub fn permits(&self, resource: &ResourceRef) -> bool {
        self.authorization.decision == AuthorizationDecision::Allow
            && self.authorization.resource_ref == *resource
            && self
                .authorization
                .expires_at_epoch_seconds
                .map(|expires| self.now_epoch_seconds <= expires)
                .unwrap_or(true)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnowledgeGraph {
    nodes: BTreeMap<ResourceRef, KnowledgeNode>,
    edges: BTreeMap<Id, KnowledgeEdge>,
}

impl KnowledgeGraph {
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn insert_node(
        &mut self,
        node: KnowledgeNode,
        access: &KnowledgeGraphAccessContext,
    ) -> Result<(), &'static str> {
        if !access.permits(&node.resource) {
            return Err("knowledge graph node mutation is unauthorized");
        }
        self.nodes.insert(node.resource.clone(), node);
        Ok(())
    }

    pub fn insert_edge(
        &mut self,
        edge: KnowledgeEdge,
        access: &KnowledgeGraphAccessContext,
    ) -> Result<(), &'static str> {
        if !access.permits(&edge.source) {
            return Err("knowledge graph edge mutation is unauthorized");
        }
        if edge.class == KnowledgeLinkClass::Strong && !self.nodes.contains_key(&edge.target) {
            return Err("strong knowledge link target must exist");
        }
        if self.edges.contains_key(&edge.edge_id) {
            return Err("knowledge edge id already exists");
        }
        self.edges.insert(edge.edge_id.clone(), edge);
        Ok(())
    }

    pub fn get_node(
        &self,
        resource: &ResourceRef,
        access: &KnowledgeGraphAccessContext,
    ) -> Result<Option<&KnowledgeNode>, &'static str> {
        if !access.permits(resource) {
            return Err("knowledge graph read is unauthorized");
        }
        Ok(self.nodes.get(resource))
    }

    /// Breadth-first traversal over locally materialized edges.
    ///
    /// External and unresolved forward targets are returned as resource
    /// references without inventing local nodes.
    pub fn traverse(
        &self,
        start: &ResourceRef,
        max_depth: usize,
        access: &KnowledgeGraphAccessContext,
    ) -> Result<Vec<ResourceRef>, &'static str> {
        if !access.permits(start) {
            return Err("knowledge graph traversal is unauthorized");
        }

        let mut visited = BTreeSet::new();
        let mut queue = VecDeque::from([(start.clone(), 0usize)]);
        let mut result = Vec::new();

        while let Some((current, depth)) = queue.pop_front() {
            if !visited.insert(current.clone()) {
                continue;
            }
            result.push(current.clone());
            if depth >= max_depth {
                continue;
            }

            for edge in self.edges.values().filter(|edge| edge.source == current) {
                if !access.permits(&edge.source) {
                    return Err("knowledge graph traversal crossed an unauthorized source");
                }
                if !visited.contains(&edge.target) {
                    queue.push_back((edge.target.clone(), depth + 1));
                }
            }
        }

        Ok(result)
    }

    pub fn validate_conformance(&self) -> Result<(), &'static str> {
        for edge in self.edges.values() {
            if edge.edge_id.is_empty()
                || edge.predicate.is_empty()
                || edge.source.id.is_empty()
                || edge.target.id.is_empty()
                || edge.provenance.id.is_empty()
            {
                return Err("knowledge edge contains an incomplete canonical field");
            }
            if edge.class == KnowledgeLinkClass::Strong
                && !self.nodes.contains_key(&edge.target)
            {
                return Err("strong knowledge link target is unresolved");
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AuthorizationDecision, ResourceType};

    fn reference(resource_type: ResourceType, id: &str) -> ResourceRef {
        ResourceRef::new(resource_type, id).unwrap()
    }

    fn access(resource: &ResourceRef, now: u64) -> KnowledgeGraphAccessContext {
        KnowledgeGraphAccessContext {
            authorization: AuthorizationResult {
                request_id: "req-kg-1".into(),
                authorization_ref: reference(ResourceType::Other, "auth-1"),
                subject_ref: reference(ResourceType::Party, "researcher-1"),
                action: reference(ResourceType::Action, "graph-read-write"),
                resource_ref: resource.clone(),
                purpose: "knowledge graph research".into(),
                jurisdiction_ref: None,
                data_class: Some("research".into()),
                decision: AuthorizationDecision::Allow,
                constraints: vec![],
                policy_refs: vec![reference(ResourceType::Other, "policy-kg")],
                evaluated_at_epoch_seconds: now,
                expires_at_epoch_seconds: Some(now + 60),
            },
            now_epoch_seconds: now,
        }
    }

    #[test]
    fn strong_edges_require_materialized_targets() {
        let source = reference(ResourceType.Document, "doc-1");
        let target = reference(ResourceType.Evidence, "evidence-1");
        let provenance = reference(ResourceType.Provenance, "prov-1");
        let mut graph = KnowledgeGraph::default();
        graph.insert_node(
            KnowledgeNode {
                resource: source.clone(),
                labels: BTreeSet::from(["document".into()]),
            },
            &access(&source, 100),
        ).unwrap();

        let edge = KnowledgeEdge::new(
            "edge-1", source.clone(), "supports", target.clone(),
            KnowledgeLinkClass::Strong, provenance,
        ).unwrap();

        assert_eq!(
            graph.insert_edge(edge, &access(&source, 100)),
            Err("strong knowledge link target must exist")
        );
    }

    #[test]
    fn authorization_is_fail_closed_and_resource_bound() {
        let resource = reference(ResourceType::Document, "doc-1");
        let other = reference(ResourceType::Document, "doc-2");
        let denied = KnowledgeGraphAccessContext {
            authorization: AuthorizationResult {
                request_id: "req-kg-2".into(),
                authorization_ref: reference(ResourceType.Other, "auth-2"),
                subject_ref: reference(ResourceType::Party, "researcher-1"),
                action: reference(ResourceType::Action, "graph-read"),
                resource_ref: resource.clone(),
                purpose: "knowledge graph research".into(),
                jurisdiction_ref: None,
                data_class: Some("research".into()),
                decision: AuthorizationDecision::Deny,
                constraints: vec![],
                policy_refs: vec![],
                evaluated_at_epoch_seconds: 100,
                expires_at_epoch_seconds: Some(200),
            },
            now_epoch_seconds: 100,
        };

        assert!(!denied.permits(&resource));
        assert!(!access(&resource, 100).permits(&other));
    }

    #[test]
    fn expired_authorization_is_rejected() {
        let resource = reference(ResourceType::Document, "doc-1");
        let mut ctx = access(&resource, 100);
        ctx.now_epoch_seconds = 161;
        assert!(!ctx.permits(&resource));
    }

    #[test]
    fn traversal_is_deterministic_and_preserves_external_refs() {
        let a = reference(ResourceType.Document, "a");
        let b = reference(ResourceType.Evidence, "b");
        let external = reference(ResourceType.Other, "external-1");
        let provenance = reference(ResourceType.Provenance, "p1");
        let mut graph = KnowledgeGraph::default();

        for node in [&a, &b] {
            graph.insert_node(
                KnowledgeNode {
                    resource: node.clone(),
                    labels: BTreeSet::new(),
                },
                &access(&a, 100),
            ).unwrap();
        }

        graph.insert_edge(
            KnowledgeEdge::new(
                "e1", a.clone(), "supports", b.clone(),
                KnowledgeLinkClass::Strong, provenance.clone(),
            ).unwrap(),
            &access(&a, 100),
        ).unwrap();

        graph.insert_edge(
            KnowledgeEdge::new(
                "e2", a.clone(), "references", external.clone(),
                KnowledgeLinkClass::External, provenance,
            ).unwrap(),
            &access(&a, 100),
        ).unwrap();

        let result = graph.traverse(&a, 1, &access(&a, 100)).unwrap();
        assert_eq!(result, vec![a, b, external]);
        assert!(graph.validate_conformance().is_ok());
    }
}
