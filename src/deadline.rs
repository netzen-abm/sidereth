use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::Id;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct CivilDate {
    pub year: i32,
    pub month: u8,
    pub day: u8,
}

impl CivilDate {
    pub fn new(year: i32, month: u8, day: u8) -> Result<Self, &'static str> {
        if !(1..=12).contains(&month) || day == 0 {
            return Err("invalid civil date");
        }
        let max_day = days_in_month(year, month);
        if day > max_day {
            return Err("invalid civil date");
        }
        Ok(Self { year, month, day })
    }

    pub fn checked_add_days(self, days: u32) -> Option<Self> {
        let ordinal = days_from_civil(self.year, self.month, self.day);
        let target = ordinal.checked_add(i64::from(days))?;
        civil_from_days(target)
    }
}

fn is_leap_year(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn days_from_civil(year: i32, month: u8, day: u8) -> i64 {
    let y = i64::from(year) - if month <= 2 { 1 } else { 0 };
    let era = (if y >= 0 { y } else { y - 399 }) / 400;
    let year_of_era = y - era * 400;
    let month_i = i64::from(month);
    let day_i = i64::from(day);
    let adjusted_month = month_i + if month_i > 2 { -3 } else { 9 };
    let day_of_year = (153 * adjusted_month + 2) / 5 + day_i - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

fn civil_from_days(days: i64) -> Option<CivilDate> {
    let days = days + 719_468;
    let era = (if days >= 0 { days } else { days - 146_096 }) / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_part = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_part + 2) / 5 + 1;
    let month = month_part + if month_part < 10 { 3 } else { -9 };
    let year = year + if month <= 2 { 1 } else { 0 };
    let year = i32::try_from(year).ok()?;
    let month = u8::try_from(month).ok()?;
    let day = u8::try_from(day).ok()?;
    CivilDate::new(year, month, day).ok()
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeadlineType {
    Filing,
    Response,
    Appeal,
    Other,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeadlineStatus {
    Planned,
    Due,
    Satisfied,
    Overdue,
    Cancelled,
}

impl DeadlineStatus {
    pub fn can_transition_to(&self, next: &Self) -> bool {
        matches!(
            (self, next),
            (Self::Planned, Self::Due)
                | (Self::Planned, Self::Cancelled)
                | (Self::Due, Self::Satisfied)
                | (Self::Due, Self::Overdue)
                | (Self::Due, Self::Cancelled)
                | (Self::Overdue, Self::Satisfied)
                | (Self::Overdue, Self::Cancelled)
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Deadline {
    pub deadline_id: Id,
    pub procedure_id: Id,
    pub name: String,
    pub deadline_type: DeadlineType,
    pub anchor_date: CivilDate,
    pub duration_days: u32,
    pub due_date: CivilDate,
    pub status: DeadlineStatus,
    pub source_refs: Vec<Id>,
}

impl Deadline {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.deadline_id.is_empty() {
            return Err("deadline id is required");
        }
        if self.procedure_id.is_empty() {
            return Err("deadline procedure is required");
        }
        if self.name.is_empty() {
            return Err("deadline name is required");
        }
        if self.source_refs.is_empty() {
            return Err("deadline source references are required");
        }
        let expected = self
            .anchor_date
            .checked_add_days(self.duration_days)
            .ok_or("deadline due date overflow")?;
        if self.due_date != expected {
            return Err("deadline due date does not match duration");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ApplicabilityStatus {
    Unverified,
    Verified,
    Uncertain,
    ReviewRequired,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Obligation {
    pub obligation_id: Id,
    pub deadline_id: Id,
    pub description: String,
    pub applicability: ApplicabilityStatus,
    pub source_refs: Vec<Id>,
}

impl Obligation {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.obligation_id.is_empty() {
            return Err("obligation id is required");
        }
        if self.deadline_id.is_empty() {
            return Err("obligation deadline is required");
        }
        if self.description.is_empty() {
            return Err("obligation description is required");
        }
        if self.source_refs.is_empty() {
            return Err("obligation source references are required");
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct DeadlineRegistry {
    deadlines: HashMap<Id, Deadline>,
    obligations: HashMap<Id, Obligation>,
}

impl DeadlineRegistry {
    pub fn insert(
        &mut self,
        deadline: Deadline,
        obligations: Vec<Obligation>,
    ) -> Result<(), &'static str> {
        deadline.validate()?;
        if self.deadlines.contains_key(&deadline.deadline_id) {
            return Err("duplicate deadline id");
        }

        for obligation in &obligations {
            obligation.validate()?;
            if obligation.deadline_id != deadline.deadline_id {
                return Err("obligation belongs to another deadline");
            }
            if self.obligations.contains_key(&obligation.obligation_id) {
                return Err("duplicate obligation id");
            }
        }

        self.deadlines
            .insert(deadline.deadline_id.clone(), deadline.clone());
        for obligation in obligations {
            self.obligations
                .insert(obligation.obligation_id.clone(), obligation);
        }

        Ok(())
    }

    pub fn transition(
        &mut self,
        deadline_id: &Id,
        next: DeadlineStatus,
    ) -> Result<(), &'static str> {
        let deadline = self
            .deadlines
            .get_mut(deadline_id)
            .ok_or("deadline not found")?;
        if !deadline.status.can_transition_to(&next) {
            return Err("invalid deadline status transition");
        }
        deadline.status = next;
        Ok(())
    }

    pub fn get_deadline(&self, id: &Id) -> Option<&Deadline> {
        self.deadlines.get(id)
    }

    pub fn get_obligation(&self, id: &Id) -> Option<&Obligation> {
        self.obligations.get(id)
    }

    pub fn deadline_ids(&self) -> Vec<Id> {
        let mut ids: Vec<Id> = self.deadlines.keys().cloned().collect();
        ids.sort();
        ids
    }

    pub fn obligations_for(&self, deadline_id: &Id) -> Vec<&Obligation> {
        let mut values: Vec<&Obligation> = self
            .obligations
            .values()
            .filter(|item| &item.deadline_id == deadline_id)
            .collect();
        values.sort_by_key(|item| item.obligation_id.clone());
        values
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date() -> CivilDate {
        CivilDate::new(2026, 1, 31).unwrap()
    }

    fn deadline() -> Deadline {
        Deadline {
            deadline_id: "d-1".into(),
            procedure_id: "p-1".into(),
            name: "Response deadline".into(),
            deadline_type: DeadlineType::Response,
            anchor_date: date(),
            duration_days: 28,
            due_date: CivilDate::new(2026, 2, 28).unwrap(),
            status: DeadlineStatus::Planned,
            source_refs: vec!["src-1".into()],
        }
    }

    fn obligation() -> Obligation {
        Obligation {
            obligation_id: "o-1".into(),
            deadline_id: "d-1".into(),
            description: "Provide the required response".into(),
            applicability: ApplicabilityStatus::Unverified,
            source_refs: vec!["src-1".into()],
        }
    }

    #[test]
    fn calendar_addition_handles_month_end() {
        assert_eq!(
            date().checked_add_days(28),
            Some(CivilDate::new(2026, 2, 28).unwrap())
        );
    }

    #[test]
    fn leap_day_is_valid() {
        assert_eq!(
            CivilDate::new(2028, 2, 29).unwrap().checked_add_days(1),
            Some(CivilDate::new(2028, 3, 1).unwrap())
        );
    }

    #[test]
    fn invalid_date_is_rejected() {
        assert_eq!(CivilDate::new(2026, 2, 29), Err("invalid civil date"));
    }

    #[test]
    fn inconsistent_due_date_is_rejected() {
        let mut value = deadline();
        value.due_date = CivilDate::new(2026, 3, 1).unwrap();
        assert_eq!(
            value.validate(),
            Err("deadline due date does not match duration")
        );
    }

    #[test]
    fn missing_source_is_rejected() {
        let mut value = deadline();
        value.source_refs.clear();
        assert_eq!(
            value.validate(),
            Err("deadline source references are required")
        );
    }

    #[test]
    fn duplicate_ids_are_rejected() {
        let mut registry = DeadlineRegistry::default();
        registry.insert(deadline(), vec![]).unwrap();
        assert_eq!(
            registry.insert(deadline(), vec![]),
            Err("duplicate deadline id")
        );
    }

    #[test]
    fn obligation_mismatch_is_rejected() {
        let mut value = obligation();
        value.deadline_id = "other".into();
        let mut registry = DeadlineRegistry::default();
        assert_eq!(
            registry.insert(deadline(), vec![value]),
            Err("obligation belongs to another deadline")
        );
    }

    #[test]
    fn invalid_status_transition_is_rejected() {
        let mut registry = DeadlineRegistry::default();
        registry.insert(deadline(), vec![]).unwrap();
        assert_eq!(
            registry.transition(&"d-1".into(), DeadlineStatus::Satisfied),
            Err("invalid deadline status transition")
        );
    }

    #[test]
    fn status_transition_is_deterministic() {
        let mut registry = DeadlineRegistry::default();
        registry.insert(deadline(), vec![]).unwrap();
        assert!(registry
            .transition(&"d-1".into(), DeadlineStatus::Due)
            .is_ok());
        assert_eq!(
            registry.get_deadline(&"d-1".into()).unwrap().status,
            DeadlineStatus::Due
        );
    }

    #[test]
    fn ids_are_sorted() {
        let mut registry = DeadlineRegistry::default();
        let mut second = deadline();
        second.deadline_id = "d-2".into();
        registry.insert(second, vec![]).unwrap();
        registry.insert(deadline(), vec![]).unwrap();
        assert_eq!(registry.deadline_ids(), vec!["d-1", "d-2"]);
    }

    #[test]
    fn applicability_is_not_a_lawfulness_conclusion() {
        let item = obligation();
        assert_eq!(item.applicability, ApplicabilityStatus::Unverified);
        assert!(item.validate().is_ok());
    }
}
