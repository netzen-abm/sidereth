#![cfg(feature = "postgres")]

//! Live PostgreSQL proof matrix for the universal transactional core.
//!
//! These tests are intentionally ignored by default because they require a real
//! PostgreSQL instance. Run with `DATABASE_URL=... cargo test --features postgres
//! --test postgres_atomicity -- --ignored --nocapture`.

use std::env;
use std::sync::{Arc, Barrier};
use std::thread;

use serde_json::json;
use sidereth_core::persistence::{
    PersistenceError, ResourceWrite, ResourceWriteMode, Revision, UnitOfWork, UnitOfWorkContext,
    UnitOfWorkFactory,
};
use sidereth_core::{
    EvidenceCapture, EvidenceOriginal, EvidenceTransformation, EvidenceTrustMetadata,
    EvidenceTrustUnitOfWorkRepository, PostgresUnitOfWorkFactory, Provenance, ResourceRef,
    ResourceType,
};

fn database_url() -> String {
    env::var("DATABASE_URL").expect("DATABASE_URL is required for live PostgreSQL tests")
}

fn prepare(factory: &mut PostgresUnitOfWorkFactory) {
    let mut uow = factory.begin().expect("begin setup transaction");
    uow.execute(|ctx| {
        let mut client = postgres::Client::connect(factory.connection_string(), postgres::NoTls)
            .expect("connect for setup");
        client
            .batch_execute(
                "BEGIN;
                SELECT pg_advisory_xact_lock(734945731);
                CREATE TABLE IF NOT EXISTS sidereth_resource_records (
                    resource_type TEXT NOT NULL,
                    resource_id TEXT NOT NULL,
                    schema_version INTEGER NOT NULL CHECK (schema_version > 0),
                    revision BIGINT NOT NULL DEFAULT 0 CHECK (revision >= 0),
                    payload JSONB NOT NULL,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    PRIMARY KEY (resource_type, resource_id)
                );
                CREATE TABLE IF NOT EXISTS sidereth_resource_links (
                    source_type TEXT NOT NULL,
                    source_id TEXT NOT NULL,
                    relation TEXT NOT NULL CHECK (btrim(relation) <> ''),
                    target_type TEXT NOT NULL,
                    target_id TEXT NOT NULL,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    PRIMARY KEY (source_type, source_id, relation, target_type, target_id)
                );
                COMMIT;",
            )
            .expect("create test tables");
        let _ = ctx;
        Ok::<_, sidereth_core::persistence::UnitOfWorkError>(())
    })
    .expect("setup operation");
    uow.commit().expect("commit setup");
}

fn unique_id(prefix: &str) -> String {
    format!("{}-{}", prefix, std::process::id())
}

fn evidence_for(case_id: &str, evidence_id: &str) -> EvidenceOriginal {
    EvidenceOriginal::from_capture(EvidenceCapture {
        evidence_id: evidence_id.into(),
        schema_version: 1,
        case_id: Some(case_id.into()),
        incident_id: None,
        captured_at: "2026-09-07T10:00:00Z".into(),
        captured_by: "party-proof".into(),
        media_type: "text/plain".into(),
        storage_ref: format!("storage-{}", evidence_id),
        content: b"evidence payload",
    })
    .unwrap()
}

fn trust_metadata() -> EvidenceTrustMetadata {
    EvidenceTrustMetadata::default()
}

#[test]
#[ignore = "requires live PostgreSQL"]
fn live_postgres_cas_race_allows_exactly_one_writer() {
    let url = database_url();
    let mut setup = PostgresUnitOfWorkFactory::new(url.clone());
    prepare(&mut setup);

    let resource = ResourceRef::new(ResourceType::Other, unique_id("cas-race")).unwrap();
    let mut seed = PostgresUnitOfWorkFactory::new(url.clone());
    let mut uow = seed.begin().unwrap();
    uow.execute(|ctx| {
        ctx.write_resource(ResourceWrite::new(
            resource.clone(),
            1,
            json!({"value": 0}),
            ResourceWriteMode::Insert,
        )?)
    })
    .unwrap();
    uow.commit().unwrap();

    let barrier = Arc::new(Barrier::new(2));
    let mut handles = Vec::new();
    for value in [1, 2] {
        let url = url.clone();
        let resource = resource.clone();
        let barrier = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            let mut factory = PostgresUnitOfWorkFactory::new(url);
            let mut uow = factory.begin().unwrap();
            let observed = uow
                .execute(|ctx| ctx.read_resource(&resource))
                .unwrap()
                .unwrap();
            assert_eq!(observed.revision, Revision::initial());
            barrier.wait();
            let result = uow.execute(|ctx| {
                ctx.write_resource(
                    ResourceWrite::new(
                        resource,
                        1,
                        json!({"value": value}),
                        ResourceWriteMode::Upsert,
                    )?
                    .with_expected_revision(observed.revision),
                )
            });
            match result {
                Ok(()) => {
                    uow.commit().unwrap();
                    true
                }
                Err(_) => {
                    let _ = uow.rollback();
                    false
                }
            }
        }));
    }

    let successes = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .filter(|success| *success)
        .count();
    assert_eq!(successes, 1, "exactly one CAS writer must commit");
}

