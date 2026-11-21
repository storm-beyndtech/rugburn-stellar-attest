#![no_std]

use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, Address, BytesN, Env,
    Symbol,
};

const TTL_RENEW_BELOW: u32 = 900_000;
const TTL_TARGET: u32 = 1_555_200;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    InvalidScore = 3,
    NotFound = 4,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Attestation {
    pub score: u32,
    pub report_hash: BytesN<32>,
    pub scanned_at: u64,
    pub score_version: Symbol,
    pub revoked: bool,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Attested {
    #[topic]
    pub asset_key: BytesN<32>,
    pub score: u32,
    pub scanned_at: u64,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Revoked {
    #[topic]
    pub asset_key: BytesN<32>,
}

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
    Attest(BytesN<32>),
}

#[contract]
pub struct AttestationRegistry;

#[contractimpl]
impl AttestationRegistry {
    pub fn __constructor(env: Env, admin: Address) {
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    pub fn attest(
        env: Env,
        asset_key: BytesN<32>,
        score: u32,
        report_hash: BytesN<32>,
        scanned_at: u64,
        score_version: Symbol,
    ) -> Result<(), Error> {
        Self::admin(env.clone()).require_auth();
        if score > 100 {
            return Err(Error::InvalidScore);
        }

        store(
            &env,
            &DataKey::Attest(asset_key.clone()),
            &Attestation {
                score,
                report_hash,
                scanned_at,
                score_version,
                revoked: false,
            },
        );
        Attested {
            asset_key,
            score,
            scanned_at,
        }
        .publish(&env);
        Ok(())
    }

    pub fn revoke(env: Env, asset_key: BytesN<32>) -> Result<(), Error> {
        Self::admin(env.clone()).require_auth();
        let key = DataKey::Attest(asset_key.clone());
        let mut attestation: Attestation = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::NotFound)?;

        attestation.revoked = true;
        store(&env, &key, &attestation);
        Revoked { asset_key }.publish(&env);
        Ok(())
    }

    pub fn get(env: Env, asset_key: BytesN<32>) -> Result<Attestation, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Attest(asset_key))
            .ok_or(Error::NotFound)
    }

    pub fn set_admin(env: Env, new_admin: Address) -> Result<(), Error> {
        Self::admin(env.clone()).require_auth();
        env.storage().instance().set(&DataKey::Admin, &new_admin);
        Ok(())
    }

    pub fn admin(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Admin).unwrap()
    }
}

fn store(env: &Env, key: &DataKey, attestation: &Attestation) {
    let entries = env.storage().persistent();
    entries.set(key, attestation);
    entries.extend_ttl(key, TTL_RENEW_BELOW, TTL_TARGET);
}

#[cfg(test)]
mod test;
