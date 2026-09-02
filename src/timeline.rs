use crate::event::EventEnvelope;
use crate::Id;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Timeline {
    events: Vec<EventEnvelope>,
}

impl Timeline {
    pub fn from_events(mut events: Vec<EventEnvelope>) -> Result<Self, &'static str> {
        for event in &events {
            event.validate()?;
        }

        events.sort_by(|left, right| {
            left.occurred_at
                .cmp(&right.occurred_at)
                .then_with(|| left.event_id.cmp(&right.event_id))
        });

        for pair in events.windows(2) {
            if pair[0].event_id == pair[1].event_id {
                return Err("duplicate event id");
            }
        }

        Ok(Self { events })
    }

    pub fn events(&self) -> &[EventEnvelope] {
        &self.events
    }

    pub fn event_ids(&self) -> Vec<Id> {
        self.events
            .iter()
            .map(|event| event.event_id.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

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

    #[test]
    fn timeline_orders_events_by_time() {
        let timeline = Timeline::from_events(vec![
            event("event-2", "2026-09-02T11:00:00Z"),
            event("event-1", "2026-09-02T10:00:00Z"),
        ])
        .unwrap();

        assert_eq!(
            timeline.event_ids(),
            vec!["event-1".to_string(), "event-2".to_string()]
        );
    }

    #[test]
    fn equal_times_use_event_id_as_tie_breaker() {
        let timeline = Timeline::from_events(vec![
            event("event-b", "2026-09-02T10:00:00Z"),
            event("event-a", "2026-09-02T10:00:00Z"),
        ])
        .unwrap();

        assert_eq!(
            timeline.event_ids(),
            vec!["event-a".to_string(), "event-b".to_string()]
        );
    }

    #[test]
    fn duplicate_event_ids_are_rejected() {
        let result = Timeline::from_events(vec![
            event("event-1", "2026-09-02T10:00:00Z"),
            event("event-1", "2026-09-02T11:00:00Z"),
        ]);

        assert_eq!(result, Err("duplicate event id"));
    }
}
