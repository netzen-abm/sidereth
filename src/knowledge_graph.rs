use crate::{AuthorizationDecision, AuthorizationResult, Id, ResourceRef};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeLinkClass {
    Strong,
    Forward,
    External,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnowledgeNode {
    pub resource: ResourceRef,
    pub labels: BTreeSet<String>,
}

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
            if edge.class == KnowledgeLinkClass::Strong && !self.nodes.contains_key(&edge.target) {
                return Err("strong knowledge link target is unresolved");
            }
        }
        Ok(())
    }
}
