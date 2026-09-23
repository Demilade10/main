//! V1 deployment-state fixtures. Seed canonical DataKey/contracttype values in
//! persistent storage to model an existing instance before invoking the public
//! upgrade entry point. No media, witness, or private key material is used.
#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::{Address as _, Events as _}, Address, BytesN, Env};

fn digest(env: &Env, byte: u8) -> BytesN<32> {
    BytesN::from_array(env, &[byte; 32])
}

struct Fixture {
    env: Env,
    id: Address,
    client: HarpocratesRegistryClient<'static>,
    admin: Address,
    proof_id: BytesN<32>,
    proof: ProofRecord,
    issuer: Address,
    root: BytesN<32>,
}

fn deployed_v1_fixture(version: Option<u32>) -> Fixture {
    let env = Env::default();
    env.mock_all_auths();
    let id = env.register(HarpocratesRegistry, ());
    let client = HarpocratesRegistryClient::new(&env, &id);
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let proof_id = digest(&env, 1);
    let root = digest(&env, 2);
    let proof = ProofRecord {
        video_hash: digest(&env, 3), metadata_hash: digest(&env, 4),
        tier: TIER_CONSISTENT_SOURCE, status: STATUS_REGISTERED,
        created_at: 100, expires_at: 0, source: None,
        issuer: Some(issuer.clone()), nullifier: Some(digest(&env, 5)), batch_size: 0,
    };
    env.as_contract(&id, || {
        let storage = env.storage().persistent();
        storage.set(&DataKey::Admin, &admin);
        if let Some(v) = version { storage.set(&DataKey::SchemaVersion, &v); }
        storage.set(&DataKey::Proof(proof_id.clone()), &proof);
        storage.set(&DataKey::Video(proof.video_hash.clone()), &proof_id);
        storage.set(&DataKey::Nullifier(proof.nullifier.clone().unwrap()), &true);
        storage.set(&DataKey::Issuer(issuer.clone()), &IssuerRecord {
            metadata_hash: digest(&env, 6), active: true,
        });
        storage.set(&DataKey::CredentialRoot(root.clone()), &CredentialRootRecord {
            metadata_hash: digest(&env, 7), active: false, issued_at: 99,
        });
    });
    Fixture { env, id, client, admin, proof_id, proof, issuer, root }
}

fn assert_evidence_preserved(f: &Fixture) {
    assert_eq!(f.client.get_proof(&f.proof_id), Some(f.proof.clone()));
    assert_eq!(f.client.get_by_video(&f.proof.video_hash), Some(f.proof.clone()));
    assert!(f.client.has_nullifier(f.proof.nullifier.as_ref().unwrap()));
    assert!(f.client.get_issuer(&f.issuer).unwrap().active);
    assert!(!f.client.get_credential_root(&f.root).unwrap().active);
}

#[test]
fn legacy_deployment_without_version_remains_readable_and_upgrade_is_idempotent() {
    let f = deployed_v1_fixture(None);
    assert_evidence_preserved(&f);
    let before = f.env.events().all().len();
    f.client.upgrade_storage(&f.admin);
    f.client.upgrade_storage(&f.admin);
    assert_evidence_preserved(&f);
    f.env.as_contract(&f.id, || {
        assert!(!f.env.storage().persistent().has(&DataKey::SchemaVersion));
    });
    assert_eq!(f.env.events().all().len(), before);
}

#[test]
fn versioned_deployment_and_future_version_are_never_rewritten() {
    for version in [SchemaVersion::V1 as u32, u32::MAX] {
        let f = deployed_v1_fixture(Some(version));
        let before = f.env.events().all().len();
        f.client.upgrade_storage(&f.admin);
        assert_evidence_preserved(&f);
        f.env.as_contract(&f.id, || {
            assert_eq!(f.env.storage().persistent().get::<_, u32>(&DataKey::SchemaVersion), Some(version));
        });
        assert_eq!(f.env.events().all().len(), before);
    }
}

#[test]
fn non_admin_cannot_upgrade_deployment_fixture() {
    let f = deployed_v1_fixture(None);
    let stranger = Address::generate(&f.env);
    assert_eq!(f.client.try_upgrade_storage(&stranger), Err(Ok(RegistryError::Unauthorized)));
    assert_evidence_preserved(&f);
    f.env.as_contract(&f.id, || {
        assert!(!f.env.storage().persistent().has(&DataKey::SchemaVersion));
    });
}
