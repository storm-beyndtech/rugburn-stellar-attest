extern crate std;

use super::*;
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, EnvTestConfig, MockAuth, MockAuthInvoke},
    Bytes, IntoVal,
};

fn env() -> Env {
    Env::new_with_config(EnvTestConfig {
        capture_snapshot_at_drop: false,
    })
}

fn setup(env: &Env) -> (AttestationRegistryClient<'_>, Address) {
    let admin = Address::generate(env);
    let contract = match std::env::var_os("RUGBURN_TEST_WASM") {
        Some(path) => {
            let wasm = std::fs::read(path).expect("read RUGBURN_TEST_WASM");
            env.register(wasm.as_slice(), (&admin,))
        }
        None => env.register(AttestationRegistry, (&admin,)),
    };
    let client = AttestationRegistryClient::new(env, &contract);
    (client, admin)
}

fn bytes(env: &Env, byte: u8) -> BytesN<32> {
    BytesN::from_array(env, &[byte; 32])
}

#[test]
fn constructor_sets_the_publisher_at_deployment() {
    let env = env();
    let (client, admin) = setup(&env);
    assert_eq!(client.admin(), admin);
}

#[test]
fn publishes_the_full_record_for_anonymous_readers() {
    let env = env();
    let (client, _) = setup(&env);
    let key = bytes(&env, 1);
    let report_hash = bytes(&env, 9);
    let version = symbol_short!("v0_1");

    env.mock_all_auths();
    client.attest(&key, &45, &report_hash, &1_760_000_000, &version);
    env.set_auths(&[]);

    assert_eq!(
        client.get(&key),
        Attestation {
            score: 45,
            report_hash,
            scanned_at: 1_760_000_000,
            score_version: version,
            revoked: false,
        }
    );
}

#[test]
fn revocation_preserves_the_report_and_a_refresh_clears_it() {
    let env = env();
    let (client, _) = setup(&env);
    let key = bytes(&env, 2);
    let first_hash = bytes(&env, 8);
    let next_hash = bytes(&env, 7);
    env.mock_all_auths();

    client.attest(&key, &30, &first_hash, &100, &symbol_short!("v0_1"));
    client.revoke(&key);
    let revoked = client.get(&key);
    assert!(revoked.revoked);
    assert_eq!(revoked.report_hash, first_hash);
    assert_eq!(revoked.scanned_at, 100);

    client.attest(&key, &74, &next_hash, &200, &symbol_short!("v0_2"));
    let latest = client.get(&key);
    assert!(!latest.revoked);
    assert_eq!(latest.score, 74);
    assert_eq!(latest.scanned_at, 200);
    assert_eq!(latest.report_hash, next_hash);
}

#[test]
fn missing_record_is_not_a_revocation() {
    let env = env();
    let (client, _) = setup(&env);
    let key = bytes(&env, 3);
    assert_eq!(client.try_get(&key), Err(Ok(Error::NotFound)));
    env.mock_all_auths();
    assert_eq!(client.try_revoke(&key), Err(Ok(Error::NotFound)));
}

#[test]
fn rejects_scores_above_one_hundred_without_writing() {
    let env = env();
    let (client, _) = setup(&env);
    let key = bytes(&env, 4);
    env.mock_all_auths();
    assert_eq!(
        client.try_attest(&key, &101, &bytes(&env, 5), &1, &symbol_short!("v0_1")),
        Err(Ok(Error::InvalidScore))
    );
    assert_eq!(client.try_get(&key), Err(Ok(Error::NotFound)));
}

#[test]
fn rejects_publication_without_publisher_auth() {
    let env = env();
    let (client, _) = setup(&env);
    let key = bytes(&env, 6);

    assert!(matches!(
        client.try_attest(&key, &50, &bytes(&env, 7), &1, &symbol_short!("v0_1")),
        Err(Err(_))
    ));
    assert_eq!(client.try_get(&key), Err(Ok(Error::NotFound)));
}

#[test]
fn rejects_publication_authorized_by_a_different_account() {
    let env = env();
    let (client, _) = setup(&env);
    let outsider = Address::generate(&env);
    let key = bytes(&env, 8);
    let hash = bytes(&env, 9);
    let version = symbol_short!("v0_1");
    let args = (key.clone(), 50u32, hash.clone(), 1u64, version.clone()).into_val(&env);
    let attempted = client
        .mock_auths(&[MockAuth {
            address: &outsider,
            invoke: &MockAuthInvoke {
                contract: &client.address,
                fn_name: "attest",
                args,
                sub_invokes: &[],
            },
        }])
        .try_attest(&key, &50, &hash, &1, &version);

    assert!(matches!(attempted, Err(Err(_))));
    assert_eq!(client.try_get(&key), Err(Ok(Error::NotFound)));
}

#[test]
fn rejects_revocation_without_publisher_auth() {
    let env = env();
    let (client, _) = setup(&env);
    let key = bytes(&env, 10);
    env.mock_all_auths();
    client.attest(&key, &50, &bytes(&env, 11), &1, &symbol_short!("v0_1"));
    env.set_auths(&[]);

    assert!(matches!(client.try_revoke(&key), Err(Err(_))));
    assert!(!client.get(&key).revoked);
}

#[test]
fn rotates_the_publisher_only_with_existing_publisher_auth() {
    let env = env();
    let (client, admin) = setup(&env);
    let next = Address::generate(&env);
    assert!(matches!(client.try_set_admin(&next), Err(Err(_))));
    assert_eq!(client.admin(), admin);

    env.mock_all_auths();
    client.set_admin(&next);
    assert_eq!(client.admin(), next);
}

#[test]
fn commits_to_the_exact_bytes_of_both_passport_fixtures() {
    let env = env();
    let (client, _) = setup(&env);
    let key = bytes(&env, 12);
    let version = symbol_short!("v1");
    let before: BytesN<32> = env
        .crypto()
        .sha256(&Bytes::from_slice(
            &env,
            include_bytes!("../examples/passport-before.json"),
        ))
        .into();
    let after: BytesN<32> = env
        .crypto()
        .sha256(&Bytes::from_slice(
            &env,
            include_bytes!("../examples/passport-after.json"),
        ))
        .into();
    assert_ne!(before, after);

    env.mock_all_auths();
    client.attest(&key, &45, &before, &1_760_000_000, &version);
    assert_eq!(client.get(&key).report_hash, before);
    client.attest(&key, &45, &after, &1_760_000_000, &version);
    assert_eq!(client.get(&key).report_hash, after);
}
