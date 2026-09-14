//! SIDERETH Core
//! Legal and regulatory workflow primitives.
//!
//! The core is deterministic and independent of AI, network transport,
//! government submission, and autonomous legal decisions.

use serde::{Deserialize, Serialize};

pub type Id = String;

/// Explicit cross-primitive reference contract for ecosystem boundaries.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ResourceRef {
    pub resource_type: ResourceType,
    pub id: Id,
}

impl ResourceRef {
    pub fn new(resource_type: ResourceType, id: impl Into<Id>) -> Result<Self, &'static str> {
        let id = id.into();
        if id.is_empty() { return Err("resource reference id is required"); }
        Ok(Self { resource_type, id })
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ResourceType {
    Case, Incident, Event, Observation, Evidence, Authority, Jurisdiction, Party,
    PartyRelationship, Document, Action, Deadline, Response, Escalation, Remedy,
    Resolution, Procedure, ComplianceRequirement, LegalSource, Timeline, Audit,
    Provenance, Idempotency, Other,
}

pub mod action;
pub mod audit;
pub mod authority;
pub mod authorization;
pub mod capability_registry;
pub mod command;
pub mod compliance;
pub mod deadline;
pub mod document;
pub mod event;
pub mod evidence;
pub mod evidence_repository;
pub mod evidence_store;
pub mod evidence_trust;
pub mod intelligence;
pub mod jurisdiction;
pub mod legal_source;
pub mod legal_source_registry;
pub mod lifecycle;
pub mod local_store;
pub mod longitudinal;
pub mod observation;
pub mod observation_command;
pub mod observation_lifecycle;
pub mod party;
pub mod party_repository;
pub mod persistence;
#[cfg(feature = "postgres")]
pub mod postgres;
pub mod procedure;
pub mod provenance;
pub mod query;
pub mod remedy;
pub mod repository;
pub mod resolution;
pub mod response;
pub mod security;
pub mod service;
pub mod service_canonical;
pub mod timeline;
#[allow(clippy::manual_flatten)]
pub mod tool_registry;

pub use action::{Action, ActionKind, ActionStatus, ApprovalDecision, ApprovalOrigin, ApprovalRecord, ExecutionGate, ExecutionGateError, ExecutionGateInput};
pub use audit::{AuditRecord, AuditSink, InMemoryAudit};
pub use authority::{Authority, AuthorityPower, AuthorityRegistry, AuthorityStatus, AuthorityType};
pub use authorization::{AccessAction, AccessRequest, AuthorizationDecision, AuthorizationEvaluator, AuthorizationPolicy, AuthorizationRequest, AuthorizationResult, CaseAccessPolicy};
pub use capability_registry::{CapabilityDataClass, CapabilityDependency, CapabilityImplementation, CapabilityLifecycle, CapabilityRegistryEntry, CapabilityRegistryError, CapabilityRiskClass, CapabilityVersion, ExecutionMode, InMemoryCapabilityRegistry, RegistryAuditRecord, RegistryCriteria, VersionRequirement};
pub use command::{apply_plan, execute_authoritative_command, AtomicCommandPlan, AuthoritativeCommandError};
pub use compliance::{ComplianceRequirement, ComplianceStatus};
pub use deadline::{Deadline, DeadlineStatus};
pub use document::{Document, DocumentType};
pub use event::EventEnvelope;
pub use evidence::{DerivedArtifact, EvidenceOriginal};
pub use evidence_trust::EvidenceTrust;
pub use intelligence::{IntelligenceRequest, IntelligenceResult};
pub use jurisdiction::Jurisdiction;
pub use legal_source::LegalSource;
pub use lifecycle::Lifecycle;
pub use longitudinal::{LongitudinalEntry, LongitudinalView};
pub use observation::{Observation, ObservationOrigin, ObservationStatus};
pub use observation_command::{CreateObservationCommand, ObservationCommandError};
pub use observation_lifecycle::{ObservationLifecycleCommand, ObservationLifecycleCommandContext, ObservationLifecycleError, ObservationLifecycleOperation};
pub use party::{Party, PartyRelationship};
pub use persistence::{PersistenceError, ResourceLink, ResourceLinkClass, ResourceRecord, ResourceWrite, ResourceWriteMode, Revision, UnitOfWork, UnitOfWorkContext, UnitOfWorkError, UnitOfWorkFactory};
pub use query::{QueryError, ResourceQuery, ResourceQueryRequest, UnitOfWorkResourceQuery};
pub use remedy::Remedy;
pub use repository::Repository;
pub use resolution::Resolution;
pub use response::Response;
pub use security::{AuthorizedAudit, EvidenceError, EvidenceExport, EvidenceExporter, KeyProvider, RetentionPolicy};
pub use service::{CaseCommand, CaseService, CommandContext, CommandResult, ServiceError};
pub use timeline::Timeline;
pub use tool_registry::{InMemoryToolRegistry, ToolDataClass, ToolDependency, ToolExecutionMode, ToolImplementation, ToolRegistry, ToolRegistryEntry, ToolRegistryError, ToolRiskClass, ToolVersion};
