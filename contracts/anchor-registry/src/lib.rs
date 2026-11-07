#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec,
};

const MAX_HISTORY_PAGE_SIZE: u32 = 50;

#[contracterror]
#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegistryError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    AnchorAlreadyRegistered = 4,
    UnknownAnchor = 5,
    InvalidScore = 6,
    InvalidTimestamp = 7,
    InvalidHistoryPage = 8,
    ArithmeticError = 9,
    NoAttestation = 10,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnchorMetadata {
    pub name: soroban_sdk::String,
    pub home_domain: soroban_sdk::String,
    pub asset_code: Symbol,
    pub country_code: Symbol,
    pub reporter: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HealthAttestation {
    pub score: u32,
    pub timestamp: u64,
    pub sequence: u32,
}

#[contracttype]
enum DataKey {
    Admin,
    Anchor(Symbol),
    AttestationCount(Symbol),
    Attestation(Symbol, u32),
    LatestAttestation(Symbol),
}

#[contract]
pub struct AnchorRegistry;

#[contractimpl]
impl AnchorRegistry {
    pub fn initialize(env: Env, admin: Address) -> Result<(), RegistryError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(RegistryError::AlreadyInitialized);
        }

        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        Ok(())
    }

    pub fn register_anchor(
        env: Env,
        admin: Address,
        anchor_id: Symbol,
        metadata: AnchorMetadata,
    ) -> Result<(), RegistryError> {
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(RegistryError::NotInitialized)?;
        if admin != stored_admin {
            return Err(RegistryError::Unauthorized);
        }
        admin.require_auth();

        let key = DataKey::Anchor(anchor_id.clone());
        if env.storage().persistent().has(&key) {
            return Err(RegistryError::AnchorAlreadyRegistered);
        }

        env.storage().persistent().set(&key, &metadata);
        env.storage()
            .persistent()
            .set(&DataKey::AttestationCount(anchor_id.clone()), &0u32);
        env.events()
            .publish((symbol_short!("register"), anchor_id), metadata.reporter);
        Ok(())
    }

    pub fn get_anchor(env: Env, anchor_id: Symbol) -> Result<AnchorMetadata, RegistryError> {
        ensure_initialized(&env)?;
        env.storage()
            .persistent()
            .get(&DataKey::Anchor(anchor_id))
            .ok_or(RegistryError::UnknownAnchor)
    }

    pub fn attest_health(
        env: Env,
        reporter: Address,
        anchor_id: Symbol,
        score: u32,
        timestamp: u64,
    ) -> Result<HealthAttestation, RegistryError> {
        ensure_initialized(&env)?;
        let metadata: AnchorMetadata = env
            .storage()
            .persistent()
            .get(&DataKey::Anchor(anchor_id.clone()))
            .ok_or(RegistryError::UnknownAnchor)?;
        if reporter != metadata.reporter {
            return Err(RegistryError::Unauthorized);
        }
        reporter.require_auth();

        if score > 100 {
            return Err(RegistryError::InvalidScore);
        }
        if timestamp > env.ledger().timestamp() {
            return Err(RegistryError::InvalidTimestamp);
        }

        let latest_key = DataKey::LatestAttestation(anchor_id.clone());
        let count_key = DataKey::AttestationCount(anchor_id.clone());
        let sequence: u32 = env.storage().persistent().get(&count_key).unwrap_or(0);
        if let Some(latest) = env
            .storage()
            .persistent()
            .get::<_, HealthAttestation>(&latest_key)
        {
            if timestamp <= latest.timestamp {
                return Err(RegistryError::InvalidTimestamp);
            }
        }

        let next_sequence = sequence
            .checked_add(1)
            .ok_or(RegistryError::ArithmeticError)?;
        let attestation = HealthAttestation {
            score,
            timestamp,
            sequence,
        };

        env.storage().persistent().set(
            &DataKey::Attestation(anchor_id.clone(), sequence),
            &attestation,
        );
        env.storage().persistent().set(&latest_key, &attestation);
        env.storage().persistent().set(&count_key, &next_sequence);
        env.events().publish(
            (symbol_short!("health"), anchor_id),
            (score, timestamp, sequence),
        );
        Ok(attestation)
    }

    pub fn get_health(env: Env, anchor_id: Symbol) -> Result<HealthAttestation, RegistryError> {
        ensure_initialized(&env)?;
        if !env
            .storage()
            .persistent()
            .has(&DataKey::Anchor(anchor_id.clone()))
        {
            return Err(RegistryError::UnknownAnchor);
        }
        env.storage()
            .persistent()
            .get(&DataKey::LatestAttestation(anchor_id))
            .ok_or(RegistryError::NoAttestation)
    }

    pub fn health_history(
        env: Env,
        anchor_id: Symbol,
        from: u32,
        limit: u32,
    ) -> Result<Vec<HealthAttestation>, RegistryError> {
        ensure_initialized(&env)?;
        let count: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::AttestationCount(anchor_id.clone()))
            .ok_or(RegistryError::UnknownAnchor)?;
        if limit == 0 || limit > MAX_HISTORY_PAGE_SIZE || from > count {
            return Err(RegistryError::InvalidHistoryPage);
        }

        let mut history = Vec::new(&env);
        let end = from.saturating_add(limit).min(count);
        for sequence in from..end {
            let attestation: HealthAttestation = env
                .storage()
                .persistent()
                .get(&DataKey::Attestation(anchor_id.clone(), sequence))
                .ok_or(RegistryError::UnknownAnchor)?;
            history.push_back(attestation);
        }
        Ok(history)
    }

    pub fn get_attestation_count(env: Env, anchor_id: Symbol) -> Result<u32, RegistryError> {
        ensure_initialized(&env)?;
        env.storage()
            .persistent()
            .get(&DataKey::AttestationCount(anchor_id))
            .ok_or(RegistryError::UnknownAnchor)
    }
}

fn ensure_initialized(env: &Env) -> Result<(), RegistryError> {
    if env.storage().instance().has(&DataKey::Admin) {
        Ok(())
    } else {
        Err(RegistryError::NotInitialized)
    }
}

#[cfg(test)]
mod test;