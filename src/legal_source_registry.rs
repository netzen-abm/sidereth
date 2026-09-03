use std::collections::{HashMap, HashSet};

use crate::legal_source::LegalSource;
use crate::Id;

#[derive(Debug, Default)]
pub struct LegalSourceRegistry {
    sources: HashMap<Id, LegalSource>,
}

impl LegalSourceRegistry {
    pub fn insert(&mut self, source: LegalSource) -> Result<(), &'static str> {
        source.validate()?;
        if self.sources.contains_key(&source.source_id) {
            return Err("duplicate source id");
        }
        self.sources.insert(source.source_id.clone(), source);
        self.validate_supersession_graph()?;
        Ok(())
    }

    pub fn get(&self, source_id: &Id) -> Option<&LegalSource> {
        self.sources.get(source_id)
    }

    pub fn ids(&self) -> Vec<Id> {
        let mut ids: Vec<Id> = self.sources.keys().cloned().collect();
        ids.sort();
        ids
    }

    fn validate_supersession_graph(&self) -> Result<(), &'static str> {
        for source in self.sources.values() {
            let mut current = source.source_id.clone();
            let mut visited = HashSet::new();

            while let Some(next) = self
                .sources
                .get(&current)
                .and_then(|value| value.supersedes.clone())
            {
                if !visited.insert(current.clone()) {
                    return Err("supersession cycle detected");
                }
                current = next;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::legal_source::{SourceType, VerificationStatus};

    fn source(id: &str) -> LegalSource {
        LegalSource {
            source_id: id.into(),
            source_type: SourceType::ConstitutionOrLegislation,
            title: format!("Source {id}"),
            issuing_authority: "authority".into(),
            jurisdiction: "IN".into(),
            citation: format!("citation-{id}"),
            published_at: "2026-01-01T00:00:00Z".into(),
            effective_from: "2026-01-01T00:00:00Z".into(),
            effective_to: None,
            version: "1".into(),
            retrieved_at: "2026-09-03T00:00:00Z".into(),
            verification_status: VerificationStatus::Verified,
            supersedes: None,
        }
    }

    #[test]
    fn duplicate_source_ids_are_rejected() {
        let mut registry = LegalSourceRegistry::default();
        registry.insert(source("law-1")).unwrap();
        assert_eq!(registry.insert(source("law-1")), Err("duplicate source id"));
    }

    #[test]
    fn ids_are_returned_in_stable_order() {
        let mut registry = LegalSourceRegistry::default();
        registry.insert(source("law-2")).unwrap();
        registry.insert(source("law-1")).unwrap();
        assert_eq!(registry.ids(), vec!["law-1", "law-2"]);
    }

    #[test]
    fn supersession_cycle_is_rejected() {
        let mut registry = LegalSourceRegistry::default();
        let mut first = source("law-1");
        first.supersedes = Some("law-2".into());
        let mut second = source("law-2");
        second.supersedes = Some("law-1".into());
        registry.sources.insert(first.source_id.clone(), first);
        registry.sources.insert(second.source_id.clone(), second);

        assert_eq!(
            registry.validate_supersession_graph(),
            Err("supersession cycle detected")
        );
    }
}