#[test]
#[ignore = "requires live PostgreSQL"]
fn live_postgres_failed_command_rolls_back_all_resource_writes() {
    let url = database_url();
    let mut setup = PostgresUnitOfWorkFactory::new(url.clone());
    prepare(&mut setup);

    let case_ref = ResourceRef::new(ResourceType::Other, unique_id("rollback-case")).unwrap();
    let event_ref = ResourceRef::new(ResourceType::Other, unique_id("rollback-event")).unwrap();
    let mut factory = PostgresUnitOfWorkFactory::new(url);
    let mut uow = factory.begin().unwrap();
    let result = uow.execute(|ctx| {
        ctx.write_resource(ResourceWrite::new(
            case_ref.clone(),
            1,
            json!({"state": "mutated"}),
            ResourceWriteMode::Insert,
        )?)?;
        ctx.write_resource(ResourceWrite::new(
            event_ref.clone(),
            1,
            json!({"event": "created"}),
            ResourceWriteMode::Insert,
        )?)?;
        Err::<(), _>(sidereth_core::persistence::UnitOfWorkError::Persistence(
            PersistenceError::Conflict,
        ))
    });
    assert!(result.is_err());
    uow.rollback().unwrap();

    let mut verify = PostgresUnitOfWorkFactory::new(factory.connection_string().to_owned());
    let mut read_uow = verify.begin().unwrap();
    let records = read_uow
        .execute(|ctx| {
            Ok::<_, sidereth_core::persistence::UnitOfWorkError>((
                ctx.read_resource(&case_ref)?,
                ctx.read_resource(&event_ref)?,
            ))
        })
        .unwrap();
    read_uow.commit().unwrap();
    assert!(records.0.is_none());
    assert!(records.1.is_none());
}

#[test]
#[ignore = "requires live PostgreSQL"]
fn live_postgres_duplicate_insert_is_atomic() {
    let url = database_url();
    let mut setup = PostgresUnitOfWorkFactory::new(url.clone());
    prepare(&mut setup);
    let resource = ResourceRef::new(ResourceType::Other, unique_id("duplicate")).unwrap();

    let mut first = PostgresUnitOfWorkFactory::new(url.clone());
    let mut first_uow = first.begin().unwrap();
    first_uow
        .execute(|ctx| {
            ctx.write_resource(ResourceWrite::new(
                resource.clone(),
                1,
                json!({"n": 1}),
                ResourceWriteMode::Insert,
            )?)
        })
        .unwrap();
    first_uow.commit().unwrap();

    let mut second = PostgresUnitOfWorkFactory::new(url);
    let mut second_uow = second.begin().unwrap();
    let result = second_uow.execute(|ctx| {
        ctx.write_resource(ResourceWrite::new(
            resource.clone(),
            1,
            json!({"n": 2}),
            ResourceWriteMode::Insert,
        )?)
    });
    assert_eq!(
        result,
        Err(sidereth_core::persistence::UnitOfWorkError::Persistence(
            PersistenceError::Duplicate
        ))
    );
    second_uow.rollback().unwrap();
}

