//! Deterministic preflight checks for administrative correspondence drafts.
//!
//! `evaluate` validates the request shape and typed references;
//! `evaluate_with_registries` additionally resolves supported references and
//! checks registry status, jurisdiction scope, and a canonical legal-applicability
//! instant. Neither method decides whether a legal proposition is correct,
//! authenticates source law beyond recorded metadata, calculates RTI/appeal
//! deadlines, reads document bytes, or authorizes dispatch. A structurally
//! complete result is never authorization to send a document; source content
//! verification, applicable-procedure review, user approval, and dispatch remain
//! separate gates.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::authority::{AuthorityRegistry, AuthorityStatus};
use crate::deadline::CivilDate;
use crate::document::{
    DocumentRegistry, DocumentStatus, IntegrityStatus as DocumentIntegrityStatus,
};
use crate::evidence_repository::EvidenceTrustRepository;
use crate::jurisdiction::{JurisdictionRegistry, JurisdictionStatus};
use crate::legal_source_registry::LegalSourceRegistry;
use crate::procedure::{ProcedureRegistry, ProcedureStatus};
use crate::{PropositionType, ResourceRef, ResourceType, VerificationStatus};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CorrespondenceTrack {
    NoticeReply,
    RtiApplication,
    FollowUpOrAppeal,
}

/// The requested material must already exist or be held by the public authority.
/// A request for new analysis is retained as an explicit review item, not silently
/// rewritten into a demand that the authority create new information.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RtiRequestKind {
    ExistingRecord,
    ExistingStatisticsOrDataset,
    Inspection,
    CertifiedCopy,
    ExistingElectronicRecord,
    ExplanationOrNewAnalysis,
}

/// One item in the RTI request list. Reporting periods are optional and
/// inclusive. Personal/sensitive data is flagged for disclosure review rather
/// than automatically included or automatically refused.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RtiRequestItem {
    pub description: String,
    pub kind: RtiRequestKind,
    pub period_start: Option<CivilDate>,
    pub period_end: Option<CivilDate>,
    pub includes_personal_or_sensitive_data: bool,
}

