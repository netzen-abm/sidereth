//! Deterministic preflight checklist for administrative correspondence drafts.
//!
//! This module validates route-specific input shape and claim-source references.
//! It does not verify that a referenced record exists, that a legal source is
//! current or operative, that a deadline is correctly calculated, or that a
//! legal proposition is correct. A structurally complete result is never
//! authorization to send a document; source resolution and human review remain
//! separate gates.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::{PropositionType, ResourceRef, ResourceType, VerificationStatus};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CorrespondenceTrack {
    NoticeReply,
    RtiApplication,
    FollowUpOrAppeal,
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
    /// For an RTI application, normally the CPIO/SPIO or other designated recipient role.
    pub recipient_designation: Option<String>,
    /// Identifiable existing records sought; required for the RTI track.
    pub requested_records: Vec<String>,
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
            findings.push(blocking(
                ChecklistFindingCode::MissingSubject,
                "subject",
            ));
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
                if self.requested_records.iter().any(|record| record.trim().is_empty()) {
                    findings.push(blocking(
                        ChecklistFindingCode::EmptyRequestedRecord,
                        "requested_records",
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

            if claim.source_refs.iter().any(|reference| !valid_reference(reference)) {
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

    fn base(track: CorrespondenceTrack) -> CorrespondenceChecklistInput {
        CorrespondenceChecklistInput {
            track,
            issuing_authority: Some(reference(ResourceType::Authority, "authority-1")),
            jurisdiction: Some(reference(ResourceType::Jurisdiction, "jurisdiction-1")),
            subject: Some("Administrative notice".into()),
            source_document: None,
            trigger_date: None,
            recipient_designation: None,
            requested_records: Vec::new(),
            procedure_ref: None,
            claims: Vec::new(),
        }
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
        input.requested_records = vec!["Copy of the applicable order".into()];
        input.procedure_ref = Some(reference(ResourceType::Procedure, "rti-procedure-1"));

        let result = input.evaluate();

        assert_eq!(result.readiness, ChecklistReadiness::Blocked);
        assert!(result.findings.iter().any(|finding| {
            finding.code == ChecklistFindingCode::MissingRecipientDesignation
        }));
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
        input.requested_records = vec!["Inspection of the existing file".into()];

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
        assert!(result.findings.iter().any(|finding| {
            finding.code == ChecklistFindingCode::LegalClaimMissingLegalSource
        }));
    }

    #[test]
    fn user_fact_requires_document_or_evidence_source_and_verified_claims_still_need_review_if_unverified() {
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