#[test]
#[ignore = "requires live PostgreSQL"]
fn live_postgres_case_and_evidence_trust_commit_atomically() {
    let url = database_url();
    let mut setup = PostgresUnitOfWorkFactory::new(url.clone());
    prepare(&mut setup);

    let case_id = unique_id("evidence-commit-case");
    let evidence_id = unique_id("evidence-commit");
    let provenance_id = unique_id("evidence-commit-provenance");
    let case_ref = ResourceRef::new(ResourceType::Case, case_id.clone()).unwrap();
    let evidence_ref = ResourceRef::new(ResourceType::Evidence, evidence_id.clone()).unwrap();
    let provenance_ref = ResourceRef::new(ResourceType::Provenance, provenance_id.clone()).unwrap();
    let evidence = sidereth_core::PersistedEvidence::new(
        1,
        evidence_for(&case_id, &evidence_id),
        trust_metadata(),
    )
    .unwrap()
    .with_transformation(EvidenceTransformation {
        transformation_id: unique_id("evidence-transform"),
        source_evidence_id: evidence_id.clone(),
        transformation_type: "proof-normalization".into(),
        created_at: "2026-09-07T10:01:00Z".into(),
        created_by: "proof-runner".into(),
        tool_id: Some("tool-proof".into()),
        tool_version: Some("1.0".into()),
        input_hash: None,
        output_hash: None,
    })
    .unwrap();
    let provenance = Provenance {
        provenance_id,
        actor_ref: None,
        source_refs: vec![evidence_ref.clone()],
        input_refs: vec![case_ref.clone()],
        operation: "case.evidence.capture".into(),
        occurred_at: "2026-09-07T10:01:00Z".into(),
    };

    let mut factory = PostgresUnitOfWorkFactory::new(url.clone());
    let mut uow = factory.begin().unwrap();
    uow.execute(|ctx| {
        ctx.write_resource(ResourceWrite::new(
            case_ref.clone(),
            1,
            serde_json::to_value(sidereth_core::Case::new(case_id.clone()).unwrap()).unwrap(),
            ResourceWriteMode::Insert,
        )?)?;

        let mut evidence_repo = EvidenceTrustUnitOfWorkRepository::new(ctx);
        evidence_repo.create(evidence.clone())?;

        provenance.validate().unwrap();
        ctx.write_resource(ResourceWrite::new(
            provenance_ref.clone(),
            1,
            serde_json::to_value(provenance.clone()).unwrap(),
            ResourceWriteMode::Insert,
        )?)?;
        Ok(())
    })
    .unwrap();
    uow.commit().unwrap();

    let mut verify = PostgresUnitOfWorkFactory::new(url);
    let mut read_uow = verify.begin().unwrap();
    let records = read_uow
        .execute(|ctx| {
            let case = ctx.read_resource(&case_ref)?;
            let evidence = ctx.read_resource(&evidence_ref)?;
            let provenance = ctx.read_resource(&provenance_ref)?;
            Ok::<_, sidereth_core::persistence::UnitOfWorkError>((case, evidence, provenance))
        })
        .unwrap();
    read_uow.commit().unwrap();

    assert!(records.0.is_some(), "case must persist after commit");
    let persisted_evidence = records.1.expect("evidence must persist after commit");
    let decoded: sidereth_core::PersistedEvidence =
        serde_json::from_value(persisted_evidence.payload).unwrap();
    assert_eq!(decoded, evidence);
    assert!(records.2.is_some(), "provenance must persist after commit");
}

#[test]
#[ignore = "requires live PostgreSQL"]
fn live_postgres_case_and_evidence_trust_rollback_is_atomic() {
    let url = database_url();
    let mut setup = PostgresUnitOfWorkFactory::new(url.clone());
    prepare(&mut setup);

    let case_id = unique_id("evidence-rollback-case");
    let evidence_id = unique_id("evidence-rollback");
    let provenance_id = unique_id("evidence-rollback-provenance");
    let case_ref = ResourceRef::new(ResourceType::Case, case_id.clone()).unwrap();
    let evidence_ref = ResourceRef::new(ResourceType::Evidence, evidence_id.clone()).unwrap();
    let provenance_ref = ResourceRef::new(ResourceType::Provenance, provenance_id.clone()).unwrap();
    let evidence = sidereth_core::PersistedEvidence::new(
        1,
        evidence_for(&case_id, &evidence_id),
        trust_metadata(),
    )
    .unwrap();
    let provenance = Provenance {
        provenance_id,
        actor_ref: None,
        source_refs: vec![evidence_ref.clone()],
        input_refs: vec![case_ref.clone()],
        operation: "case.evidence.capture".into(),
        occurred_at: "2026-09-07T10:02:00Z".into(),
    };

    let mut factory = PostgresUnitOfWorkFactory::new(url.clone());
    let mut uow = factory.begin().unwrap();
    let result = uow.execute(|ctx| {
        ctx.write_resource(ResourceWrite::new(
            case_ref.clone(),
            1,
            serde_json::to_value(sidereth_core::Case::new(case_id.clone()).unwrap()).unwrap(),
            ResourceWriteMode::Insert,
        )?)?;

        let mut evidence_repo = EvidenceTrustUnitOfWorkRepository::new(ctx);
        evidence_repo.create(evidence)?;

        provenance.validate().unwrap();
        ctx.write_resource(ResourceWrite::new(
            provenance_ref.clone(),
            1,
            serde_json::to_value(provenance).unwrap(),
            ResourceWriteMode::Insert,
        )?)?;

        Err::<(), _>(PersistenceError::Conflict.into())
    });
    assert_eq!(
        result,
        Err(sidereth_core::persistence::UnitOfWorkError::Persistence(
            PersistenceError::Conflict
        ))
    );
    uow.rollback().unwrap();

    let mut verify = PostgresUnitOfWorkFactory::new(url);
    let mut read_uow = verify.begin().unwrap();
    let records = read_uow
        .execute(|ctx| {
            Ok::<_, sidereth_core::persistence::UnitOfWorkError>((
                ctx.read_resource(&case_ref)?,
                ctx.read_resource(&evidence_ref)?,
                ctx.read_resource(&provenance_ref)?,
            ))
        })
        .unwrap();
    read_uow.commit().unwrap();

    assert!(records.0.is_none(), "case must roll back");
    assert!(records.1.is_none(), "evidence trust state must roll back");
    assert!(records.2.is_none(), "provenance must roll back");
}
