use crate::{EvidenceOriginal, Id, Provenance, ResourceRef, ResourceType};
use serde::{Deserialize, Serialize};

/// Canonical result of verifying a bounded property of an evidence item.
///
/// Verification is distinct from integrity and provenance: it records what was
/// checked and the resulting status. It does not establish legal authority,
/// truth beyond the stated claim, or authorization.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceVerificationStatus {
    Verified,
    Failed,
    Indeterminate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceVerificationMethod {
    HashComparison,
    SignatureVerification,
    HardwareAttestation,
    ProvenanceConsistency,
    ManualInspection,
    ProviderVerification,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceVerification {
    pub verification_id: Id,
    pub evidence_ref: ResourceRef,
    pub claim: String,
    pub method: EvidenceVerificationMethod,
    pub status: EvidenceVerificationStatus,
    pub verifier_ref: Option<ResourceRef>,
    pub provenance_ref: Option<Id>,
    pub verified_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceVerificationError {
    InvalidVerification,
    EvidenceMismatch,
    ProvenanceMismatch,
}

impl EvidenceVerification {
    pub fn validate(&self) -> Result<(), EvidenceVerificationError> {
        if self.verification_id.is_empty()
            || self.evidence_ref.id.is_empty()
            || self.evidence_ref.resource_type != ResourceType::Evidence
            || self.claim.trim().is_empty()
            || self.verified_at.trim().is_empty()
        {
            return Err(EvidenceVerificationError::InvalidVerification);
        }
        if let Some(verifier) = &self.verifier_ref {
            if verifier.id.is_empty() {
                return Err(EvidenceVerificationError::InvalidVerification);
            }
        }
        Ok(())
    }

    pub fn verify_against_original(
        &self,
        evidence: &EvidenceOriginal,
    ) -> Result<(), EvidenceVerificationError> {
        self.validate()?;
        if self.evidence_ref.id != evidence.evidence_id {
            return Err(EvidenceVerificationError::EvidenceMismatch);
        }
        Ok(())
    }

    pub fn verify_against_provenance(
        &self,
        provenance: &Provenance,
    ) -> Result<(), EvidenceVerificationError> {
        self.validate()?;
        let evidence_ref = ResourceRef::new(ResourceType::Evidence, self.evidence_ref.id.clone())
            .map_err(|_| EvidenceVerificationError::InvalidVerification)?;
        if !provenance.input_refs.contains(&evidence_ref)
            && !provenance.source_refs.contains(&evidence_ref)
        {
            return Err(EvidenceVerificationError::ProvenanceMismatch);
        }
        if let Some(verifier) = &self.verifier_ref {
            if provenance.actor_ref.as_ref() != Some(verifier) {
                return Err(EvidenceVerificationError::ProvenanceMismatch);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn verification() -> EvidenceVerification {
        EvidenceVerification {
            verification_id: "verification-1".into(),
            evidence_ref: ResourceRef::new(ResourceType::Evidence, "evidence-1").unwrap(),
            claim: "content hash matches recorded original hash".into(),
            method: EvidenceVerificationMethod::HashComparison,
            status: EvidenceVerificationStatus::Verified,
            verifier_ref: Some(ResourceRef::new(ResourceType::Party, "verifier-1").unwrap()),
            provenance_ref: Some("prov-1".into()),
            verified_at: "2026-10-07T08:00:00Z".into(),
        }
    }

    fn original() -> EvidenceOriginal {
        EvidenceOriginal {
            evidence_id: "evidence-1".into(),
            schema_version: 1,
            case_id: Some("case-1".into()),
            incident_id: None,
            captured_at: "2026-10-07T07:00:00Z".into(),
            captured_by: "user-1".into(),
            media_type: "text/plain".into(),
            content_hash: "hash-1".into(),
            storage_ref: "object-1".into(),
        }
    }

    fn provenance() -> Provenance {
        Provenance {
            provenance_id: "prov-1".into(),
            actor_ref: Some(ResourceRef::new(ResourceType::Party, "verifier-1").unwrap()),
            source_refs: vec![ResourceRef::new(ResourceType::Evidence, "evidence-1").unwrap()],
            input_refs: Vec::new(),
            operation: "verify".into(),
            occurred_at: "2026-10-07T08:00:00Z".into(),
        }
    }

    #[test]
    fn verification_binds_to_evidence() {
        assert!(verification().verify_against_original(&original()).is_ok());
    }

    #[test]
    fn verification_rejects_another_evidence_item() {
        let mut value = verification();
        value.evidence_ref = ResourceRef::new(ResourceType::Evidence, "evidence-2").unwrap();
        assert_eq!(
            value.verify_against_original(&original()),
            Err(EvidenceVerificationError::EvidenceMismatch)
        );
    }

    #[test]
    fn verification_binds_to_provenance_actor_and_source() {
        assert!(verification().verify_against_provenance(&provenance()).is_ok());
    }

    #[test]
    fn verification_rejects_unrelated_provenance() {
        let mut value = provenance();
        value.source_refs.clear();
        assert_eq!(
            verification().verify_against_provenance(&value),
            Err(EvidenceVerificationError::ProvenanceMismatch)
        );
    }
}
