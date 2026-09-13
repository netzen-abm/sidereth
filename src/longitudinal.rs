use crate::{EventEnvelope, Id, Observation, ResourceRef};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LongitudinalEntry {
    Event(EventEnvelope),
    Observation(Observation),
}

impl LongitudinalEntry {
    pub fn source_ref(&self) -> ResourceRef {
        match self {
            Self::Event(event) => ResourceRef::new(crate::ResourceType::Event, event.event_id.clone())
                .expect("event id has already been validated"),
            Self::Observation(observation) => ResourceRef::new(
                crate::ResourceType::Observation,
                observation.observation_id.clone(),
            )
            .expect("observation id has already been validated"),
        }
    }

    fn sort_key(&self) -> (&str, &str) {
        match self {
            Self::Event(event) => (&event.occurred_at, &event.event_id),
            Self::Observation(observation) => (&observation.observed_at, &observation.observation_id),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LongitudinalView {
    subject_ref: ResourceRef,
    entries: Vec<LongitudinalEntry>,
}

impl LongitudinalView {
    pub fn from_entries(
        subject_ref: ResourceRef,
        entries: Vec<LongitudinalEntry>,
    ) -> Result<Self, &'static str> {
        if entries.is_empty() {
            return Err("longitudinal view requires at least one entry");
        }

        for entry in &entries {
            match entry {
                LongitudinalEntry::Event(event) => event.validate()?,
                LongitudinalEntry::Observation(observation) => observation.validate()?,
            }
        }

        entries.iter().try_for_each(|entry| {
            let entry_subject = match entry {
                LongitudinalEntry::Event(event) => event.aggregate_ref()?,
                LongitudinalEntry::Observation(observation) => observation.subject_ref.clone(),
            };
            if entry_subject == subject_ref {
                Ok(())
            } else {
                Err("longitudinal entry does not belong to subject")
            }
        })?;

        let mut entries = entries;
        entries.sort_by(|left, right| left.sort_key().cmp(&right.sort_key()));
        Ok(Self { subject_ref, entries })
    }

    pub fn subject_ref(&self) -> &ResourceRef {
        &self.subject_ref
    }

    pub fn entries(&self) -> &[LongitudinalEntry] {
        &self.entries
    }

    pub fn source_ids(&self) -> Vec<Id> {
        self.entries
            .iter()
            .map(|entry| match entry {
                LongitudinalEntry::Event(event) => event.event_id.clone(),
                LongitudinalEntry::Observation(observation) => observation.observation_id.clone(),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EpistemicStatus, IntelligenceDataClass, ObservationOrigin, ObservationType};
    use serde_json::Value;

    fn case_ref() -> ResourceRef {
        ResourceRef::new(crate::ResourceType::Case, "case-1").unwrap()
    }

    fn event(id: &str, time: &str) -> EventEnvelope {
        EventEnvelope {
            event_id: id.into(),
            event_type: "case.event".into(),
            aggregate_type: "case".into(),
            aggregate_id: "case-1".into(),
            occurred_at: time.into(),
            actor_type: "user".into(),
            actor_id: "user-1".into(),
            schema_version: 1,
            payload: Value::Null,
            source_refs: vec![],
            correlation_id: "corr-1".into(),
            causation_id: None,
        }
    }

    fn observation(id: &str, time: &str) -> Observation {
        Observation {
            observation_id: id.into(),
            schema_version: 1,
            subject_ref: case_ref(),
            observation_type: ObservationType::Condition,
            observation_origin: ObservationOrigin::UserReport,
            observed_at: time.into(),
            recorded_at: "2026-09-13T10:00:00Z".into(),
            assertion: Value::String("reported".into()),
            epistemic_status: EpistemicStatus::UserReported,
            source_refs: vec![],
            evidence_refs: vec![],
            provenance_ref: None,
            context_refs: vec![],
            data_class: IntelligenceDataClass::Public,
        }
    }

    #[test]
    fn projection_orders_events_and_observations_without_rewriting_sources() {
        let view = LongitudinalView::from_entries(
            case_ref(),
            vec![
                LongitudinalEntry::Observation(observation("obs-2", "2026-09-02T11:00:00Z")),
                LongitudinalEntry::Event(event("event-1", "2026-09-02T10:00:00Z")),
                LongitudinalEntry::Observation(observation("obs-1", "2026-09-02T10:30:00Z")),
            ],
        )
        .unwrap();

        assert_eq!(view.source_ids(), vec!["event-1", "obs-1", "obs-2"]);
    }

    #[test]
    fn projection_rejects_entry_for_different_subject() {
        let mut other = observation("obs-1", "2026-09-02T10:00:00Z");
        other.subject_ref = ResourceRef::new(crate::ResourceType::Incident, "incident-1").unwrap();
        assert_eq!(
            LongitudinalView::from_entries(case_ref(), vec![LongitudinalEntry::Observation(other)]),
            Err("longitudinal entry does not belong to subject")
        );
    }

    #[test]
    fn projection_does_not_upgrade_observation_status() {
        let obs = observation("obs-1", "2026-09-02T10:00:00Z");
        let view = LongitudinalView::from_entries(case_ref(), vec![LongitudinalEntry::Observation(obs)]).unwrap();
        match &view.entries()[0] {
            LongitudinalEntry::Observation(observation) => {
                assert_eq!(observation.epistemic_status, EpistemicStatus::UserReported);
            }
            _ => panic!("expected observation"),
        }
    }

    #[test]
    fn empty_projection_is_rejected() {
        assert_eq!(
            LongitudinalView::from_entries(case_ref(), vec![]),
            Err("longitudinal view requires at least one entry")
        );
    }
}
