use crate::authorization::{AuthorizationEvaluator, AuthorizationRequest};
use crate::authorization_enforcement::{current_epoch_seconds, validate_authorization};
use crate::{AuthorizationResult, Id, ResourceRef, ResourceType};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

const GRAPH_READ: &str = "graph.read";
const GRAPH_WRITE: &str = "graph.write";

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
        if edge_id.trim().is_empty() || predicate.trim().is_empty() {
            return Err("knowledge edge id and predicate are required");
        }
        if source.id.is_empty() || target.id.is_empty() || provenance.id.is_empty() {
            return Err("knowledge edge endpoints and provenance are required");
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

/// Internal graph state. Callers must use AuthorizedKnowledgeGraph for reads
/// and mutations; authorization result/time data never enters this structure.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct KnowledgeGraph {
    nodes: BTreeMap<ResourceRef, KnowledgeNode>,
    edges: BTreeMap<Id, KnowledgeEdge>,
}

impl KnowledgeGraph {
    fn insert_node(&mut self, node: KnowledgeNode) {
        self.nodes.insert(node.resource.clone(), node);
    }

    fn insert_edge(&mut self, edge: KnowledgeEdge) -> Result<(), &'static str> {
        // Struct fields remain public for wire/API ergonomics, so constructors
        // are not the trust boundary. Validate every invariant before mutating
        // state; a rejected edge must never remain in the graph.
        if edge.edge_id.trim().is_empty()
            || edge.predicate.trim().is_empty()
            || edge.source.id.is_empty()
            || edge.target.id.is_empty()
            || edge.provenance.id.is_empty()
        {
            return Err("knowledge edge contains an incomplete canonical field");
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

    fn get_node(&self, resource: &ResourceRef) -> Option<KnowledgeNode> {
        self.nodes.get(resource).cloned()
    }

    fn validate_conformance(&self) -> Result<(), &'static str> {
        for edge in self.edges.values() {
            if edge.edge_id.is_empty()
                || edge.predicate.trim().is_empty()
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

/// Authorized graph facade. It obtains every decision from the configured
/// canonical evaluator and reads the clock at the protected boundary. Callers
/// can submit request intent, but cannot inject a fabricated Allow result or
/// choose the timestamp used to validate that decision.
pub struct AuthorizedKnowledgeGraph<'a, E: AuthorizationEvaluator> {
    graph: KnowledgeGraph,
    evaluator: &'a E,
}

impl<'a, E: AuthorizationEvaluator> AuthorizedKnowledgeGraph<'a, E> {
    pub fn new(evaluator: &'a E) -> Self {
        Self {
            graph: KnowledgeGraph::default(),
            evaluator,
        }
    }

    fn authorize(
        &self,
        intent: &AuthorizationRequest,
        resource: &ResourceRef,
        action_id: &str,
    ) -> Result<(), &'static str> {
        if intent.request_id.trim().is_empty()
            || intent.authorization_ref.id.trim().is_empty()
            || intent.subject_ref.id.trim().is_empty()
            || intent.purpose.trim().is_empty()
        {
            return Err("knowledge graph authorization request is invalid");
        }
        let now = current_epoch_seconds()
            .ok_or("knowledge graph authorization clock is unavailable")?;
        let mut request = intent.clone();
        request.request_id = format!(
            "{}:{}:{:?}:{}",
            intent.request_id,
            action_id,
            resource.resource_type,
            resource.id
        );
        request.action = ResourceRef::new(ResourceType::Action, action_id)
            .map_err(|_| "knowledge graph action is invalid")?;
        request.resource_ref = resource.clone();

        let result: AuthorizationResult = self.evaluator.evaluate(&request);
        validate_authorization(&request, &result, now)
            .map_err(|_| "knowledge graph authorization is denied or invalid")
    }

    pub fn insert_node(
        &mut self,
        intent: &AuthorizationRequest,
        node: KnowledgeNode,
    ) -> Result<(), &'static str> {
        self.authorize(intent, &node.resource, GRAPH_WRITE)
            .map_err(|_| "knowledge graph node mutation is unauthorized")?;
        self.graph.insert_node(node);
        self.graph.validate_conformance()
    }

    pub fn insert_edge(
        &mut self,
        intent: &AuthorizationRequest,
        edge: KnowledgeEdge,
    ) -> Result<(), &'static str> {
        self.authorize(intent, &edge.source, GRAPH_WRITE)
            .map_err(|_| "knowledge graph edge mutation is unauthorized")?;
        self.authorize(intent, &edge.target, GRAPH_WRITE)
            .map_err(|_| "knowledge graph edge mutation is unauthorized")?;
        self.graph.insert_edge(edge)?;
        self.graph.validate_conformance()
    }

    pub fn get_node(
        &self,
        intent: &AuthorizationRequest,
        resource: &ResourceRef,
    ) -> Result<Option<KnowledgeNode>, &'static str> {
        self.authorize(intent, resource, GRAPH_READ)
            .map_err(|_| "knowledge graph read is unauthorized")?;
        Ok(self.graph.get_node(resource))
    }

    pub fn traverse(
        &self,
        intent: &AuthorizationRequest,
        start: &ResourceRef,
        max_depth: usize,
    ) -> Result<Vec<ResourceRef>, &'static str> {
        self.authorize(intent, start, GRAPH_READ)
            .map_err(|_| "knowledge graph traversal is unauthorized")?;
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
            for edge in self.graph.edges.values().filter(|edge| edge.source == current) {
                // The current resource has already been authorized. A target
                // that is denied, expired, or cannot be evaluated is omitted
                // so traversal does not disclose the existence of a protected
                // node via a distinct error.
                if self.authorize(intent, &edge.target, GRAPH_READ).is_err() {
                    continue;
                }
                if !visited.contains(&edge.target) {
                    queue.push_back((edge.target.clone(), depth + 1));
                }
            }
        }
        Ok(result)
    }

}