/// Optional scheme-specific context for the universal RTI form. Fee and filing
/// rules come from the selected verified procedure; no fee amount or identity
/// document is assumed by this type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RtiApplicationDetails {
    pub scheme_name: Option<String>,
    pub official_reference: Option<ResourceRef>,
    pub reporting_period_start: Option<CivilDate>,
    pub reporting_period_end: Option<CivilDate>,
    pub fee_payment_proof: Option<ResourceRef>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FindingSeverity {
    Blocking,
    Review,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChecklistFindingCode {
    MissingIssuingAuthority,
    InvalidIssuingAuthorityType,
    MissingJurisdiction,
    InvalidJurisdictionType,
    MissingSubject,
    MissingNoticeDocument,
    InvalidNoticeDocumentType,
    MissingRecipientDesignation,
    MissingRequestedRecords,
    EmptyRequestedRecord,
    MissingProcedureReference,
    InvalidProcedureReferenceType,
    MissingTriggerDate,
    InvalidReportingPeriod,
    RequestSeeksNewAnalysisOrOpinion,
    PersonalInformationDisclosureReviewRequired,
    AuthorityNotFound,
    AuthorityInactive,
    AuthorityJurisdictionMismatch,
    DocumentJurisdictionMismatch,
    JurisdictionNotFound,
    JurisdictionInactive,
    SourceDocumentNotFound,
    SourceDocumentUnavailable,
    SourceDocumentNeedsReview,
    EvidenceNotFound,
    EvidenceVerificationFailed,
    EvidenceResolutionUnavailable,
    LegalSourceNotFound,
    LegalSourceUnverified,
    LegalSourceJurisdictionNotFound,
    LegalSourceJurisdictionMismatch,
    LegalSourceNotEffective,
    LegalSourceEffectiveDateUnparseable,
    MissingLegalApplicabilityDate,
    ProcedureNotFound,
    ProcedureInactive,
    ProcedureAuthorityMismatch,
    ProcedureJurisdictionMismatch,
    ProcedureSourceNotFound,
    InvalidOfficialReferenceType,
    MissingSchemeName,
    InvalidFeePaymentProofType,
    DocumentReferenceNotFound,
    DocumentReferenceUnavailable,
    DocumentReferenceNeedsReview,
    UnsupportedSourceReferenceType,
    EmptyClaimId,
    DuplicateClaimId,
    EmptyClaimStatement,
    ClaimHasNoSource,
    InvalidClaimSourceReference,
    LegalClaimMissingLegalSource,
    UserFactMissingDocumentOrEvidence,
    ClaimRequiresReview,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChecklistFinding {
    pub code: ChecklistFindingCode,
    pub severity: FindingSeverity,
    /// Stable field identifier for UI and automated tests; not natural-language advice.
    pub field: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChecklistReadiness {
    /// Required structure or source references are missing or malformed.
    Blocked,
    /// A draft may be prepared for review, but one or more items need review.
    ReviewRequired,
    /// Inputs pass this structural preflight only; sources still need resolution
    /// and the user must review the document before any dispatch.
    StructurallyComplete,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CorrespondenceClaim {
    pub claim_id: String,
    pub statement: String,
    pub proposition_type: PropositionType,
    pub verification_status: VerificationStatus,
    pub source_refs: Vec<ResourceRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CorrespondenceChecklistInput {
    pub track: CorrespondenceTrack,
    pub issuing_authority: Option<ResourceRef>,
    pub jurisdiction: Option<ResourceRef>,
    pub subject: Option<String>,
    /// Original notice or relevant prior document; required for notice replies and follow-ups.
    pub source_document: Option<ResourceRef>,
    /// User-provided date associated with the route (e.g. recorded delivery or decision date).
    /// This module checks presence only; it does not parse dates or calculate deadlines.
    pub trigger_date: Option<String>,
    /// Canonical UTC instant (YYYY-MM-DDTHH:MM:SSZ) for resolving legal-source
    /// applicability. It is deliberately separate from a notice/decision trigger date.
    pub legal_as_of: Option<String>,
    /// For an RTI application, normally the CPIO/SPIO or other designated recipient role.
    pub recipient_designation: Option<String>,
    /// Identifiable, typed requests for existing information; required for RTI.
    pub requested_records: Vec<RtiRequestItem>,
    pub rti_details: Option<RtiApplicationDetails>,
    /// Versioned SIDERETH procedure selected for the route, when known.
    pub procedure_ref: Option<ResourceRef>,
    /// Claims which the draft intends to state as facts, rules, procedures or interpretations.
    pub claims: Vec<CorrespondenceClaim>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChecklistResult {
    pub readiness: ChecklistReadiness,
    pub findings: Vec<ChecklistFinding>,
}

impl ChecklistResult {
    fn from_findings(findings: Vec<ChecklistFinding>) -> Self {
        let readiness = if findings
            .iter()
            .any(|finding| finding.severity == FindingSeverity::Blocking)
        {
            ChecklistReadiness::Blocked
        } else if !findings.is_empty() {
            ChecklistReadiness::ReviewRequired
        } else {
            ChecklistReadiness::StructurallyComplete
        };

        Self {
            readiness,
            findings,
        }
    }
}

impl CorrespondenceChecklistInput {
    /// Check required fields and the declared source type of each claim.
    ///
    /// The returned result is deliberately not called "valid", "legally sound",
    /// or "ready to send". Existing source registries, applicable-date checks,
    /// authorization, final user approval and any required professional review
    /// must be applied by their authoritative boundaries.
    pub fn evaluate(&self) -> ChecklistResult {
        let mut findings = Vec::new();

        match &self.issuing_authority {
            None => findings.push(blocking(
                ChecklistFindingCode::MissingIssuingAuthority,
                "issuing_authority",
            )),
            Some(reference) if !reference_has_type(reference, ResourceType::Authority) => {
                findings.push(blocking(
                    ChecklistFindingCode::InvalidIssuingAuthorityType,
                    "issuing_authority",
                ));
            }
            Some(reference) if !valid_reference(reference) => {
                findings.push(blocking(
                    ChecklistFindingCode::InvalidIssuingAuthorityType,
                    "issuing_authority",
                ));
            }
            Some(_) => {}
        }

        match &self.jurisdiction {
            None => findings.push(blocking(
                ChecklistFindingCode::MissingJurisdiction,
                "jurisdiction",
            )),
            Some(reference) if !reference_has_type(reference, ResourceType::Jurisdiction) => {
                findings.push(blocking(
                    ChecklistFindingCode::InvalidJurisdictionType,
                    "jurisdiction",
                ));
            }
            Some(reference) if !valid_reference(reference) => {
                findings.push(blocking(
                    ChecklistFindingCode::InvalidJurisdictionType,
                    "jurisdiction",
                ));
            }
            Some(_) => {}
        }

        if !has_text(self.subject.as_deref()) {
            findings.push(blocking(ChecklistFindingCode::MissingSubject, "subject"));
        }

        match self.track {
            CorrespondenceTrack::NoticeReply => {
                validate_source_document(&self.source_document, &mut findings);
                if !has_text(self.trigger_date.as_deref()) {
                    findings.push(review(
                        ChecklistFindingCode::MissingTriggerDate,
                        "trigger_date",
                    ));
                }
            }
            CorrespondenceTrack::RtiApplication => {
                if !has_text(self.recipient_designation.as_deref()) {
                    findings.push(blocking(
                        ChecklistFindingCode::MissingRecipientDesignation,
                        "recipient_designation",
                    ));
                }
                if self.requested_records.is_empty() {
                    findings.push(blocking(
                        ChecklistFindingCode::MissingRequestedRecords,
                        "requested_records",
                    ));
                }
                if self
                    .requested_records
                    .iter()
                    .any(|record| record.description.trim().is_empty())
                {
                    findings.push(blocking(
                        ChecklistFindingCode::EmptyRequestedRecord,
                        "requested_records",
                    ));
                }
                if self.requested_records.iter().any(|record| {
                    matches!(record.kind, RtiRequestKind::ExplanationOrNewAnalysis)
                }) {
                    findings.push(review(
                        ChecklistFindingCode::RequestSeeksNewAnalysisOrOpinion,
                        "requested_records",
                    ));
                }
                if self.requested_records.iter().any(|record| {
                    record.includes_personal_or_sensitive_data
                }) {
                    findings.push(review(
                        ChecklistFindingCode::PersonalInformationDisclosureReviewRequired,
                        "requested_records",
                    ));
                }
                if self.requested_records.iter().any(|record| {
                    matches!(
                        (record.period_start, record.period_end),
                        (Some(start), Some(end)) if start > end
                    )
                }) {
                    findings.push(blocking(
                        ChecklistFindingCode::InvalidReportingPeriod,
                        "requested_records",
                    ));
                }
                if let Some(details) = &self.rti_details {
                    if !has_text(details.scheme_name.as_deref()) {
                        findings.push(review(
                            ChecklistFindingCode::MissingSchemeName,
                            "rti_details.scheme_name",
                        ));
                    }
                    if matches!(
                        (details.reporting_period_start, details.reporting_period_end),
                        (Some(start), Some(end)) if start > end
                    ) {
                        findings.push(blocking(
                            ChecklistFindingCode::InvalidReportingPeriod,
                            "rti_details.reporting_period",
                        ));
                    }
                    if let Some(reference) = &details.official_reference {
                        if !valid_reference(reference)
                            || (!reference_has_type(reference, ResourceType::Document)
                                && !reference_has_type(reference, ResourceType::LegalSource))
                        {
                            findings.push(blocking(
                                ChecklistFindingCode::InvalidOfficialReferenceType,
                                "rti_details.official_reference",
                            ));
                        }
                    }
                    if let Some(reference) = &details.fee_payment_proof {
                        if !valid_reference(reference)
                            || !reference_has_type(reference, ResourceType::Evidence)
                        {
                            findings.push(blocking(
                                ChecklistFindingCode::InvalidFeePaymentProofType,
                                "rti_details.fee_payment_proof",
                            ));
                        }
                    }
                } else {
                    findings.push(review(
                        ChecklistFindingCode::MissingSchemeName,
                        "rti_details",
                    ));
                }
                validate_procedure_reference(&self.procedure_ref, &mut findings);
            }
            CorrespondenceTrack::FollowUpOrAppeal => {
                validate_source_document(&self.source_document, &mut findings);
                if !has_text(self.trigger_date.as_deref()) {
                    findings.push(review(
                        ChecklistFindingCode::MissingTriggerDate,
                        "trigger_date",
                    ));
                }
                validate_procedure_reference(&self.procedure_ref, &mut findings);
            }
        }

        let mut seen_claim_ids = HashSet::new();
        for claim in &self.claims {
            if claim.claim_id.trim().is_empty() {
                findings.push(blocking(
                    ChecklistFindingCode::EmptyClaimId,
                    "claims.claim_id",
                ));
            } else if !seen_claim_ids.insert(claim.claim_id.as_str()) {
                findings.push(blocking(
                    ChecklistFindingCode::DuplicateClaimId,
                    "claims.claim_id",
                ));
            }

            if claim.statement.trim().is_empty() {
                findings.push(blocking(
                    ChecklistFindingCode::EmptyClaimStatement,
                    "claims.statement",
                ));
            }

            if claim.source_refs.is_empty() {
                findings.push(blocking(
                    ChecklistFindingCode::ClaimHasNoSource,
                    "claims.source_refs",
                ));
            }

            if claim
                .source_refs
                .iter()
                .any(|reference| !valid_reference(reference))
            {
                findings.push(blocking(
                    ChecklistFindingCode::InvalidClaimSourceReference,
                    "claims.source_refs",
                ));
            }

            match claim.proposition_type {
                PropositionType::VerifiedRule
                | PropositionType::OfficialProcedure
                | PropositionType::AuthoritativeInterpretation => {
                    if !claim
                        .source_refs
                        .iter()
                        .any(|reference| reference_has_type(reference, ResourceType::LegalSource))
                    {
                        findings.push(blocking(
                            ChecklistFindingCode::LegalClaimMissingLegalSource,
                            "claims.source_refs",
                        ));
                    }
                }
                PropositionType::UserFact => {
                    if !claim.source_refs.iter().any(|reference| {
                        reference_has_type(reference, ResourceType::Document)
                            || reference_has_type(reference, ResourceType::Evidence)
                    }) {
                        findings.push(blocking(
                            ChecklistFindingCode::UserFactMissingDocumentOrEvidence,
                            "claims.source_refs",
                        ));
                    }
                }
                PropositionType::Inference
                | PropositionType::Uncertainty
                | PropositionType::DisputedInterpretation
                | PropositionType::ProfessionalReviewRequired => {
                    findings.push(review(
                        ChecklistFindingCode::ClaimRequiresReview,
                        "claims.proposition_type",
                    ));
                }
            }

            if claim.verification_status != VerificationStatus::Verified {
                findings.push(review(
                    ChecklistFindingCode::ClaimRequiresReview,
                    "claims.verification_status",
                ));
            }
        }

        ChecklistResult::from_findings(findings)
    }

    /// Run the structural checklist and then resolve references against the
    /// existing domain registries. This is still not authorization to send.
    pub fn evaluate_with_registries(
        &self,
        registries: &mut CorrespondenceRegistries<'_>,
    ) -> ChecklistResult {
        let mut result = self.evaluate();
        let mut findings = std::mem::take(&mut result.findings);

        let selected_jurisdiction = self
            .jurisdiction
            .as_ref()
            .filter(|reference| {
                reference.resource_type == ResourceType::Jurisdiction
                    && valid_reference(reference)
            })
            .map(|reference| reference.id.clone());

        if let Some(reference) = &self.issuing_authority {
            if reference.resource_type == ResourceType::Authority && valid_reference(reference) {
                match registries.authorities.get_authority(&reference.id) {
                    None => push_finding(
                        &mut findings,
                        blocking(
                            ChecklistFindingCode::AuthorityNotFound,
                            "issuing_authority",
                        ),
                    ),
                    Some(authority) => {
                        if authority.status != AuthorityStatus::Active {
                            push_finding(
                                &mut findings,
                                blocking(
                                    ChecklistFindingCode::AuthorityInactive,
                                    "issuing_authority",
                                ),
                            );
                        }
                        if let Some(target_id) = &selected_jurisdiction {
                            if !jurisdiction_scope_covers(
                                registries.jurisdictions,
                                &authority.jurisdiction_id,
                                target_id,
                            ) {
                                push_finding(
                                    &mut findings,
                                    blocking(
                                        ChecklistFindingCode::AuthorityJurisdictionMismatch,
                                        "issuing_authority",
                                    ),
                                );
                            }
                        }
                    }
                }
            }
        }

        if let Some(reference) = &self.jurisdiction {
            if reference.resource_type == ResourceType::Jurisdiction && valid_reference(reference) {
                match registries.jurisdictions.get(&reference.id) {
                    None => push_finding(
                        &mut findings,
                        blocking(
                            ChecklistFindingCode::JurisdictionNotFound,
                            "jurisdiction",
                        ),
                    ),
                    Some(jurisdiction) if jurisdiction.status != JurisdictionStatus::Active => {
                        push_finding(
                            &mut findings,
                            blocking(
                                ChecklistFindingCode::JurisdictionInactive,
                                "jurisdiction",
                            ),
                        );
                    }
                    Some(_) => {}
                }
            }
        }

        if let Some(reference) = &self.source_document {
            resolve_reference(
                reference,
                "source_document",
                self,
                registries,
                &mut findings,
            );
            if reference.resource_type == ResourceType::Document {
                if let Some(document) = registries.documents.get_document(&reference.id) {
                    if !document.jurisdiction_refs.is_empty() {
                        let same_scope = selected_jurisdiction.as_ref().is_some_and(|target_id| {
                            document.jurisdiction_refs.iter().any(|document_scope| {
                                jurisdiction_scope_covers(
                                    registries.jurisdictions,
                                    document_scope,
                                    target_id,
                                ) || jurisdiction_scope_covers(
                                    registries.jurisdictions,
                                    target_id,
                                    document_scope,
                                )
                            })
                        });
                        if !same_scope {
                            push_finding(
                                &mut findings,
                                review(
                                    ChecklistFindingCode::DocumentJurisdictionMismatch,
                                    "source_document",
                                ),
                            );
                        }
                    }
                }
            }
        }

        if let Some(details) = &self.rti_details {
            if let Some(reference) = &details.official_reference {
                resolve_reference(
                    reference,
                    "rti_details.official_reference",
                    self,
                    registries,
                    &mut findings,
                );
            }
            if let Some(reference) = &details.fee_payment_proof {
                resolve_reference(
                    reference,
                    "rti_details.fee_payment_proof",
                    self,
                    registries,
                    &mut findings,
                );
            }
        }

        if let Some(reference) = &self.procedure_ref {
            resolve_reference(
                reference,
                "procedure_ref",
                self,
                registries,
                &mut findings,
            );
            if reference.resource_type == ResourceType::Procedure {
                if let Some(procedure) = registries.procedures.get_procedure(&reference.id) {
                    if let Some(authority_reference) = &self.issuing_authority {
                        if authority_reference.resource_type == ResourceType::Authority
                            && procedure.authority_id != authority_reference.id
                        {
                            push_finding(
                                &mut findings,
                                blocking(
                                    ChecklistFindingCode::ProcedureAuthorityMismatch,
                                    "procedure_ref",
                                ),
                            );
                        }
                    }
                    if let Some(target_id) = &selected_jurisdiction {
                        if !jurisdiction_scope_covers(
                            registries.jurisdictions,
                            &procedure.jurisdiction_id,
                            target_id,
                        ) {
                            push_finding(
                                &mut findings,
                                blocking(
                                    ChecklistFindingCode::ProcedureJurisdictionMismatch,
                                    "procedure_ref",
                                ),
                            );
                        }
                    }
                    for source_id in &procedure.source_refs {
                        let reference =
                            ResourceRef::new(ResourceType::LegalSource, source_id.clone());
                        if let Ok(reference) = reference {
                            resolve_reference(
                                &reference,
                                "procedure_ref.source_refs",
                                self,
                                registries,
                                &mut findings,
                            );
                        } else {
                            push_finding(
                                &mut findings,
                                blocking(
                                    ChecklistFindingCode::ProcedureSourceNotFound,
                                    "procedure_ref.source_refs",
                                ),
                            );
                        }
                    }
                }
            }
        }

        for claim in &self.claims {
            for reference in &claim.source_refs {
                resolve_reference(
                    reference,
                    "claims.source_refs",
                    self,
                    registries,
                    &mut findings,
                );
            }
        }

        result.findings = deduplicate_findings(findings);
        result.readiness = readiness_from_findings(&result.findings);
        result
    }
}


/// Registry references are resolved through their existing domain owner.
/// This context deliberately owns no copied registry or alternate persistence.
pub struct CorrespondenceRegistries<'a> {
    pub authorities: &'a AuthorityRegistry,
    pub jurisdictions: &'a JurisdictionRegistry,
    pub documents: &'a DocumentRegistry,
    pub legal_sources: &'a LegalSourceRegistry,
    pub procedures: &'a ProcedureRegistry,
    pub evidence: &'a mut dyn EvidenceTrustRepository,
}

fn resolve_reference(
    reference: &ResourceRef,
    field: &'static str,
    input: &CorrespondenceChecklistInput,
    registries: &mut CorrespondenceRegistries<'_>,
    findings: &mut Vec<ChecklistFinding>,
) {
    if !valid_reference(reference) {
        push_finding(
            findings,
            blocking(
                ChecklistFindingCode::InvalidClaimSourceReference,
                field,
            ),
        );
        return;
    }

    match reference.resource_type {
        ResourceType::Authority => match registries.authorities.get_authority(&reference.id) {
            None => push_finding(
                findings,
                blocking(ChecklistFindingCode::AuthorityNotFound, field),
            ),
            Some(authority) if authority.status != AuthorityStatus::Active => push_finding(
                findings,
                review(ChecklistFindingCode::AuthorityInactive, field),
            ),
            Some(_) => {}
        },
        ResourceType::Jurisdiction => match registries.jurisdictions.get(&reference.id) {
            None => push_finding(
                findings,
                blocking(ChecklistFindingCode::JurisdictionNotFound, field),
            ),
            Some(jurisdiction) if jurisdiction.status != JurisdictionStatus::Active => {
                push_finding(
                    findings,
                    blocking(ChecklistFindingCode::JurisdictionInactive, field),
                )
            }
            Some(_) => {}
        },
        ResourceType::Document => match registries.documents.get_document(&reference.id) {
            None => push_finding(
                findings,
                blocking(ChecklistFindingCode::DocumentReferenceNotFound, field),
            ),
            Some(document) => {
                match &document.status {
                    DocumentStatus::Deleted => push_finding(
                        findings,
                        blocking(ChecklistFindingCode::DocumentReferenceUnavailable, field),
                    ),
                    DocumentStatus::Draft | DocumentStatus::Unknown => push_finding(
                        findings,
                        review(ChecklistFindingCode::DocumentReferenceNeedsReview, field),
                    ),
                    DocumentStatus::Active
                    | DocumentStatus::Superseded
                    | DocumentStatus::Revoked
                    | DocumentStatus::Archived => {}
                }

                match registries.documents.get_version(&document.current_version_id) {
                    None => push_finding(
                        findings,
                        blocking(ChecklistFindingCode::DocumentReferenceUnavailable, field),
                    ),
                    Some(version) if version.document_id != document.document_id => {
                        push_finding(
                            findings,
                            blocking(ChecklistFindingCode::DocumentReferenceUnavailable, field),
                        );
                    }
                    Some(version) => match &version.integrity_status {
                        DocumentIntegrityStatus::Modified
                        | DocumentIntegrityStatus::Unavailable => push_finding(
                            findings,
                            blocking(ChecklistFindingCode::DocumentReferenceUnavailable, field),
                        ),
                        DocumentIntegrityStatus::Unverified => push_finding(
                            findings,
                            review(ChecklistFindingCode::DocumentReferenceNeedsReview, field),
                        ),
                        DocumentIntegrityStatus::Verified => {}
                    },
                }
            },
        },
        ResourceType::Evidence => match registries.evidence.get(&reference.id) {
            Ok(None) => push_finding(
                findings,
                blocking(ChecklistFindingCode::EvidenceNotFound, field),
            ),
            Err(_) => push_finding(
                findings,
                blocking(ChecklistFindingCode::EvidenceResolutionUnavailable, field),
            ),
            Ok(Some(evidence)) => {
                if evidence.validate().is_err() {
                    push_finding(
                        findings,
                        blocking(ChecklistFindingCode::EvidenceVerificationFailed, field),
                    );
                }
                if evidence.trust.integrity_status
                    == crate::evidence_trust::IntegrityStatus::IntegrityFailed
                {
                    push_finding(
                        findings,
                        blocking(ChecklistFindingCode::EvidenceVerificationFailed, field),
                    );
                } else if matches!(
                    evidence.trust.integrity_status,
                    crate::evidence_trust::IntegrityStatus::Unverified
                ) {
                    push_finding(
                        findings,
                        review(ChecklistFindingCode::EvidenceVerificationFailed, field),
                    );
                }
            }
        },
        ResourceType::LegalSource => {
            match registries.legal_sources.get(&reference.id) {
                None => push_finding(
                    findings,
                    blocking(ChecklistFindingCode::LegalSourceNotFound, field),
                ),
                Some(source) => {
                    if source.verification_status != VerificationStatus::Verified {
                        push_finding(
                            findings,
                            review(ChecklistFindingCode::LegalSourceUnverified, field),
                        );
                    }

                    let target_jurisdiction = input
                        .jurisdiction
                        .as_ref()
                        .filter(|jurisdiction| {
                            jurisdiction.resource_type == ResourceType::Jurisdiction
                                && valid_reference(jurisdiction)
                        })
                        .map(|jurisdiction| jurisdiction.id.clone());

                    if registries.jurisdictions.get(&source.jurisdiction).is_none() {
                        push_finding(
                            findings,
                            blocking(
                                ChecklistFindingCode::LegalSourceJurisdictionNotFound,
                                field,
                            ),
                        );
                    } else if let Some(target_id) = target_jurisdiction {
                        if !jurisdiction_scope_covers(
                            registries.jurisdictions,
                            &source.jurisdiction,
                            &target_id,
                        ) {
                            push_finding(
                                findings,
                                blocking(
                                    ChecklistFindingCode::LegalSourceJurisdictionMismatch,
                                    field,
                                ),
                            );
                        }
                    }

                    match input.legal_as_of.as_deref() {
                        None => push_finding(
                            findings,
                            review(
                                ChecklistFindingCode::MissingLegalApplicabilityDate,
                                "legal_as_of",
                            ),
                        ),
                        Some(as_of) if !is_canonical_utc_instant(as_of) => push_finding(
                            findings,
                            review(
                                ChecklistFindingCode::LegalSourceEffectiveDateUnparseable,
                                "legal_as_of",
                            ),
                        ),
                        Some(as_of) => {
                            if !is_canonical_utc_instant(&source.effective_from)
                                || source
                                    .effective_to
                                    .as_deref()
                                    .is_some_and(|end| !is_canonical_utc_instant(end))
                            {
                                push_finding(
                                    findings,
                                    review(
                                        ChecklistFindingCode::LegalSourceEffectiveDateUnparseable,
                                        field,
                                    ),
                                );
                            } else if as_of < source.effective_from.as_str()
                                || source
                                    .effective_to
                                    .as_deref()
                                    .is_some_and(|end| as_of > end)
                            {
                                push_finding(
                                    findings,
                                    blocking(
                                        ChecklistFindingCode::LegalSourceNotEffective,
                                        field,
                                    ),
                                );
                            }
                        }
                    }
                }
            }
        }
        ResourceType::Procedure => match registries.procedures.get_procedure(&reference.id) {
            None => push_finding(
                findings,
                blocking(ChecklistFindingCode::ProcedureNotFound, field),
            ),
            Some(procedure) if procedure.status != ProcedureStatus::Active => push_finding(
                findings,
                blocking(ChecklistFindingCode::ProcedureInactive, field),
            ),
            Some(_) => {}
        },
        _ => push_finding(
            findings,
            review(ChecklistFindingCode::UnsupportedSourceReferenceType, field),
        ),
    }
}

fn jurisdiction_scope_covers(
    registry: &JurisdictionRegistry,
    scope_id: &str,
    target_id: &str,
) -> bool {
    let mut current = target_id.to_owned();
    let mut visited = HashSet::new();

    loop {
        if current == scope_id {
            return true;
        }
        if !visited.insert(current.clone()) {
            return false;
        }
        let Some(jurisdiction) = registry.get(&current) else {
            return false;
        };
        let Some(parent_id) = &jurisdiction.parent_id else {
            return false;
        };
        current = parent_id.clone();
    }
}

fn is_canonical_utc_instant(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 20
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
        || bytes[19] != b'Z'
    {
        return false;
    }

    let digit_ranges = [(0, 4), (5, 7), (8, 10), (11, 13), (14, 16), (17, 19)];
    if digit_ranges
        .iter()
        .any(|(start, end)| !bytes[*start..*end].iter().all(u8::is_ascii_digit))
    {
        return false;
    }

    let Ok(year) = value[0..4].parse::<i32>() else {
        return false;
    };
    let Ok(month) = value[5..7].parse::<u8>() else {
        return false;
    };
    let Ok(day) = value[8..10].parse::<u8>() else {
        return false;
    };
    let Ok(hour) = value[11..13].parse::<u8>() else {
        return false;
    };
    let Ok(minute) = value[14..16].parse::<u8>() else {
        return false;
    };
    let Ok(second) = value[17..19].parse::<u8>() else {
        return false;
    };

    CivilDate::new(year, month, day).is_ok()
        && hour < 24
        && minute < 60
        && second < 60
}

fn push_finding(findings: &mut Vec<ChecklistFinding>, finding: ChecklistFinding) {
    if !findings.contains(&finding) {
        findings.push(finding);
    }
}

fn deduplicate_findings(findings: Vec<ChecklistFinding>) -> Vec<ChecklistFinding> {
    let mut unique = Vec::new();
    for finding in findings {
        push_finding(&mut unique, finding);
    }
    unique
}

fn readiness_from_findings(findings: &[ChecklistFinding]) -> ChecklistReadiness {
    if findings
        .iter()
        .any(|finding| finding.severity == FindingSeverity::Blocking)
    {
        ChecklistReadiness::Blocked
    } else if !findings.is_empty() {
        ChecklistReadiness::ReviewRequired
    } else {
        ChecklistReadiness::StructurallyComplete
    }
}

fn validate_source_document(
    source_document: &Option<ResourceRef>,
    findings: &mut Vec<ChecklistFinding>,
) {
    match source_document {
        None => findings.push(blocking(
            ChecklistFindingCode::MissingNoticeDocument,
            "source_document",
        )),
        Some(reference)
            if !valid_reference(reference)
                || (!reference_has_type(reference, ResourceType::Document)
                    && !reference_has_type(reference, ResourceType::Evidence)) =>
        {
            findings.push(blocking(
                ChecklistFindingCode::InvalidNoticeDocumentType,
                "source_document",
            ));
        }
        Some(_) => {}
    }
}

fn validate_procedure_reference(
    procedure_ref: &Option<ResourceRef>,
    findings: &mut Vec<ChecklistFinding>,
) {
    match procedure_ref {
        None => findings.push(review(
            ChecklistFindingCode::MissingProcedureReference,
            "procedure_ref",
        )),
        Some(reference)
            if !valid_reference(reference)
                || !reference_has_type(reference, ResourceType::Procedure) =>
        {
            findings.push(blocking(
                ChecklistFindingCode::InvalidProcedureReferenceType,
                "procedure_ref",
            ));
        }
        Some(_) => {}
    }
}

fn has_text(value: Option<&str>) -> bool {
    value.is_some_and(|text| !text.trim().is_empty())
}

fn valid_reference(reference: &ResourceRef) -> bool {
    !reference.id.trim().is_empty()
}

fn reference_has_type(reference: &ResourceRef, expected: ResourceType) -> bool {
    reference.resource_type == expected
}

fn blocking(code: ChecklistFindingCode, field: &'static str) -> ChecklistFinding {
    ChecklistFinding {
        code,
        severity: FindingSeverity::Blocking,
        field: field.into(),
    }
}

fn review(code: ChecklistFindingCode, field: &'static str) -> ChecklistFinding {
    ChecklistFinding {
        code,
        severity: FindingSeverity::Review,
        field: field.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reference(resource_type: ResourceType, id: &str) -> ResourceRef {
        ResourceRef {
            resource_type,
            id: id.into(),
        }
    }

    fn record_request(description: &str, kind: RtiRequestKind) -> RtiRequestItem {
        RtiRequestItem {
            description: description.into(),
            kind,
            period_start: None,
            period_end: None,
            includes_personal_or_sensitive_data: false,
        }
    }

    fn base(track: CorrespondenceTrack) -> CorrespondenceChecklistInput {
        CorrespondenceChecklistInput {
            track,
            issuing_authority: Some(reference(ResourceType::Authority, "authority-1")),
            jurisdiction: Some(reference(ResourceType::Jurisdiction, "jurisdiction-1")),
            subject: Some("Administrative notice".into()),
            source_document: None,
            trigger_date: None,
            legal_as_of: None,
            recipient_designation: None,
            requested_records: Vec::new(),
            rti_details: None,
            procedure_ref: None,
            claims: Vec::new(),
        }
    }

    fn fixture_registries() -> (
        AuthorityRegistry,
        JurisdictionRegistry,
        DocumentRegistry,
        LegalSourceRegistry,
        ProcedureRegistry,
    ) {
        use crate::authority::{Authority, AuthorityType};
        use crate::document::{Document, DocumentStatus};
        use crate::jurisdiction::{Jurisdiction, JurisdictionType};
        use crate::legal_source::{LegalSource, SourceType};
        use crate::procedure::{Procedure, ProcedureStatus};

        let mut jurisdictions = JurisdictionRegistry::default();
        jurisdictions
            .insert(Jurisdiction {
                jurisdiction_id: "jurisdiction-1".into(),
                jurisdiction_type: JurisdictionType::Territorial,
                name: "Example jurisdiction".into(),
                parent_id: None,
                status: JurisdictionStatus::Active,
            })
            .unwrap();

        let mut authorities = AuthorityRegistry::default();
        authorities
            .insert_authority(Authority {
                authority_id: "authority-1".into(),
                name: "Example public authority".into(),
                authority_type: AuthorityType::GovernmentBody,
                jurisdiction_id: "jurisdiction-1".into(),
                status: AuthorityStatus::Active,
            })
            .unwrap();

        let mut documents = DocumentRegistry::default();
        documents
            .insert_document_with_initial_version(
                Document {
                    document_id: "notice-1".into(),
                    schema_version: 1,
                    document_type: "notice".into(),
                    status: DocumentStatus::Active,
                    title: "Example notice".into(),
                    issuer_party_id: None,
                    recipient_party_refs: Vec::new(),
                    case_refs: Vec::new(),
                    incident_refs: Vec::new(),
                    jurisdiction_refs: vec!["jurisdiction-1".into()],
                    authority_ref: Some("authority-1".into()),
                    current_version_id: "notice-version-1".into(),
                    provenance_ref: None,
                    privacy_classification: "sensitive".into(),
                    retention_policy_ref: None,
                    created_at: "2026-01-01T00:00:00Z".into(),
                    updated_at: "2026-01-01T00:00:00Z".into(),
                },
                crate::document::DocumentVersion {
                    document_version_id: "notice-version-1".into(),
                    document_id: "notice-1".into(),
                    schema_version: 1,
                    version_number: 1,
                    media_type: "application/pdf".into(),
                    content_ref: "notice-blob-1".into(),
                    content_hash: "sha256:example".into(),
                    byte_length: Some(100),
                    captured_at: Some("2026-01-01T00:00:00Z".into()),
                    created_by: "party-1".into(),
                    source_ref: None,
                    provenance_ref: None,
                    integrity_status: DocumentIntegrityStatus::Verified,
                    supersedes_version_id: None,
                    language: Some("en".into()),
                    created_at: "2026-01-01T00:00:00Z".into(),
                },
            )
            .unwrap();

        let mut legal_sources = LegalSourceRegistry::default();
        legal_sources
            .insert(LegalSource {
                source_id: "law-1".into(),
                source_type: SourceType::ConstitutionOrLegislation,
                title: "Example law".into(),
                issuing_authority: "authority-1".into(),
                jurisdiction: "jurisdiction-1".into(),
                citation: "Section 1".into(),
                published_at: "2025-12-01T00:00:00Z".into(),
                effective_from: "2026-01-01T00:00:00Z".into(),
                effective_to: None,
                version: "1".into(),
                retrieved_at: "2026-01-02T00:00:00Z".into(),
                verification_status: VerificationStatus::Verified,
                supersedes: None,
            })
            .unwrap();

        let mut procedures = ProcedureRegistry::default();
        procedures
            .insert(
                Procedure {
                    procedure_id: "procedure-1".into(),
                    name: "Example RTI procedure".into(),
                    jurisdiction_id: "jurisdiction-1".into(),
                    authority_id: "authority-1".into(),
                    status: ProcedureStatus::Active,
                    step_ids: Vec::new(),
                    source_refs: vec!["law-1".into()],
                },
                Vec::new(),
            )
            .unwrap();

        (
            authorities,
            jurisdictions,
            documents,
            legal_sources,
            procedures,
        )
    }

    #[test]
    fn notice_reply_requires_a_source_document() {
        let mut input = base(CorrespondenceTrack::NoticeReply);
        input.trigger_date = Some("2026-10-01".into());

        let result = input.evaluate();

        assert_eq!(result.readiness, ChecklistReadiness::Blocked);
        assert!(result.findings.iter().any(|finding| {
            finding.code == ChecklistFindingCode::MissingNoticeDocument
                && finding.severity == FindingSeverity::Blocking
        }));
    }

    #[test]
    fn complete_notice_reply_is_only_structurally_complete() {
        let mut input = base(CorrespondenceTrack::NoticeReply);
        input.source_document = Some(reference(ResourceType::Document, "notice-1"));
        input.trigger_date = Some("2026-10-01".into());

        assert_eq!(
            input.evaluate().readiness,
            ChecklistReadiness::StructurallyComplete
        );
    }

    #[test]
    fn rti_application_requires_recipient_and_identifiable_records() {
        let mut input = base(CorrespondenceTrack::RtiApplication);
        input.requested_records = vec![record_request(
            "Copy of the applicable order",
            RtiRequestKind::CertifiedCopy,
        )];
        input.rti_details = Some(RtiApplicationDetails {
            scheme_name: Some("Example scheme".into()),
            official_reference: None,
            reporting_period_start: None,
            reporting_period_end: None,
            fee_payment_proof: None,
        });
        input.procedure_ref = Some(reference(ResourceType::Procedure, "rti-procedure-1"));

        let result = input.evaluate();

        assert_eq!(result.readiness, ChecklistReadiness::Blocked);
        assert!(result
            .findings
            .iter()
            .any(|finding| { finding.code == ChecklistFindingCode::MissingRecipientDesignation }));
        input.recipient_designation = Some("CPIO".into());
        assert_eq!(
            input.evaluate().readiness,
            ChecklistReadiness::StructurallyComplete
        );
    }

    #[test]
    fn missing_procedure_for_rti_requires_review_not_automatic_deadline_claims() {
        let mut input = base(CorrespondenceTrack::RtiApplication);
        input.recipient_designation = Some("SPIO".into());
        input.requested_records = vec![record_request(
            "Inspection of the existing file",
            RtiRequestKind::Inspection,
        )];
        input.rti_details = Some(RtiApplicationDetails {
            scheme_name: Some("Example scheme".into()),
            official_reference: None,
            reporting_period_start: None,
            reporting_period_end: None,
            fee_payment_proof: None,
        });

        let result = input.evaluate();

        assert_eq!(result.readiness, ChecklistReadiness::ReviewRequired);
        assert!(result.findings.iter().any(|finding| {
            finding.code == ChecklistFindingCode::MissingProcedureReference
                && finding.severity == FindingSeverity::Review
        }));
    }

    #[test]
    fn legal_rule_requires_a_typed_legal_source_reference() {
        let mut input = base(CorrespondenceTrack::NoticeReply);
        input.source_document = Some(reference(ResourceType::Document, "notice-1"));
        input.trigger_date = Some("2026-10-01".into());
        input.claims.push(CorrespondenceClaim {
            claim_id: "claim-1".into(),
            statement: "A statutory rule applies.".into(),
            proposition_type: PropositionType::VerifiedRule,
            verification_status: VerificationStatus::Verified,
            source_refs: vec![reference(ResourceType::Document, "some-document")],
        });

        let result = input.evaluate();

        assert_eq!(result.readiness, ChecklistReadiness::Blocked);
        assert!(result
            .findings
            .iter()
            .any(|finding| { finding.code == ChecklistFindingCode::LegalClaimMissingLegalSource }));
    }

    #[test]
    fn user_fact_requires_document_or_evidence_source_and_verified_claims_still_need_review_if_unverified(
    ) {
        let mut input = base(CorrespondenceTrack::NoticeReply);
        input.source_document = Some(reference(ResourceType::Document, "notice-1"));
        input.trigger_date = Some("2026-10-01".into());
        input.claims.push(CorrespondenceClaim {
            claim_id: "fact-1".into(),
            statement: "The notice was received.".into(),
            proposition_type: PropositionType::UserFact,
            verification_status: VerificationStatus::Unverified,
            source_refs: vec![reference(ResourceType::LegalSource, "law-1")],
        });

        let result = input.evaluate();

        assert_eq!(result.readiness, ChecklistReadiness::Blocked);
        assert!(result.findings.iter().any(|finding| {
            finding.code == ChecklistFindingCode::UserFactMissingDocumentOrEvidence
        }));
        assert!(result.findings.iter().any(|finding| {
            finding.code == ChecklistFindingCode::ClaimRequiresReview
                && finding.field == "claims.verification_status"
        }));
    }

    #[test]
    fn inference_is_never_treated_as_a_verified_rule() {
        let mut input = base(CorrespondenceTrack::NoticeReply);
        input.source_document = Some(reference(ResourceType::Document, "notice-1"));
        input.trigger_date = Some("2026-10-01".into());
        input.claims.push(CorrespondenceClaim {
            claim_id: "inference-1".into(),
            statement: "The calculation may use an outdated rate.".into(),
            proposition_type: PropositionType::Inference,
            verification_status: VerificationStatus::Verified,
            source_refs: vec![reference(ResourceType::Evidence, "calculation-1")],
        });

        let result = input.evaluate();

        assert_eq!(result.readiness, ChecklistReadiness::ReviewRequired);
        assert!(result.findings.iter().any(|finding| {
            finding.code == ChecklistFindingCode::ClaimRequiresReview
                && finding.field == "claims.proposition_type"
        }));
    }

    #[test]
    fn registry_preflight_blocks_when_source_document_is_not_registered() {
        let (authorities, jurisdictions, documents, legal_sources, procedures) =
            fixture_registries();
        let mut evidence = crate::evidence_repository::InMemoryEvidenceTrustRepository::default();
        let mut registries = CorrespondenceRegistries {
            authorities: &authorities,
            jurisdictions: &jurisdictions,
            documents: &documents,
            legal_sources: &legal_sources,
            procedures: &procedures,
            evidence: &mut evidence,
        };
        let mut input = base(CorrespondenceTrack::NoticeReply);
        input.source_document = Some(reference(ResourceType::Document, "missing-notice"));
        input.trigger_date = Some("2026-01-02".into());

        let result = input.evaluate_with_registries(&mut registries);

        assert_eq!(result.readiness, ChecklistReadiness::Blocked);
        assert!(result.findings.iter().any(|finding| {
            finding.code == ChecklistFindingCode::DocumentReferenceNotFound
                && finding.field == "source_document"
        }));
    }

    #[test]
    fn registry_preflight_blocks_legal_claim_outside_effective_interval() {
        let (authorities, jurisdictions, documents, legal_sources, procedures) =
            fixture_registries();
        let mut evidence = crate::evidence_repository::InMemoryEvidenceTrustRepository::default();
        let mut registries = CorrespondenceRegistries {
            authorities: &authorities,
            jurisdictions: &jurisdictions,
            documents: &documents,
            legal_sources: &legal_sources,
            procedures: &procedures,
            evidence: &mut evidence,
        };
        let mut input = base(CorrespondenceTrack::NoticeReply);
        input.source_document = Some(reference(ResourceType::Document, "notice-1"));
        input.trigger_date = Some("2025-12-31".into());
        input.legal_as_of = Some("2025-12-31T00:00:00Z".into());
        input.claims.push(CorrespondenceClaim {
            claim_id: "legal-claim-1".into(),
            statement: "The example law applied on the relevant date.".into(),
            proposition_type: PropositionType::VerifiedRule,
            verification_status: VerificationStatus::Verified,
            source_refs: vec![reference(ResourceType::LegalSource, "law-1")],
        });

        let result = input.evaluate_with_registries(&mut registries);

        assert_eq!(result.readiness, ChecklistReadiness::Blocked);
        assert!(result.findings.iter().any(|finding| {
            finding.code == ChecklistFindingCode::LegalSourceNotEffective
                && finding.field == "claims.source_refs"
        }));
    }

    #[test]
    fn registry_preflight_accepts_a_registered_in_scope_effective_legal_source() {
        let (authorities, jurisdictions, documents, legal_sources, procedures) =
            fixture_registries();
        let mut evidence = crate::evidence_repository::InMemoryEvidenceTrustRepository::default();
        let mut registries = CorrespondenceRegistries {
            authorities: &authorities,
            jurisdictions: &jurisdictions,
            documents: &documents,
            legal_sources: &legal_sources,
            procedures: &procedures,
            evidence: &mut evidence,
        };
        let mut input = base(CorrespondenceTrack::NoticeReply);
        input.source_document = Some(reference(ResourceType::Document, "notice-1"));
        input.trigger_date = Some("2026-01-02".into());
        input.legal_as_of = Some("2026-01-02T00:00:00Z".into());
        input.claims.push(CorrespondenceClaim {
            claim_id: "legal-claim-1".into(),
            statement: "The example law applied on the relevant date.".into(),
            proposition_type: PropositionType::VerifiedRule,
            verification_status: VerificationStatus::Verified,
            source_refs: vec![reference(ResourceType::LegalSource, "law-1")],
        });

        let result = input.evaluate_with_registries(&mut registries);

        assert_eq!(result.readiness, ChecklistReadiness::StructurallyComplete);
        assert!(result.findings.is_empty());
    }

    #[test]
    fn rti_request_flags_sensitive_data_and_blocks_reversed_reporting_period() {
        let mut input = base(CorrespondenceTrack::RtiApplication);
        input.recipient_designation = Some("SPIO".into());
        input.requested_records = vec![RtiRequestItem {
            description: "Beneficiary list including contact details".into(),
            kind: RtiRequestKind::ExistingRecord,
            period_start: Some(CivilDate::new(2026, 2, 1).unwrap()),
            period_end: Some(CivilDate::new(2026, 1, 1).unwrap()),
            includes_personal_or_sensitive_data: true,
        }];
        input.rti_details = Some(RtiApplicationDetails {
            scheme_name: Some("Example scheme".into()),
            official_reference: None,
            reporting_period_start: None,
            reporting_period_end: None,
            fee_payment_proof: None,
        });
        input.procedure_ref = Some(reference(ResourceType::Procedure, "procedure-1"));

        let result = input.evaluate();

        assert_eq!(result.readiness, ChecklistReadiness::Blocked);
        assert!(result.findings.iter().any(|finding| {
            finding.code == ChecklistFindingCode::InvalidReportingPeriod
        }));
        assert!(result.findings.iter().any(|finding| {
            finding.code == ChecklistFindingCode::PersonalInformationDisclosureReviewRequired
                && finding.severity == FindingSeverity::Review
        }));
    }

    #[test]
    fn new_analysis_request_is_reviewed_instead_of_silently_treated_as_existing_record() {
        let mut input = base(CorrespondenceTrack::RtiApplication);
        input.recipient_designation = Some("CPIO".into());
        input.requested_records = vec![record_request(
            "Prepare a fresh cost-benefit analysis for the scheme",
            RtiRequestKind::ExplanationOrNewAnalysis,
        )];
        input.rti_details = Some(RtiApplicationDetails {
            scheme_name: Some("Example scheme".into()),
            official_reference: None,
            reporting_period_start: None,
            reporting_period_end: None,
            fee_payment_proof: None,
        });
        input.procedure_ref = Some(reference(ResourceType::Procedure, "procedure-1"));

        let result = input.evaluate();

        assert_eq!(result.readiness, ChecklistReadiness::ReviewRequired);
        assert!(result.findings.iter().any(|finding| {
            finding.code == ChecklistFindingCode::RequestSeeksNewAnalysisOrOpinion
                && finding.severity == FindingSeverity::Review
        }));
    }

    #[test]
    fn follow_up_requires_the_prior_document_and_procedure_reference_shape() {
        let mut input = base(CorrespondenceTrack::FollowUpOrAppeal);
        input.trigger_date = Some("2026-10-01".into());
        input.procedure_ref = Some(reference(ResourceType::Procedure, "procedure-1"));

        assert_eq!(input.evaluate().readiness, ChecklistReadiness::Blocked);
        input.source_document = Some(reference(ResourceType::Evidence, "prior-response-1"));
        assert_eq!(
            input.evaluate().readiness,
            ChecklistReadiness::StructurallyComplete
        );
    }
}
