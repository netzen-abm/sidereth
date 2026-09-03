use serde::{Deserialize, Serialize};

use crate::Id;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SourceType {
    ConstitutionOrLegislation,
    RuleOrRegulation,
    NotificationOrderOrCircular,
    OfficialProcedure,
    JudicialDecision,
    OfficialGuidance,
    SecondarySource,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VerificationStatus {
    Verified,
    Unverified,
    Disputed,
    ReviewRequired,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PropositionType {
    VerifiedRule,
    OfficialProcedure,
    AuthoritativeInterpretation,
    UserFact,
    Inference,
    Uncertainty,
    DisputedInterpretation,
    ProfessionalReviewRequired,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LegalSource {
    pub source_id: Id,
    pub source_type: SourceType,
    pub title: String,
    pub issuing_authority: Id,
    pub jurisdiction: Id,
    pub citation: String,
    pub published_at: String,
    pub effective_from: String,
    pub effective_to: Option<String>,
    pub version: String,
    pub retrieved_at: String,
    pub verification_status: VerificationStatus,
    pub supersedes: Option<Id>,
}

impl LegalSource {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.source_id.is_empty() {
            return Err("source id is required");
        }
        if self.title.is_empty() {
            return Err("source title is required");
        }
        if self.issuing_authority.is_empty() {
            return Err("issuing authority is required");
        }
        if self.jurisdiction.is_empty() {
            return Err("jurisdiction is required");
        }
        if self.citation.is_empty() {
            return Err("source citation is required");
        }
        if self.published_at.is_empty() {
            return Err("publication time is required");
        }
        if self.effective_from.is_empty() {
            return Err("effective from is required");
        }
        if self.version.is_empty() {
            return Err("source version is required");
        }
        if self.retrieved_at.is_empty() {
            return Err("retrieval time is required");
        }
        if self.supersedes.as_ref() == Some(&self.source_id) {
            return Err("source cannot supersede itself");
        }
        if let Some(end) = &self.effective_to {
            if end < &self.effective_from {
                return Err("effective end precedes effective start");
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LegalProposition {
    pub proposition_id: Id,
    pub schema_version: u32,
    pub proposition_type: PropositionType,
    pub statement: String,
    pub source_refs: Vec<Id>,
    pub verification_status: VerificationStatus,
    pub confidence_basis_points: u16,
}

impl LegalProposition {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.proposition_id.is_empty() {
            return Err("proposition id is required");
        }
        if self.schema_version == 0 {
            return Err("schema version must be positive");
        }
        if self.statement.is_empty() {
            return Err("proposition statement is required");
        }
        if self.source_refs.is_empty() {
            return Err("proposition requires a source reference");
        }
        if self.source_refs.iter().any(Id::is_empty) {
            return Err("proposition source reference cannot be empty");
        }
        if self.confidence_basis_points > 10_000 {
            return Err("confidence must be between 0 and 10000 basis points");
        }
        Ok(())
    }

    pub fn canonical_source_refs(&mut self) {
        self.source_refs.sort();
        self.source_refs.dedup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source() -> LegalSource {
        LegalSource {
            source_id: "law-1".into(),
            source_type: SourceType::ConstitutionOrLegislation,
            title: "Example Act".into(),
            issuing_authority: "legislature".into(),
            jurisdiction: "IN".into(),
            citation: "s. 1".into(),
            published_at: "2026-01-01T00:00:00Z".into(),
            effective_from: "2026-02-01T00:00:00Z".into(),
            effective_to: None,
            version: "1".into(),
            retrieved_at: "2026-09-03T00:00:00Z".into(),
            verification_status: VerificationStatus::Verified,
            supersedes: None,
        }
    }

    fn proposition() -> LegalProposition {
        LegalProposition {
            proposition_id: "prop-1".into(),
            schema_version: 1,
            proposition_type: PropositionType::VerifiedRule,
            statement: "A requirement applies.".into(),
            source_refs: vec!["law-1".into()],
            verification_status: VerificationStatus::Verified,
            confidence_basis_points: 10_000,
        }
    }

    #[test]
    fn valid_source_passes_validation() {
        assert!(source().validate().is_ok());
    }

    #[test]
    fn self_supersession_is_rejected() {
        let mut value = source();
        value.supersedes = Some(value.source_id.clone());
        assert_eq!(
            value.validate(),
            Err("source cannot supersede itself")
        );
    }

    #[test]
    fn invalid_effective_interval_is_rejected() {
        let mut value = source();
        value.effective_to = Some("2026-01-31T23:59:59Z".into());
        assert_eq!(
            value.validate(),
            Err("effective end precedes effective start")
        );
    }

    #[test]
    fn proposition_requires_source() {
        let mut value = proposition();
        value.source_refs.clear();
        assert_eq!(
            value.validate(),
            Err("proposition requires a source reference")
        );
    }

    #[test]
    fn confidence_is_bounded() {
        let mut value = proposition();
        value.confidence_basis_points = 10_001;
        assert_eq!(
            value.validate(),
            Err("confidence must be between 0 and 10000 basis points")
        );
    }

    #[test]
    fn source_refs_have_deterministic_order() {
        let mut value = proposition();
        value.source_refs = vec!["law-2".into(), "law-1".into(), "law-2".into()];
        value.canonical_source_refs();
        assert_eq!(value.source_refs, vec!["law-1", "law-2"]);
    }

    #[test]
    fn provenance_types_remain_distinct() {
        assert_ne!(
            PropositionType::UserFact,
            PropositionType::VerifiedRule
        );
        assert_ne!(
            PropositionType::Inference,
            PropositionType::AuthoritativeInterpretation
        );
        assert_ne!(
            PropositionType::Uncertainty,
            PropositionType::DisputedInterpretation
        );
    }
}
