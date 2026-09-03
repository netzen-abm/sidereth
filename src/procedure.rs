use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::Id;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProcedureStatus {
    Active,
    Inactive,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Procedure {
    pub procedure_id: Id,
    pub name: String,
    pub jurisdiction_id: Id,
    pub authority_id: Id,
    pub status: ProcedureStatus,
    pub step_ids: Vec<Id>,
    pub source_refs: Vec<Id>,
}

impl Procedure {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.procedure_id.is_empty() {
            return Err("procedure id is required");
        }
        if self.name.is_empty() {
            return Err("procedure name is required");
        }
        if self.jurisdiction_id.is_empty() {
            return Err("procedure jurisdiction is required");
        }
        if self.authority_id.is_empty() {
            return Err("procedure authority is required");
        }
        if self.source_refs.is_empty() {
            return Err("procedure source references are required");
        }
        if self.step_ids.iter().any(|id| id.is_empty()) {
            return Err("procedure step id is required");
        }
        let mut seen = HashSet::new();
        if self.step_ids.iter().any(|id| !seen.insert(id)) {
            return Err("duplicate procedure step id");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProcedureStep {
    pub step_id: Id,
    pub procedure_id: Id,
    pub sequence: u32,
    pub name: String,
    pub authority_id: Id,
    pub prerequisite_ids: Vec<Id>,
    pub evidence_refs: Vec<Id>,
    pub source_refs: Vec<Id>,
}

impl ProcedureStep {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.step_id.is_empty() {
            return Err("procedure step id is required");
        }
        if self.procedure_id.is_empty() {
            return Err("procedure step procedure is required");
        }
        if self.sequence == 0 {
            return Err("procedure step sequence must be positive");
        }
        if self.name.is_empty() {
            return Err("procedure step name is required");
        }
        if self.authority_id.is_empty() {
            return Err("procedure step authority is required");
        }
        if self.source_refs.is_empty() {
            return Err("procedure step source references are required");
        }
        if self.prerequisite_ids.iter().any(|id| id.is_empty()) {
            return Err("procedure prerequisite id is required");
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct ProcedureRegistry {
    procedures: HashMap<Id, Procedure>,
    steps: HashMap<Id, ProcedureStep>,
}

impl ProcedureRegistry {
    pub fn insert(
        &mut self,
        procedure: Procedure,
        steps: Vec<ProcedureStep>,
    ) -> Result<(), &'static str> {
        procedure.validate()?;
        if self.procedures.contains_key(&procedure.procedure_id) {
            return Err("duplicate procedure id");
        }
        if procedure.step_ids.len() != steps.len() {
            return Err("procedure step set does not match step definitions");
        }

        let mut local_ids = HashSet::new();
        let mut local_sequences = HashSet::new();
        for step in &steps {
            step.validate()?;
            if step.procedure_id != procedure.procedure_id {
                return Err("procedure step belongs to another procedure");
            }
            if !procedure.step_ids.contains(&step.step_id) {
                return Err("procedure step is not declared");
            }
            if !local_ids.insert(step.step_id.clone()) {
                return Err("duplicate procedure step id");
            }
            if !local_sequences.insert(step.sequence) {
                return Err("duplicate procedure step sequence");
            }
            if self.steps.contains_key(&step.step_id) {
                return Err("duplicate procedure step id");
            }
        }

        self.procedures
            .insert(procedure.procedure_id.clone(), procedure.clone());
        for step in steps {
            self.steps.insert(step.step_id.clone(), step);
        }

        if let Err(error) = self.validate_procedure(&procedure.procedure_id) {
            self.procedures.remove(&procedure.procedure_id);
            self.steps.retain(|_, step| {
                step.procedure_id != procedure.procedure_id
            });
            return Err(error);
        }
        Ok(())
    }

    pub fn get_procedure(&self, id: &Id) -> Option<&Procedure> {
        self.procedures.get(id)
    }

    pub fn get_step(&self, id: &Id) -> Option<&ProcedureStep> {
        self.steps.get(id)
    }

    pub fn procedure_ids(&self) -> Vec<Id> {
        let mut ids: Vec<Id> = self.procedures.keys().cloned().collect();
        ids.sort();
        ids
    }

    pub fn steps_for(&self, procedure_id: &Id) -> Vec<&ProcedureStep> {
        let mut steps: Vec<&ProcedureStep> = self
            .steps
            .values()
            .filter(|step| &step.procedure_id == procedure_id)
            .collect();
        steps.sort_by_key(|step| (step.sequence, step.step_id.clone()));
        steps
    }

    fn validate_procedure(&self, procedure_id: &Id) -> Result<(), &'static str> {
        let procedure = self
            .procedures
            .get(procedure_id)
            .ok_or("procedure not found")?;
        let steps = self.steps_for(procedure_id);
        if steps.len() != procedure.step_ids.len() {
            return Err("procedure step set does not match step definitions");
        }

        let ids: HashSet<&Id> = procedure.step_ids.iter().collect();
        let mut sequences = HashSet::new();
        for step in &steps {
            if !ids.contains(&step.step_id) {
                return Err("procedure step is not declared");
            }
            if step.authority_id != procedure.authority_id {
                return Err("procedure step authority mismatch");
            }
            if !sequences.insert(step.sequence) {
                return Err("duplicate procedure step sequence");
            }
            for prerequisite in &step.prerequisite_ids {
                let prior = self.steps.get(prerequisite).ok_or(
                    "procedure prerequisite step not found",
                )?;
                if prior.procedure_id != *procedure_id {
                    return Err("procedure prerequisite belongs to another procedure");
                }
                if prior.sequence >= step.sequence {
                    return Err("procedure prerequisite must precede step");
                }
            }
        }

        for step in &steps {
            let mut current = step.step_id.clone();
            let mut visited = HashSet::new();
            while let Some(prerequisite) = self.steps.get(&current)
                .and_then(|value| value.prerequisite_ids.first())
            {
                if !visited.insert(current.clone()) {
                    return Err("procedure prerequisite cycle detected");
                }
                current = prerequisite.clone();
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn procedure() -> Procedure {
        Procedure {
            procedure_id: "p-1".into(),
            name: "Example procedure".into(),
            jurisdiction_id: "j-1".into(),
            authority_id: "a-1".into(),
            status: ProcedureStatus::Active,
            step_ids: vec!["s-1".into(), "s-2".into()],
            source_refs: vec!["src-1".into()],
        }
    }

    fn step(id: &str, sequence: u32) -> ProcedureStep {
        ProcedureStep {
            step_id: id.into(),
            procedure_id: "p-1".into(),
            sequence,
            name: format!("Step {id}"),
            authority_id: "a-1".into(),
            prerequisite_ids: Vec::new(),
            evidence_refs: Vec::new(),
            source_refs: vec!["src-1".into()],
        }
    }

    #[test]
    fn valid_procedure_is_accepted() {
        let mut registry = ProcedureRegistry::default();
        assert!(registry
            .insert(procedure(), vec![step("s-1", 1), step("s-2", 2)])
            .is_ok());
    }

    #[test]
    fn missing_scope_is_rejected() {
        let mut value = procedure();
        value.jurisdiction_id.clear();
        assert_eq!(value.validate(), Err("procedure jurisdiction is required"));
    }

    #[test]
    fn missing_source_is_rejected() {
        let mut value = procedure();
        value.source_refs.clear();
        assert_eq!(value.validate(), Err("procedure source references are required"));
    }

    #[test]
    fn duplicate_step_ids_are_rejected() {
        let mut value = procedure();
        value.step_ids = vec!["s-1".into(), "s-1".into()];
        assert_eq!(value.validate(), Err("duplicate procedure step id"));
    }

    #[test]
    fn zero_sequence_is_rejected() {
        assert_eq!(step("s-1", 0).validate(), Err("procedure step sequence must be positive"));
    }

    #[test]
    fn unknown_prerequisite_is_rejected_atomically() {
        let mut registry = ProcedureRegistry::default();
        let mut second = step("s-2", 2);
        second.prerequisite_ids = vec!["missing".into()];
        assert_eq!(
            registry.insert(procedure(), vec![step("s-1", 1), second]),
            Err("procedure prerequisite step not found")
        );
        assert!(registry.get_procedure(&"p-1".into()).is_none());
    }

    #[test]
    fn forward_prerequisite_is_rejected() {
        let mut registry = ProcedureRegistry::default();
        let mut first = step("s-1", 1);
        first.prerequisite_ids = vec!["s-2".into()];
        assert_eq!(
            registry.insert(procedure(), vec![first, step("s-2", 2)]),
            Err("procedure prerequisite must precede step")
        );
    }

    #[test]
    fn authority_mismatch_is_rejected() {
        let mut registry = ProcedureRegistry::default();
        let mut value = step("s-1", 1);
        value.authority_id = "a-2".into();
        assert_eq!(
            registry.insert(
                procedure(),
                vec![value, step("s-2", 2)]
            ),
            Err("procedure step authority mismatch")
        );
    }

    #[test]
    fn duplicate_sequences_are_rejected() {
        let mut registry = ProcedureRegistry::default();
        assert_eq!(
            registry.insert(procedure(), vec![step("s-1", 1), step("s-2", 1)]),
            Err("duplicate procedure step sequence")
        );
    }

    #[test]
    fn steps_are_returned_in_stable_sequence_order() {
        let mut registry = ProcedureRegistry::default();
        registry
            .insert(procedure(), vec![step("s-2", 2), step("s-1", 1)])
            .unwrap();
        let ids: Vec<Id> = registry
            .steps_for(&"p-1".into())
            .into_iter()
            .map(|step| step.step_id.clone())
            .collect();
        assert_eq!(ids, vec!["s-1", "s-2"]);
    }
}
