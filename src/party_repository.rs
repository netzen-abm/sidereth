use crate::party::{Party, PartyRelationship};
use crate::persistence::{PersistenceError, ResourceWrite, ResourceWriteMode, UnitOfWorkContext};
use crate::{Id, ResourceRef, ResourceType};

/// Provider-neutral persistence boundary for Party and PartyRelationship.
///
/// The repository deliberately does not own transactions. A caller may use the
/// same Unit-of-Work context to atomically persist parties, relationships, cases,
/// incidents, evidence, and other resources.
pub struct PartyUnitOfWorkRepository<'a, C: UnitOfWorkContext> {
    context: &'a mut C,
}

impl<'a, C: UnitOfWorkContext> PartyUnitOfWorkRepository<'a, C> {
    pub fn new(context: &'a mut C) -> Self {
        Self { context }
    }

    pub fn create_party(&mut self, party: Party) -> Result<(), PersistenceError> {
        party
            .validate()
            .map_err(|_| PersistenceError::ValidationFailure)?;
        let resource_ref = ResourceRef::new(ResourceType::Party, party.party_id.clone())
            .map_err(|_| PersistenceError::IntegrityFailure)?;
        let payload = serde_json::to_value(&party)
            .map_err(|_| PersistenceError::SerializationFailure)?;
        let write = ResourceWrite::new(
            resource_ref,
            party.schema_version as u16,
            payload,
            ResourceWriteMode::Insert,
        )
        .map_err(|_| PersistenceError::ValidationFailure)?;
        self.context.write_resource(write).map_err(map_uow_error)
    }

    pub fn get_party(&mut self, party_id: &Id) -> Result<Option<Party>, PersistenceError> {
        let resource_ref = ResourceRef::new(ResourceType::Party, party_id.clone())
            .map_err(|_| PersistenceError::IntegrityFailure)?;
        let record = self
            .context
            .read_resource(&resource_ref)
            .map_err(map_uow_error)?;
        record
            .map(|record| {
                let party = serde_json::from_value::<Party>(record.payload)
                    .map_err(|_| PersistenceError::SerializationFailure)?;
                party
                    .validate()
                    .map_err(|_| PersistenceError::ValidationFailure)?;
                Ok(party)
            })
            .transpose()
    }

    pub fn create_relationship(
        &mut self,
        relationship: PartyRelationship,
    ) -> Result<(), PersistenceError> {
        relationship
            .validate()
            .map_err(|_| PersistenceError::ValidationFailure)?;
        let resource_ref = ResourceRef::new(
            ResourceType::PartyRelationship,
            relationship.relationship_id.clone(),
        )
        .map_err(|_| PersistenceError::IntegrityFailure)?;
        let payload = serde_json::to_value(&relationship)
            .map_err(|_| PersistenceError::SerializationFailure)?;
        let write = ResourceWrite::new(
            resource_ref,
            relationship.schema_version as u16,
            payload,
            ResourceWriteMode::Insert,
        )
        .map_err(|_| PersistenceError::ValidationFailure)?;
        self.context.write_resource(write).map_err(map_uow_error)
    }

    pub fn get_relationship(
        &mut self,
        relationship_id: &Id,
    ) -> Result<Option<PartyRelationship>, PersistenceError> {
        let resource_ref = ResourceRef::new(
            ResourceType::PartyRelationship,
            relationship_id.clone(),
        )
        .map_err(|_| PersistenceError::IntegrityFailure)?;
        let record = self
            .context
            .read_resource(&resource_ref)
            .map_err(map_uow_error)?;
        record
            .map(|record| {
                let relationship = serde_json::from_value::<PartyRelationship>(record.payload)
                    .map_err(|_| PersistenceError::SerializationFailure)?;
                relationship
                    .validate()
                    .map_err(|_| PersistenceError::ValidationFailure)?;
                Ok(relationship)
            })
            .transpose()
    }
}

fn map_uow_error(error: crate::persistence::UnitOfWorkError) -> PersistenceError {
    match error {
        crate::persistence::UnitOfWorkError::Persistence(error) => error,
        crate::persistence::UnitOfWorkError::InvalidOperation => {
            PersistenceError::ValidationFailure
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::party::{PartyKind, PartyStatus};
    use crate::persistence::{ResourceRecord, Revision, UnitOfWorkError};

    #[derive(Default)]
    struct FakeContext {
        records: std::collections::HashMap<(ResourceType, Id), ResourceRecord>,
    }

    impl UnitOfWorkContext for FakeContext {
        fn read_resource(
            &mut self,
            resource_ref: &ResourceRef,
        ) -> Result<Option<ResourceRecord>, UnitOfWorkError> {
            Ok(self
                .records
                .get(&(resource_ref.resource_type, resource_ref.id.clone()))
                .cloned())
        }

        fn write_resource(&mut self, write: ResourceWrite) -> Result<(), UnitOfWorkError> {
            let key = (write.resource_ref.resource_type, write.resource_ref.id.clone());
            if write.mode == ResourceWriteMode::Insert && self.records.contains_key(&key) {
                return Err(UnitOfWorkError::Persistence(PersistenceError::Duplicate));
            }
            self.records.insert(
                key,
                ResourceRecord {
                    resource_ref: write.resource_ref,
                    schema_version: write.schema_version,
                    revision: Revision::initial(),
                    payload: write.payload,
                },
            );
            Ok(())
        }

        fn link_resources(
            &mut self,
            _link: crate::persistence::ResourceLink,
        ) -> Result<(), UnitOfWorkError> {
            Ok(())
        }
    }

    fn party(id: &str) -> Party {
        Party {
            party_id: id.into(),
            schema_version: 1,
            party_kind: PartyKind::Person,
            status: PartyStatus::Active,
            display_name: "Test Person".into(),
            identity_refs: vec![],
            jurisdiction_refs: vec!["j-1".into()],
            organization_ref: None,
            provenance_ref: None,
            privacy_classification: "private".into(),
            created_at: "2026-09-07T00:00:00Z".into(),
            updated_at: "2026-09-07T00:00:00Z".into(),
        }
    }

    fn relationship() -> PartyRelationship {
        PartyRelationship {
            relationship_id: "rel-1".into(),
            schema_version: 1,
            from_party_id: "p-1".into(),
            to_party_id: "p-2".into(),
            relationship_type: "representation".into(),
            context_ref: Some("case-1".into()),
            role: "legal_representative".into(),
            valid_from: "2026-09-07T00:00:00Z".into(),
            valid_to: None,
            provenance_ref: None,
            authorization_ref: None,
            created_at: "2026-09-07T00:00:00Z".into(),
        }
    }

    #[test]
    fn party_round_trip_uses_canonical_uow_boundary() {
        let mut context = FakeContext::default();
        let mut repository = PartyUnitOfWorkRepository::new(&mut context);
        let value = party("p-1");
        repository.create_party(value.clone()).unwrap();
        assert_eq!(repository.get_party(&"p-1".into()).unwrap(), Some(value));
    }

    #[test]
    fn relationship_round_trip_uses_distinct_resource_type() {
        let mut context = FakeContext::default();
        let mut repository = PartyUnitOfWorkRepository::new(&mut context);
        let value = relationship();
        repository.create_relationship(value.clone()).unwrap();
        assert_eq!(
            repository.get_relationship(&"rel-1".into()).unwrap(),
            Some(value)
        );
    }

    #[test]
    fn duplicate_party_is_propagated_from_provider() {
        let mut context = FakeContext::default();
        let mut repository = PartyUnitOfWorkRepository::new(&mut context);
        let value = party("p-1");
        repository.create_party(value.clone()).unwrap();
        assert_eq!(
            repository.create_party(value),
            Err(PersistenceError::Duplicate)
        );
    }

    #[test]
    fn invalid_party_is_rejected_before_write() {
        let mut context = FakeContext::default();
        let mut repository = PartyUnitOfWorkRepository::new(&mut context);
        let mut value = party("p-1");
        value.display_name.clear();
        assert_eq!(
            repository.create_party(value),
            Err(PersistenceError::ValidationFailure)
        );
        assert!(context.records.is_empty());
    }
}
