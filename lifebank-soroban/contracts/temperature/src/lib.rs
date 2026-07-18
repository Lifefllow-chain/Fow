#![no_std]

mod error;
mod storage;
mod types;

use crate::error::ContractError;
use crate::types::{
    BatchOutcome, DataKey, DeviceInfo, ExcursionSummary, ProductThreshold, RawReading,
    ShipmentEvidence, TemperatureReading, TemperatureSummary, TemperatureThreshold,
};
use soroban_sdk::{
    contract, contractclient, contractimpl, contracttype, symbol_short, Address, Bytes, BytesN,
    Env, Vec,
};

const PAGE_SIZE: u32 = 20;

// ── Oracle authentication (#41) ───────────────────────────────────────────────

/// How far a reading's timestamp may sit ahead of current ledger time before
/// it's rejected as implausible clock skew.
const MAX_FUTURE_SKEW_SECONDS: u64 = 300;
/// How far a reading's timestamp may predate the shipment's start before
/// it's rejected as outside the shipment window.
const MAX_PAST_SKEW_SECONDS: u64 = 86_400;
/// Physically impossible cold-chain bounds (±80.00°C), distinct from and far
/// wider than any product's business threshold — these catch corrupted or
/// spoofed sensor values, not ordinary excursions.
const PLAUSIBLE_MIN_CELSIUS_X100: i32 = -8_000;
const PLAUSIBLE_MAX_CELSIUS_X100: i32 = 8_000;
/// Number of implausible batches after which a device is quarantined.
const IMPLAUSIBLE_QUARANTINE_THRESHOLD: u32 = 3;

#[contract]
pub struct TemperatureContract;

/// Minimal coordinator interface for cross-contract excursion reporting.
#[contractclient(name = "CoordinatorContractClient")]
pub trait CoordinatorContractInterface {
    fn flag_temperature_breach(
        env: soroban_sdk::Env,
        caller: Address,
        payment_id: u64,
        excursion_summary: ExcursionSummary,
    ) -> Result<(), soroban_sdk::Error>;
}

#[contractimpl]
impl TemperatureContract {
    pub fn initialize(env: Env, admin: Address) -> Result<(), ContractError> {
        admin.require_auth();

        if env.storage().instance().has(&DataKey::Admin) {
            return Err(ContractError::AlreadyInitialized);
        }

        storage::set_admin(&env, &admin);
        Ok(())
    }

    /// Pause all state-mutating functions. Admin only.
    pub fn pause(env: Env, admin: Address) -> Result<(), ContractError> {
        admin.require_auth();
        let stored_admin = storage::get_admin(&env);
        if admin != stored_admin {
            return Err(ContractError::Unauthorized);
        }
        env.storage().instance().set(&DataKey::Paused, &true);
        Ok(())
    }

    /// Unpause the contract. Admin only.
    pub fn unpause(env: Env, admin: Address) -> Result<(), ContractError> {
        admin.require_auth();
        let stored_admin = storage::get_admin(&env);
        if admin != stored_admin {
            return Err(ContractError::Unauthorized);
        }
        env.storage().instance().set(&DataKey::Paused, &false);
        Ok(())
    }

    /// Returns whether the contract is currently paused.
    pub fn is_paused(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false)
    }

    fn require_not_paused(env: &Env) -> Result<(), ContractError> {
        if env
            .storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false)
        {
            return Err(ContractError::ContractPaused);
        }
        Ok(())
    }

    pub fn set_threshold(
        env: Env,
        admin: Address,
        unit_id: u64,
        min_celsius_x100: i32,
        max_celsius_x100: i32,
    ) -> Result<(), ContractError> {
        admin.require_auth();
        Self::require_not_paused(&env)?;

        let stored_admin = storage::get_admin(&env);
        if admin != stored_admin {
            return Err(ContractError::Unauthorized);
        }

        if min_celsius_x100 >= max_celsius_x100 {
            return Err(ContractError::InvalidThreshold);
        }

        let threshold = TemperatureThreshold {
            min_celsius_x100,
            max_celsius_x100,
        };
        storage::set_threshold(&env, unit_id, &threshold);
        Ok(())
    }

    pub fn log_reading(
        env: Env,
        unit_id: u64,
        temperature_celsius_x100: i32,
        timestamp: u64,
    ) -> Result<(), ContractError> {
        Self::require_not_paused(&env)?;
        let threshold =
            storage::get_threshold(&env, unit_id).ok_or(ContractError::ThresholdNotFound)?;

        let is_violation =
            temperature_celsius_x100 < threshold.min_celsius_x100
                || temperature_celsius_x100 > threshold.max_celsius_x100;

        let reading = TemperatureReading {
            temperature_celsius_x100,
            timestamp,
            is_violation,
        };

        // Update consecutive violation streak
        let streak_key = DataKey::ConsecutiveViolationStreak(unit_id);
        let current_streak: u32 = env.storage().persistent().get(&streak_key).unwrap_or(0);
        
        let new_streak = if is_violation {
            current_streak.saturating_add(1)
        } else {
            0 // Reset streak on non-violation
        };
        
        env.storage().persistent().set(&streak_key, &new_streak);
        
        // Check if unit should be compromised (3 consecutive violations)
        if new_streak >= 3 {
            let compromised_key = DataKey::IsCompromised(unit_id);
            env.storage().persistent().set(&compromised_key, &true);
        }

        let mut page_num: u32 = 0;
        let position: u32;

        loop {
            let len = storage::get_temp_page_len(&env, unit_id, page_num);
            if len == 0 && page_num > 0 {
                position = 0;
                break;
            }
            if len < PAGE_SIZE {
                position = len;
                break;
            }
            page_num = page_num.saturating_add(1); // Prevent overflow
        }

        let mut page = storage::get_temp_page(&env, unit_id, page_num);

        while page.len() < position {
            page.push_back(TemperatureReading::default());
        }

        if page.len() == position {
            page.push_back(reading);
        } else {
            page.set(position, reading);
        }

        storage::set_temp_page(&env, unit_id, page_num, &page);
        storage::set_temp_page_len(&env, unit_id, page_num, position.saturating_add(1)); // Prevent overflow

        Ok(())
    }

    pub fn get_violations(env: Env, unit_id: u64) -> Result<Vec<TemperatureReading>, ContractError> {
        let mut violations = Vec::new(&env);
        let mut page_num: u32 = 0;
        
        loop {
            let page_len = storage::get_temp_page_len(&env, unit_id, page_num);
            if page_len == 0 && page_num > 0 {
                break;
            }
            if page_len == 0 {
                page_num = page_num.saturating_add(1); // Prevent overflow
                continue;
            }

            let page = storage::get_temp_page(&env, unit_id, page_num);
            for i in 0..page_len {
                let reading = page.get(i).unwrap_or_default();
                if reading.is_violation {
                    violations.push_back(reading);
                }
            }

            page_num = page_num.saturating_add(1); // Prevent overflow
        }

        Ok(violations)
    }

    /// Get all temperature readings for a blood unit
    pub fn get_readings(env: Env, unit_id: u64) -> Result<Vec<TemperatureReading>, ContractError> {
        let mut all_readings = Vec::new(&env);

        let mut page_num: u32 = 0;
        loop {
            // Get the stored length for this page
            let page_len = storage::get_temp_page_len(&env, unit_id, page_num);

            // If page_len is 0 and we've checked pages before, we're done
            if page_len == 0 && page_num > 0 {
                break;
            }

            // If no entries in this page yet, try next page
            if page_len == 0 {
                page_num = page_num.saturating_add(1); // Prevent overflow
                continue;
            }

            // Get the page
            let page = storage::get_temp_page(&env, unit_id, page_num);

            // Only iterate up to the stored length, not the full page size
            for i in 0..page_len {
                let reading = page.get(i).unwrap_or_default();
                all_readings.push_back(reading);
            }

            page_num = page_num.saturating_add(1); // Prevent overflow
        }

        Ok(all_readings)
    }

    /// Get temperature summary statistics for a blood unit
    /// Uses i64 accumulator to prevent overflow with large datasets
    pub fn get_temperature_summary(env: Env, unit_id: u64) -> Result<TemperatureSummary, ContractError> {
        let mut count: u32 = 0;
        let mut sum: i64 = 0; // Use i64 to prevent overflow
        let mut min_temp: i32 = i32::MAX;
        let mut max_temp: i32 = i32::MIN;
        let mut violation_count: u32 = 0;

        let mut page_num: u32 = 0;
        loop {
            let page_len = storage::get_temp_page_len(&env, unit_id, page_num);

            if page_len == 0 && page_num > 0 {
                break;
            }

            if page_len == 0 {
                page_num = page_num.saturating_add(1); // Prevent overflow
                continue;
            }

            let page = storage::get_temp_page(&env, unit_id, page_num);

            for i in 0..page_len {
                let reading = page.get(i).unwrap_or_default();
                
                // Use i64 for accumulation to prevent overflow
                sum += reading.temperature_celsius_x100 as i64;
                count = count.saturating_add(1); // Prevent overflow

                if reading.temperature_celsius_x100 < min_temp {
                    min_temp = reading.temperature_celsius_x100;
                }
                if reading.temperature_celsius_x100 > max_temp {
                    max_temp = reading.temperature_celsius_x100;
                }
                if reading.is_violation {
                    violation_count = violation_count.saturating_add(1); // Prevent overflow
                }
            }

            page_num = page_num.saturating_add(1); // Prevent overflow
        }

        if count == 0 {
            return Err(ContractError::UnitNotFound);
        }

        // Safe to cast back to i32 after division since individual readings fit in i32
        let avg_celsius_x100 = (sum / count as i64) as i32;

        Ok(TemperatureSummary {
            count,
            avg_celsius_x100,
            min_celsius_x100: min_temp,
            max_celsius_x100: max_temp,
            violation_count,
        })
    }

    /// Get the current consecutive violation streak for a blood unit
    ///
    /// # Arguments
    /// * `unit_id` - The blood unit to check
    ///
    /// # Returns
    /// Current consecutive violation count
    pub fn get_consecutive_violation_streak(env: Env, unit_id: u64) -> u32 {
        let streak_key = DataKey::ConsecutiveViolationStreak(unit_id);
        env.storage().persistent().get(&streak_key).unwrap_or(0)
    }

    /// Check if a blood unit has been compromised due to consecutive violations
    ///
    /// # Arguments
    /// * `unit_id` - The blood unit to check
    ///
    /// # Returns
    /// `true` if unit has 3 or more consecutive violations (compromised), `false` otherwise
    pub fn is_compromised(env: Env, unit_id: u64) -> bool {
        let compromised_key = DataKey::IsCompromised(unit_id);
        env.storage().persistent().get(&compromised_key).unwrap_or(false)
    }

    /// Reset the compromised status and violation streak for a blood unit (admin only)
    ///
    /// # Arguments
    /// * `admin` - Admin address performing the reset
    /// * `unit_id` - The blood unit to reset
    ///
    /// # Errors
    /// - `Unauthorized`: Caller is not the admin
    pub fn reset_compromised_status(
        env: Env,
        admin: Address,
        unit_id: u64,
    ) -> Result<(), ContractError> {
        admin.require_auth();
        Self::require_not_paused(&env)?;

        let stored_admin = storage::get_admin(&env);
        if admin != stored_admin {
            return Err(ContractError::Unauthorized);
        }

        let streak_key = DataKey::ConsecutiveViolationStreak(unit_id);
        let compromised_key = DataKey::IsCompromised(unit_id);

        env.storage().persistent().set(&streak_key, &0u32);
        env.storage().persistent().set(&compromised_key, &false);

        Ok(())
    }

    // ── Coordinator integration ────────────────────────────────────────────────

    /// Configure the coordinator contract address. Admin only.
    pub fn set_coordinator(
        env: Env,
        admin: Address,
        coordinator: Address,
    ) -> Result<(), ContractError> {
        admin.require_auth();
        let stored_admin = storage::get_admin(&env);
        if admin != stored_admin {
            return Err(ContractError::Unauthorized);
        }
        env.storage()
            .instance()
            .set(&DataKey::CoordinatorContract, &coordinator);
        Ok(())
    }

    /// Whitelist an IoT oracle address that may call report_excursion_to_coordinator.
    pub fn add_oracle(
        env: Env,
        admin: Address,
        oracle: Address,
    ) -> Result<(), ContractError> {
        admin.require_auth();
        let stored_admin = storage::get_admin(&env);
        if admin != stored_admin {
            return Err(ContractError::Unauthorized);
        }
        env.storage()
            .persistent()
            .set(&DataKey::OracleWhitelist(oracle), &true);
        Ok(())
    }

    /// Report a sustained temperature excursion to the coordinator contract,
    /// which will transition the linked payment from Locked → Disputed.
    ///
    /// Only the admin or a whitelisted IoT oracle may call this function.
    ///
    /// # Arguments
    /// * `caller`            - Admin or whitelisted oracle address
    /// * `unit_id`           - Blood unit that experienced the excursion
    /// * `payment_id`        - Payment ID to flag in the coordinator
    /// * `excursion_summary` - Structured summary of the excursion
    ///
    /// # Errors
    /// - `Unauthorized`          - Caller is not admin or whitelisted oracle
    /// - `CoordinatorNotSet`     - Coordinator address not configured
    /// - `CoordinatorCallFailed` - Cross-contract call to coordinator failed
    pub fn report_excursion_to_coordinator(
        env: Env,
        caller: Address,
        unit_id: u64,
        payment_id: u64,
        excursion_summary: ExcursionSummary,
    ) -> Result<(), ContractError> {
        caller.require_auth();
        Self::require_not_paused(&env)?;

        // Gate: caller must be admin or whitelisted oracle
        let stored_admin = storage::get_admin(&env);
        let is_admin = caller == stored_admin;
        let is_oracle: bool = env
            .storage()
            .persistent()
            .get(&DataKey::OracleWhitelist(caller.clone()))
            .unwrap_or(false);

        if !is_admin && !is_oracle {
            return Err(ContractError::Unauthorized);
        }

        let coordinator_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::CoordinatorContract)
            .ok_or(ContractError::CoordinatorNotSet)?;

        let coord_client = CoordinatorContractClient::new(&env, &coordinator_addr);
        coord_client
            .try_flag_temperature_breach(&caller, &payment_id, &excursion_summary)
            .map_err(|_| ContractError::CoordinatorCallFailed)?
            .map_err(|_| ContractError::CoordinatorCallFailed)?;

        env.events().publish(
            (soroban_sdk::symbol_short!("tmp_excur"),),
            (unit_id, payment_id, excursion_summary.violation_count),
        );

        Ok(())
    }

    // ── Oracle authentication (#41) ───────────────────────────────────────────
    //
    // Registered devices submit signed batches scoped to one shipment (blood
    // unit). Two independent layers of authorization gate every batch:
    //   1. `submitter.require_auth()` — the gateway/backend relaying the
    //      batch must hold the Stellar signing key registered for the device.
    //   2. `ed25519_verify` — the physical device must have signed the exact
    //      batch content, so a compromised gateway key alone can't forge
    //      readings for a device it doesn't hold the private key for.
    //
    // A batch's readings are only folded into the shipment's breach verdict
    // once both layers pass and the content clears the sanity checks below;
    // every batch — accepted or not — is committed to the shipment's evidence
    // hash chain so the on-chain record is a complete, tamper-evident log an
    // arbiter can replay off-chain raw data against.

    /// Register a device's on-chain identity: its authorized submitter
    /// address, its Ed25519 public key, and the shipment (blood unit) it may
    /// report readings for. Admin only.
    pub fn register_device(
        env: Env,
        admin: Address,
        device_id: u64,
        submitter: Address,
        pubkey: BytesN<32>,
        unit_id: u64,
    ) -> Result<(), ContractError> {
        admin.require_auth();
        Self::require_not_paused(&env)?;

        let stored_admin = storage::get_admin(&env);
        if admin != stored_admin {
            return Err(ContractError::Unauthorized);
        }

        if storage::get_device(&env, device_id).is_some() {
            return Err(ContractError::DeviceAlreadyRegistered);
        }

        storage::set_device(
            &env,
            device_id,
            &DeviceInfo {
                submitter,
                pubkey,
                shipment_id: unit_id,
                last_seq: 0,
                implausible_count: 0,
                quarantined: false,
            },
        );

        env.events()
            .publish((symbol_short!("dev_reg"),), (device_id, unit_id));

        Ok(())
    }

    /// Fetch a registered device's on-chain identity.
    pub fn get_device(env: Env, device_id: u64) -> Result<DeviceInfo, ContractError> {
        storage::get_device(&env, device_id).ok_or(ContractError::DeviceNotRegistered)
    }

    /// Publish a new versioned temperature threshold configuration for a
    /// product (e.g. whole blood / platelets / plasma). Admin only. Returns
    /// the new version number; existing shipments keep the version they were
    /// pinned to at `start_shipment`, so this never retroactively changes a
    /// shipment already in flight.
    pub fn set_product_threshold(
        env: Env,
        admin: Address,
        product_id: u64,
        min_celsius_x100: i32,
        max_celsius_x100: i32,
    ) -> Result<u32, ContractError> {
        admin.require_auth();
        Self::require_not_paused(&env)?;

        let stored_admin = storage::get_admin(&env);
        if admin != stored_admin {
            return Err(ContractError::Unauthorized);
        }

        if min_celsius_x100 >= max_celsius_x100 {
            return Err(ContractError::InvalidThreshold);
        }

        let version = storage::get_product_threshold_version(&env, product_id)
            .unwrap_or(0)
            .saturating_add(1);

        storage::set_product_threshold(
            &env,
            product_id,
            version,
            &ProductThreshold {
                min_celsius_x100,
                max_celsius_x100,
                version,
            },
        );
        storage::set_product_threshold_version(&env, product_id, version);

        env.events()
            .publish((symbol_short!("thr_ver"),), (product_id, version));

        Ok(version)
    }

    /// Start a shipment (blood unit) for oracle-authenticated tracking,
    /// pinning the product's *current* threshold version so later calls to
    /// `set_product_threshold` cannot retroactively change this shipment's
    /// verdict. Admin only.
    pub fn start_shipment(
        env: Env,
        admin: Address,
        unit_id: u64,
        product_id: u64,
    ) -> Result<u32, ContractError> {
        admin.require_auth();
        Self::require_not_paused(&env)?;

        let stored_admin = storage::get_admin(&env);
        if admin != stored_admin {
            return Err(ContractError::Unauthorized);
        }

        if storage::get_shipment_threshold_version(&env, unit_id).is_some() {
            return Err(ContractError::ShipmentAlreadyStarted);
        }

        let version = storage::get_product_threshold_version(&env, product_id)
            .ok_or(ContractError::ProductThresholdNotFound)?;

        storage::set_shipment_threshold_version(&env, unit_id, version);
        storage::set_shipment_product(&env, unit_id, product_id);
        storage::set_shipment_started_at(&env, unit_id, env.ledger().timestamp());

        env.events()
            .publish((symbol_short!("ship_st"),), (unit_id, product_id, version));

        Ok(version)
    }

    /// Recompute the sha256 commitment over a batch's declared content,
    /// mirroring what a registered device signs offline: `device_id ||
    /// seq_start || seq_end || (temperature || timestamp)*`.
    fn hash_batch_payload(
        env: &Env,
        device_id: u64,
        seq_start: u64,
        seq_end: u64,
        readings: &Vec<RawReading>,
    ) -> BytesN<32> {
        let mut data = Bytes::new(env);
        data.append(&Bytes::from_array(env, &device_id.to_be_bytes()));
        data.append(&Bytes::from_array(env, &seq_start.to_be_bytes()));
        data.append(&Bytes::from_array(env, &seq_end.to_be_bytes()));
        for reading in readings.iter() {
            data.append(&Bytes::from_array(
                env,
                &reading.temperature_celsius_x100.to_be_bytes(),
            ));
            data.append(&Bytes::from_array(env, &reading.timestamp.to_be_bytes()));
        }
        env.crypto().sha256(&data).into()
    }

    /// Ingest an Ed25519-signed batch of offline-buffered readings from a
    /// registered device.
    ///
    /// # Arguments
    /// * `submitter`     - The gateway/backend relaying this batch; must
    ///   match the device's registered submitter and authorize this call.
    /// * `device_id`     - Registered device identity.
    /// * `seq_start`/`seq_end` - Inclusive sequence range covered by `readings`;
    ///   must be strictly greater than the device's last accepted sequence,
    ///   rejecting replayed or reordered batches.
    /// * `readings`      - Raw readings covering `seq_start..=seq_end`.
    /// * `readings_hash` - sha256 commitment over the batch, as signed by the
    ///   device; the contract independently recomputes and checks it so a
    ///   relayer cannot pair a valid signature with substituted data.
    /// * `signature`     - Ed25519 signature by the device over
    ///   `device_id || seq_start || seq_end || readings_hash`.
    ///
    /// Every batch is committed to the shipment's evidence hash chain
    /// regardless of outcome. Only a batch whose readings are all within the
    /// ledger-time sanity window and physically plausible bounds is folded
    /// into the shipment's breach verdict; devices are quarantined after
    /// repeated implausible batches.
    ///
    /// # Errors
    /// - `EmptyBatch`/`InvalidSequenceRange` - Malformed batch bounds
    /// - `DeviceNotRegistered`   - No device under `device_id`
    /// - `DeviceQuarantined`     - Device was quarantined by a prior batch
    /// - `SubmitterMismatch`     - `submitter` isn't this device's registered submitter
    /// - `InvalidSequence`       - `seq_start` reuses or reorders a prior sequence
    /// - `ShipmentNotStarted`    - The device's shipment has no pinned threshold version
    /// - `ProductThresholdNotFound` - Pinned threshold version no longer resolvable
    /// - `ReadingsHashMismatch`  - Recomputed hash doesn't match `readings_hash`
    ///
    /// Panics (via the host's Ed25519 verifier) if `signature` doesn't match
    /// `readings_hash` under the device's registered public key.
    pub fn submit_reading_batch(
        env: Env,
        submitter: Address,
        device_id: u64,
        seq_start: u64,
        seq_end: u64,
        readings: Vec<RawReading>,
        readings_hash: BytesN<32>,
        signature: BytesN<64>,
    ) -> Result<BatchOutcome, ContractError> {
        submitter.require_auth();
        Self::require_not_paused(&env)?;

        if readings.is_empty() {
            return Err(ContractError::EmptyBatch);
        }
        let range_len = seq_end
            .checked_sub(seq_start)
            .and_then(|d| d.checked_add(1))
            .ok_or(ContractError::InvalidSequenceRange)?;
        if range_len != readings.len() as u64 {
            return Err(ContractError::InvalidSequenceRange);
        }

        let mut device = storage::get_device(&env, device_id).ok_or(ContractError::DeviceNotRegistered)?;
        if device.quarantined {
            return Err(ContractError::DeviceQuarantined);
        }
        if device.submitter != submitter {
            return Err(ContractError::SubmitterMismatch);
        }
        if seq_start <= device.last_seq {
            return Err(ContractError::InvalidSequence);
        }

        let unit_id = device.shipment_id;
        let threshold_version = storage::get_shipment_threshold_version(&env, unit_id)
            .ok_or(ContractError::ShipmentNotStarted)?;
        let product_id =
            storage::get_shipment_product(&env, unit_id).ok_or(ContractError::ShipmentNotStarted)?;
        let threshold = storage::get_product_threshold(&env, product_id, threshold_version)
            .ok_or(ContractError::ProductThresholdNotFound)?;

        let payload_hash = Self::hash_batch_payload(&env, device_id, seq_start, seq_end, &readings);
        if payload_hash != readings_hash {
            return Err(ContractError::ReadingsHashMismatch);
        }

        let mut message = Bytes::new(&env);
        message.append(&Bytes::from_array(&env, &device_id.to_be_bytes()));
        message.append(&Bytes::from_array(&env, &seq_start.to_be_bytes()));
        message.append(&Bytes::from_array(&env, &seq_end.to_be_bytes()));
        message.append(&Bytes::from_array(&env, &readings_hash.to_array()));
        env.crypto()
            .ed25519_verify(&device.pubkey, &message, &signature);

        // ── Commit to the evidence chain unconditionally ──────────────────
        let mut evidence = storage::get_shipment_evidence(&env, unit_id).unwrap_or(ShipmentEvidence {
            product_id,
            threshold_version,
            chain_hash: BytesN::from_array(&env, &[0u8; 32]),
            batch_count: 0,
        });
        let mut chain_input = Bytes::new(&env);
        chain_input.append(&Bytes::from_array(&env, &evidence.chain_hash.to_array()));
        chain_input.append(&Bytes::from_array(&env, &payload_hash.to_array()));
        evidence.chain_hash = env.crypto().sha256(&chain_input).into();
        evidence.batch_count = evidence.batch_count.saturating_add(1);
        storage::set_shipment_evidence(&env, unit_id, &evidence);

        device.last_seq = seq_end;

        // ── Reading integrity checks ───────────────────────────────────────
        let now = env.ledger().timestamp();
        let shipment_started_at = storage::get_shipment_started_at(&env, unit_id).unwrap_or(now);

        let mut timestamp_ok = true;
        let mut plausible_ok = true;
        for reading in readings.iter() {
            if reading.timestamp > now.saturating_add(MAX_FUTURE_SKEW_SECONDS)
                || reading.timestamp < shipment_started_at.saturating_sub(MAX_PAST_SKEW_SECONDS)
            {
                timestamp_ok = false;
            }
            if reading.temperature_celsius_x100 < PLAUSIBLE_MIN_CELSIUS_X100
                || reading.temperature_celsius_x100 > PLAUSIBLE_MAX_CELSIUS_X100
            {
                plausible_ok = false;
            }
        }

        let outcome = if !timestamp_ok {
            BatchOutcome::RejectedTimestamp
        } else if !plausible_ok {
            device.implausible_count = device.implausible_count.saturating_add(1);
            if device.implausible_count >= IMPLAUSIBLE_QUARANTINE_THRESHOLD {
                device.quarantined = true;
            }
            BatchOutcome::RejectedImplausible
        } else {
            let mut state = storage::get_shipment_breach_state(&env, unit_id).unwrap_or_default();
            for reading in readings.iter() {
                let is_violation = reading.temperature_celsius_x100 < threshold.min_celsius_x100
                    || reading.temperature_celsius_x100 > threshold.max_celsius_x100;
                if is_violation {
                    state.violation_count = state.violation_count.saturating_add(1);
                    if state.violation_count == 1 {
                        state.detected_at = reading.timestamp;
                    }
                    if reading.temperature_celsius_x100 > state.peak_celsius_x100 {
                        state.peak_celsius_x100 = reading.temperature_celsius_x100;
                    }
                }
            }
            storage::set_shipment_breach_state(&env, unit_id, &state);
            BatchOutcome::Accepted
        };

        storage::set_device(&env, device_id, &device);

        let outcome_code: u32 = match outcome {
            BatchOutcome::Accepted => 0,
            BatchOutcome::RejectedTimestamp => 1,
            BatchOutcome::RejectedImplausible => 2,
        };
        env.events().publish(
            (symbol_short!("batch"),),
            (device_id, unit_id, seq_start, seq_end, readings_hash, outcome_code),
        );

        if device.quarantined {
            env.events()
                .publish((symbol_short!("quarant"),), (device_id,));
        }

        Ok(outcome)
    }

    /// Read the shipment's current breach verdict, derived only from batches
    /// that passed timestamp and plausibility validation under its pinned
    /// threshold version.
    pub fn get_shipment_verdict(env: Env, unit_id: u64) -> Result<ExcursionSummary, ContractError> {
        let state = storage::get_shipment_breach_state(&env, unit_id)
            .ok_or(ContractError::ShipmentVerdictNotFound)?;
        Ok(ExcursionSummary {
            unit_id,
            violation_count: state.violation_count,
            peak_celsius_x100: state.peak_celsius_x100,
            detected_at: state.detected_at,
        })
    }

    /// Read the shipment's evidence anchor (pinned config version + rolling
    /// hash chain over every ingested batch) for off-chain arbiter verification.
    ///
    /// To independently reproduce a disputed verdict, an arbiter: (1) collects
    /// every `batch` event for `unit_id`, ordered by `seq_start`; (2) for each,
    /// recomputes `sha256(device_id || seq_start || seq_end || (temperature ||
    /// timestamp)*)` over the off-chain-held raw readings and checks it matches
    /// the event's `readings_hash`; (3) folds those hashes — starting from 32
    /// zero bytes — via `chain_hash' = sha256(chain_hash || readings_hash)` and
    /// checks the result and batch count match this anchor; (4) re-applies
    /// `get_product_threshold(product_id, threshold_version)` (the *pinned*
    /// version, not whatever is currently live) to every accepted batch's
    /// readings and confirms the result matches `get_shipment_verdict`.
    pub fn get_shipment_evidence(env: Env, unit_id: u64) -> Result<ShipmentEvidence, ContractError> {
        storage::get_shipment_evidence(&env, unit_id).ok_or(ContractError::ShipmentEvidenceNotFound)
    }

    // ── Upgradeability & versioned storage schema (#31) ──────────────────────

    /// Code version of the currently deployed binary.
    pub fn version(_env: Env) -> u32 {
        CONTRACT_VERSION
    }

    /// Storage schema version currently recorded on-chain (1 when unset).
    pub fn schema_version(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&SCHEMA_VERSION_KEY)
            .unwrap_or(1)
    }

    /// Replace the running WASM with an already-installed hash. Admin only.
    /// The contract ID and all storage are preserved; call `migrate` after
    /// the upgrade when the new binary bumps `TARGET_SCHEMA_VERSION`.
    pub fn upgrade(env: Env, new_wasm_hash: soroban_sdk::BytesN<32>) -> Result<(), ContractError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&crate::types::DataKey::Admin)
            .ok_or(ContractError::Unauthorized)?;
        admin.require_auth();
        env.deployer().update_current_contract_wasm(new_wasm_hash);
        Ok(())
    }

    /// Apply version-gated storage migrations after an upgrade. Admin only.
    /// Refuses to run once storage already sits at `TARGET_SCHEMA_VERSION`,
    /// so a migration can never be applied twice.
    pub fn migrate(env: Env) -> Result<u32, ContractError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&crate::types::DataKey::Admin)
            .ok_or(ContractError::Unauthorized)?;
        admin.require_auth();
        let current = Self::schema_version(env.clone());
        if current >= TARGET_SCHEMA_VERSION {
            return Err(ContractError::MigrationAlreadyApplied);
        }
        // Version-gated transformations run here as the schema evolves, e.g.
        // `if current < 2 { /* rewrite v1 entries into the v2 layout */ }`.
        env.storage()
            .instance()
            .set(&SCHEMA_VERSION_KEY, &TARGET_SCHEMA_VERSION);
        Ok(TARGET_SCHEMA_VERSION)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::testutils::Ledger as _;

    fn create_test_contract<'a>() -> (Env, Address, TemperatureContractClient<'a>) {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(TemperatureContract, ());
        let client = TemperatureContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.initialize(&admin);

        (env, admin, client)
    }

    #[test]
    fn test_zero_padded_entries_not_returned_as_violations() {
        let (_env, admin, client) = create_test_contract();

        let unit_id = 42u64;
        // Set threshold: min = 200 (2.00°C), max = 600 (6.00°C)
        client.set_threshold(&admin, &unit_id, &200, &600);

        // Log exactly 21 readings (one more than page size of 20)
        for i in 0..21u64 {
            let temp = 400 + (i % 3) as i32; // Vary between 400-402 (all within range)
            let timestamp = 1000 + i;
            client.log_reading(&unit_id, &temp, &timestamp);
        }

        // Get violations
        let violations = client.get_violations(&unit_id);

        // Should have zero violations since all logged readings are within threshold
        assert_eq!(violations.len(), 0, "Expected no violations but got {}", violations.len());
    }

    #[test]
    fn test_page_size_plus_one_with_violation_in_second_page() {
        let (_env, admin, client) = create_test_contract();

        let unit_id = 43u64;
        // Set threshold: min = 200 (2.00°C), max = 600 (6.00°C)
        client.set_threshold(&admin, &unit_id, &200, &600);

        // Log exactly 21 readings
        // First 20 readings: all within range
        for i in 0..20u64 {
            let temp = 400 + (i % 3) as i32; // Within 200-600 range
            let timestamp = 1000 + i;
            client.log_reading(&unit_id, &temp, &timestamp);
        }

        // 21st reading: a violation (too cold)
        client.log_reading(&unit_id, &100, &1020);

        // Get violations
        let violations = client.get_violations(&unit_id);

        // Should have exactly 1 violation
        assert_eq!(violations.len(), 1, "Expected 1 violation but got {}", violations.len());
        assert_eq!(violations.get(0).unwrap().temperature_celsius_x100, 100);
    }

    #[test]
    fn test_multiple_pages_correct_violation_count() {
        let (_env, admin, client) = create_test_contract();

        let unit_id = 44u64;
        // Set threshold: min = 200, max = 600
        client.set_threshold(&admin, &unit_id, &200, &600);

        // Log 50 readings across multiple pages
        let mut expected_violations = 0;
        for i in 0..50u64 {
            let temp = if i % 10 == 9 {
                // Every 10th reading is a violation (too hot)
                expected_violations += 1;
                700
            } else {
                400 // Within range
            };
            let timestamp = 1000 + i;
            client.log_reading(&unit_id, &temp, &timestamp);
        }

        // Get violations
        let violations = client.get_violations(&unit_id);

        // Should have exactly 5 violations (indices 9, 19, 29, 39, 49)
        assert_eq!(
            violations.len() as u64,
            expected_violations,
            "Expected {} violations but got {}",
            expected_violations,
            violations.len()
        );

        // Verify all returned readings are violations
        for violation in violations.iter() {
            let reading = violation;
            assert!(
                reading.is_violation,
                "Returned reading should be marked as violation"
            );
            assert!(
                reading.temperature_celsius_x100 < 200 || reading.temperature_celsius_x100 > 600,
                "Returned reading should actually violate threshold"
            );
        }
    }

    #[test]
    fn test_get_all_readings_ignores_padding() {
        let (_env, admin, client) = create_test_contract();

        let unit_id = 45u64;
        // Set threshold: min = 200, max = 600
        client.set_threshold(&admin, &unit_id, &200, &600);

        // Log exactly 21 readings
        for i in 0..21u64 {
            let temp = 400 + (i % 3) as i32;
            let timestamp = 1000 + i;
            client.log_reading(&unit_id, &temp, &timestamp);
        }

        // Get all readings
        let readings = client.get_readings(&unit_id);

        // Should have exactly 21 readings, not 40 (2 pages)
        assert_eq!(
            readings.len(),
            21,
            "Expected 21 readings but got {}",
            readings.len()
        );

        // Verify none are zero-padded (all should have valid timestamps)
        for reading in readings.iter() {
            assert!(
                reading.timestamp >= 1000 && reading.timestamp < 1021,
                "Reading should have valid timestamp from actual log"
            );
        }
    }

    #[test]
    fn test_threshold_violation_detection_with_zero_temp() {
        let (_env, admin, client) = create_test_contract();

        let unit_id = 46u64;
        // Set threshold: min = 200, max = 600
        client.set_threshold(&admin, &unit_id, &200, &600);

        // Log exactly 21 readings (21st will be in second page with padding)
        for i in 0..21u64 {
            let temp = 400;
            let timestamp = 1000 + i;
            client.log_reading(&unit_id, &temp, &timestamp);
        }

        // Verify the second page still exists but has no padding pollution
        let violations = client.get_violations(&unit_id);
        assert_eq!(violations.len(), 0, "No readings should be violations");

        let all_readings = client.get_readings(&unit_id);
        assert_eq!(all_readings.len(), 21, "Should have exactly 21 readings");

        // Verify the 21st reading is not a default/zero-padded entry
        let last_reading = all_readings.get(20).unwrap();
        assert_eq!(last_reading.temperature_celsius_x100, 400, "21st reading should be valid");
        assert_eq!(last_reading.timestamp, 1020, "21st reading should have correct timestamp");
    }

    #[test]
    fn test_temperature_summary_basic() {
        let (_env, admin, client) = create_test_contract();

        let unit_id = 100u64;
        client.set_threshold(&admin, &unit_id, &200, &600);

        // Log 10 readings: 5 at 400°C, 5 at 500°C
        // Average should be 450°C
        for i in 0..10u64 {
            let temp = if i < 5 { 400 } else { 500 };
            client.log_reading(&unit_id, &temp, &(1000 + i));
        }

        let summary = client.get_temperature_summary(&unit_id);
        assert_eq!(summary.count, 10);
        assert_eq!(summary.avg_celsius_x100, 450);
        assert_eq!(summary.min_celsius_x100, 400);
        assert_eq!(summary.max_celsius_x100, 500);
        assert_eq!(summary.violation_count, 0);
    }

    #[test]
    fn test_temperature_summary_with_violations() {
        let (_env, admin, client) = create_test_contract();

        let unit_id = 101u64;
        client.set_threshold(&admin, &unit_id, &200, &600);

        // Log readings with some violations
        client.log_reading(&unit_id, &100, &1000); // violation (too cold)
        client.log_reading(&unit_id, &400, &1001); // ok
        client.log_reading(&unit_id, &700, &1002); // violation (too hot)
        client.log_reading(&unit_id, &500, &1003); // ok

        let summary = client.get_temperature_summary(&unit_id);
        assert_eq!(summary.count, 4);
        assert_eq!(summary.avg_celsius_x100, 425); // (100 + 400 + 700 + 500) / 4
        assert_eq!(summary.min_celsius_x100, 100);
        assert_eq!(summary.max_celsius_x100, 700);
        assert_eq!(summary.violation_count, 2);
    }

    #[test]
    fn test_temperature_summary_large_dataset_no_overflow() {
        let (_env, admin, client) = create_test_contract();

        let unit_id = 102u64;
        client.set_threshold(&admin, &unit_id, &0, &60_000_000);

        // Keep this small enough for CI while still proving the accumulator
        // must be wider than i32: 30,000,000 * 100 = 3,000,000,000.
        let test_temp = 30_000_000i32;
        let num_readings = 100u64;

        for i in 0..num_readings {
            client.log_reading(&unit_id, &test_temp, &(1000 + i));
        }

        let summary = client.get_temperature_summary(&unit_id);
        
        // Verify correct count
        assert_eq!(summary.count, num_readings as u32, "Count should be 100");
        
        // Verify average is correct (should be exactly 450)
        assert_eq!(
            summary.avg_celsius_x100, 
            test_temp,
            "Average should be {} but got {}", 
            test_temp, 
            summary.avg_celsius_x100
        );
        
        // Verify min/max are correct
        assert_eq!(summary.min_celsius_x100, test_temp);
        assert_eq!(summary.max_celsius_x100, test_temp);
        assert_eq!(summary.violation_count, 0);
    }

    #[test]
    fn test_temperature_summary_extreme_values() {
        let (_env, admin, client) = create_test_contract();

        let unit_id = 103u64;
        client.set_threshold(&admin, &unit_id, &-5000, &5000);

        // Test with extreme temperature values
        client.log_reading(&unit_id, &-4000, &1000);
        client.log_reading(&unit_id, &4000, &1001);
        client.log_reading(&unit_id, &0, &1002);

        let summary = client.get_temperature_summary(&unit_id);
        assert_eq!(summary.count, 3);
        assert_eq!(summary.avg_celsius_x100, 0); // (-4000 + 4000 + 0) / 3 = 0
        assert_eq!(summary.min_celsius_x100, -4000);
        assert_eq!(summary.max_celsius_x100, 4000);
    }

    #[test]
    fn test_temperature_summary_multiple_pages() {
        let (_env, admin, client) = create_test_contract();

        let unit_id = 104u64;
        client.set_threshold(&admin, &unit_id, &200, &600);

        // Log 100 readings across multiple pages (PAGE_SIZE = 20)
        // This will span 5 pages
        for i in 0..100u64 {
            let temp = 300 + (i % 10) as i32; // Vary between 300-309
            client.log_reading(&unit_id, &temp, &(1000 + i));
        }

        let summary = client.get_temperature_summary(&unit_id);
        assert_eq!(summary.count, 100);
        
        // Average should be 304 (sum of 300-309 repeated 10 times / 100)
        // (300+301+302+303+304+305+306+307+308+309) * 10 / 100 = 3045 / 10 = 304.5 -> 304
        assert_eq!(summary.avg_celsius_x100, 304);
        assert_eq!(summary.min_celsius_x100, 300);
        assert_eq!(summary.max_celsius_x100, 309);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #601)")]
    fn test_temperature_summary_no_readings() {
        let (_env, admin, client) = create_test_contract();

        let unit_id = 105u64;
        client.set_threshold(&admin, &unit_id, &200, &600);

        // Don't log any readings
        client.get_temperature_summary(&unit_id);
    }

    // ============================================================================
    // Consecutive Violation Streak Tests
    // ============================================================================

    /// Test 1: Streak reset on non-violation
    /// 2 violations → 1 normal → 2 violations → assert streak is 2 (not 4) and unit is not Compromised
    #[test]
    fn test_streak_reset_on_non_violation() {
        let (_env, admin, client) = create_test_contract();

        let unit_id = 200u64;
        client.set_threshold(&admin, &unit_id, &200, &600);

        // Log 2 violations
        client.log_reading(&unit_id, &100, &1000); // violation 1
        client.log_reading(&unit_id, &100, &1001); // violation 2

        // Check streak is 2
        assert_eq!(client.get_consecutive_violation_streak(&unit_id), 2);
        assert!(!client.is_compromised(&unit_id));

        // Log 1 normal reading (resets streak)
        client.log_reading(&unit_id, &400, &1002); // normal

        // Check streak was reset to 0
        assert_eq!(client.get_consecutive_violation_streak(&unit_id), 0);

        // Log 2 more violations
        client.log_reading(&unit_id, &100, &1003); // violation 1
        client.log_reading(&unit_id, &100, &1004); // violation 2

        // Streak should be 2, not 4 (it was reset)
        assert_eq!(client.get_consecutive_violation_streak(&unit_id), 2);
        assert!(!client.is_compromised(&unit_id), "Unit should NOT be compromised with only 2 consecutive violations");
    }

    /// Test 2: Exact threshold - exactly 3 consecutive violations → assert Compromised triggered
    #[test]
    fn test_exact_threshold_triggers_compromised() {
        let (_env, admin, client) = create_test_contract();

        let unit_id = 201u64;
        client.set_threshold(&admin, &unit_id, &200, &600);

        // Log exactly 3 consecutive violations
        client.log_reading(&unit_id, &100, &1000); // violation 1
        assert_eq!(client.get_consecutive_violation_streak(&unit_id), 1);
        assert!(!client.is_compromised(&unit_id));

        client.log_reading(&unit_id, &100, &1001); // violation 2
        assert_eq!(client.get_consecutive_violation_streak(&unit_id), 2);
        assert!(!client.is_compromised(&unit_id));

        client.log_reading(&unit_id, &100, &1002); // violation 3
        assert_eq!(client.get_consecutive_violation_streak(&unit_id), 3);
        assert!(client.is_compromised(&unit_id), "Unit should be compromised after 3 consecutive violations");
    }

    /// Test 3: Threshold not met
    /// 2 consecutive → 1 normal → 2 consecutive → assert not Compromised
    #[test]
    fn test_threshold_not_met_not_compromised() {
        let (_env, admin, client) = create_test_contract();

        let unit_id = 202u64;
        client.set_threshold(&admin, &unit_id, &200, &600);

        // Log 2 consecutive violations
        client.log_reading(&unit_id, &100, &1000);
        client.log_reading(&unit_id, &100, &1001);
        assert_eq!(client.get_consecutive_violation_streak(&unit_id), 2);

        // Log 1 normal reading
        client.log_reading(&unit_id, &400, &1002);
        assert_eq!(client.get_consecutive_violation_streak(&unit_id), 0);

        // Log 2 more consecutive violations
        client.log_reading(&unit_id, &100, &1003);
        client.log_reading(&unit_id, &100, &1004);
        assert_eq!(client.get_consecutive_violation_streak(&unit_id), 2);

        // Should NOT be compromised
        assert!(!client.is_compromised(&unit_id), "Unit should NOT be compromised - never reached 3 consecutive");
    }

    /// Test 4: Streak after recovery
    /// unit is Compromised → admin resets → 2 new violations → assert not Compromised again yet
    #[test]
    fn test_streak_after_recovery() {
        let (_env, admin, client) = create_test_contract();

        let unit_id = 203u64;
        client.set_threshold(&admin, &unit_id, &200, &600);

        // Trigger compromised status with 3 violations
        client.log_reading(&unit_id, &100, &1000);
        client.log_reading(&unit_id, &100, &1001);
        client.log_reading(&unit_id, &100, &1002);
        assert!(client.is_compromised(&unit_id));
        assert_eq!(client.get_consecutive_violation_streak(&unit_id), 3);

        // Admin resets the status
        client.reset_compromised_status(&admin, &unit_id);
        assert!(!client.is_compromised(&unit_id), "Should be reset after admin intervention");
        assert_eq!(client.get_consecutive_violation_streak(&unit_id), 0, "Streak should be reset to 0");

        // Log 2 new violations
        client.log_reading(&unit_id, &100, &1003);
        client.log_reading(&unit_id, &100, &1004);
        assert_eq!(client.get_consecutive_violation_streak(&unit_id), 2);

        // Should NOT be compromised again yet (only 2 violations)
        assert!(!client.is_compromised(&unit_id), "Should not be compromised again with only 2 new violations");
    }

    /// Test 5: Single-reading unit
    /// 1 violation → assert streak is 1, not Compromised
    #[test]
    fn test_single_reading_unit() {
        let (_env, admin, client) = create_test_contract();

        let unit_id = 204u64;
        client.set_threshold(&admin, &unit_id, &200, &600);

        // Log single violation
        client.log_reading(&unit_id, &100, &1000);

        // Check streak is 1
        assert_eq!(client.get_consecutive_violation_streak(&unit_id), 1, "Streak should be 1 after single violation");

        // Should NOT be compromised
        assert!(!client.is_compromised(&unit_id), "Single violation should not compromise unit");
    }

    /// Test 6: Interleaved violations across custody transfers
    /// violations logged by different custodians → streak is continuous across custodian changes
    /// 
    /// Note: This test demonstrates that the streak tracking is based on the blood unit itself,
    /// not on who logs the reading. The custody transfer is simulated conceptually - in practice,
    /// any authorized party can log temperature readings, and the streak counter persists.
    #[test]
    fn test_interleaved_violations_across_custody_transfers() {
        let (_env, admin, client) = create_test_contract();

        let unit_id = 205u64;
        client.set_threshold(&admin, &unit_id, &200, &600);

        // Custodian A logs violations (e.g., during initial storage)
        client.log_reading(&unit_id, &100, &1000); // violation 1
        client.log_reading(&unit_id, &100, &1001); // violation 2
        assert_eq!(client.get_consecutive_violation_streak(&unit_id), 2);

        // Simulate custody transfer (conceptually - same unit, different handler)
        // Custodian B logs a violation (e.g., during transport)
        client.log_reading(&unit_id, &700, &1002); // violation 3 (too hot)
        
        // Streak should be continuous across the conceptual custody change
        assert_eq!(client.get_consecutive_violation_streak(&unit_id), 3);
        assert!(client.is_compromised(&unit_id), "Unit should be compromised - violations span custody transfer");

        // Custodian B logs a normal reading
        client.log_reading(&unit_id, &400, &1003); // normal
        
        // Streak should reset even after custody transfer
        assert_eq!(client.get_consecutive_violation_streak(&unit_id), 0);
        
        // Note: Unit remains compromised even after streak resets
        // (once compromised, always compromised until admin reset)
        assert!(client.is_compromised(&unit_id));
    }

    /// Test 7: Large streak
    /// 100 consecutive violations → assert Compromised triggered on the 3rd and streak value is 100 at end
    #[test]
    fn test_large_streak() {
        let (_env, admin, client) = create_test_contract();

        let unit_id = 206u64;
        client.set_threshold(&admin, &unit_id, &200, &600);

        // Log 100 consecutive violations
        for i in 0..100u64 {
            client.log_reading(&unit_id, &100, &(1000 + i));
            
            // Check that compromised was triggered on the 3rd violation
            if i == 2 {
                assert!(client.is_compromised(&unit_id), "Should be compromised on 3rd consecutive violation");
            }
        }

        // Final streak should be 100
        assert_eq!(client.get_consecutive_violation_streak(&unit_id), 100, "Streak should be 100 after 100 consecutive violations");
        
        // Should definitely be compromised
        assert!(client.is_compromised(&unit_id), "Unit should be compromised after 100 violations");
    }

    // ── Circuit breaker tests ─────────────────────────────────────────────────

    #[test]
    fn test_temperature_pause_blocks_log_reading() {
        let (_env, admin, client) = create_test_contract();
        let unit_id = 1u64;
        client.set_threshold(&admin, &unit_id, &200, &600);

        client.pause(&admin);
        assert!(client.is_paused());

        let result = client.try_log_reading(&unit_id, &400, &1000u64);
        assert!(result.is_err());
    }

    #[test]
    fn test_temperature_pause_allows_get_readings() {
        let (_env, admin, client) = create_test_contract();
        let unit_id = 2u64;
        client.set_threshold(&admin, &unit_id, &200, &600);
        client.log_reading(&unit_id, &400, &1000u64);

        client.pause(&admin);

        // Read still works
        let readings = client.get_readings(&unit_id);
        assert!(!readings.is_empty());
    }

    #[test]
    fn test_temperature_unpause_restores_writes() {
        let (_env, admin, client) = create_test_contract();
        let unit_id = 3u64;
        client.set_threshold(&admin, &unit_id, &200, &600);

        client.pause(&admin);
        client.unpause(&admin);
        assert!(!client.is_paused());

        client.log_reading(&unit_id, &400, &2000u64);
        let readings = client.get_readings(&unit_id);
        assert!(!readings.is_empty());
    }

    #[test]
    #[should_panic]
    fn test_temperature_non_admin_cannot_pause() {
        let (env, _admin, client) = create_test_contract();
        let attacker = Address::generate(&env);
        client.pause(&attacker);
    }

    // ── Oracle authentication tests (#41) ─────────────────────────────────────
    //
    // Signature/hash fixtures below were generated offline with a real
    // Ed25519 keypair (Node's built-in `crypto` module) over the exact byte
    // layout `hash_batch_payload`/`submit_reading_batch` compute on-chain:
    // `device_id || seq_start || seq_end || (temperature || timestamp)*` for
    // the hash, and `device_id || seq_start || seq_end || readings_hash` for
    // the signed message. This lets these tests exercise the real
    // `ed25519_verify` host function instead of mocking it.

    const NOW: u64 = 100_000;

    fn set_ledger_time(env: &Env, ts: u64) {
        env.ledger().with_mut(|li| li.timestamp = ts);
    }

    const DEVICE_PUBKEY: [u8; 32] = [
        61, 96, 56, 225, 198, 42, 112, 73, 27, 145, 32, 19, 61, 101, 171, 209, 227, 47, 2, 25, 84,
        11, 14, 70, 21, 191, 246, 244, 147, 51, 208, 68,
    ];

    fn pubkey(env: &Env) -> BytesN<32> {
        BytesN::from_array(env, &DEVICE_PUBKEY)
    }

    // device=1001 seq=1..=2, readings [(700,100000),(300,100001)]
    const HASH_V1: [u8; 32] = [
        25, 208, 85, 197, 211, 83, 164, 29, 84, 118, 159, 198, 176, 48, 95, 185, 188, 62, 80, 15,
        21, 42, 216, 151, 232, 134, 97, 137, 223, 180, 220, 213,
    ];
    const SIG_V1: [u8; 64] = [
        237, 131, 62, 220, 195, 8, 7, 213, 41, 18, 43, 30, 84, 75, 246, 185, 206, 4, 201, 105, 42,
        82, 215, 17, 213, 173, 219, 122, 165, 143, 197, 185, 193, 26, 16, 109, 222, 124, 100, 189,
        68, 83, 47, 197, 97, 245, 135, 62, 102, 39, 148, 210, 229, 65, 114, 73, 163, 159, 32, 193,
        191, 252, 63, 13,
    ];

    // device=1001 seq=3..=3, readings [(650,100002)]
    const HASH_V2: [u8; 32] = [
        150, 49, 220, 155, 232, 90, 189, 212, 249, 5, 187, 243, 81, 157, 70, 252, 55, 144, 38, 40,
        124, 53, 73, 140, 138, 173, 242, 147, 84, 125, 147, 28,
    ];
    const SIG_V2: [u8; 64] = [
        0, 199, 202, 28, 37, 7, 248, 162, 52, 252, 70, 14, 89, 100, 100, 120, 16, 136, 40, 123,
        217, 127, 61, 111, 184, 164, 130, 238, 18, 140, 198, 223, 189, 145, 106, 102, 202, 116,
        138, 193, 25, 253, 172, 109, 124, 45, 43, 47, 206, 188, 250, 215, 94, 235, 132, 45, 88,
        164, 52, 178, 197, 37, 42, 10,
    ];

    // device=1002 seq=1..=2, readings [(400,100000),(410,100001)]
    const HASH_V3: [u8; 32] = [
        160, 93, 209, 41, 234, 231, 116, 127, 114, 59, 78, 118, 194, 63, 217, 154, 67, 70, 101,
        104, 36, 161, 127, 71, 16, 181, 155, 238, 247, 159, 218, 9,
    ];
    const SIG_V3: [u8; 64] = [
        239, 199, 149, 150, 195, 159, 47, 9, 10, 203, 3, 58, 30, 43, 147, 39, 73, 96, 54, 50, 110,
        169, 121, 158, 39, 169, 8, 233, 102, 69, 141, 250, 53, 139, 128, 161, 127, 14, 161, 141,
        158, 70, 247, 27, 142, 76, 68, 51, 24, 81, 204, 51, 107, 235, 210, 229, 200, 9, 187, 221,
        20, 175, 196, 11,
    ];

    // device=1003 seq=1..=1, readings [(400,100000)] — hash only, signature deliberately forged
    const HASH_V4: [u8; 32] = [
        65, 201, 235, 129, 7, 73, 232, 174, 25, 183, 66, 213, 125, 63, 40, 242, 173, 179, 215,
        198, 47, 4, 11, 155, 84, 242, 40, 2, 68, 113, 7, 123,
    ];
    const SIG_V4: [u8; 64] = [
        203, 122, 10, 82, 82, 115, 165, 169, 155, 206, 166, 165, 114, 71, 121, 81, 248, 23, 108,
        129, 143, 13, 50, 105, 164, 44, 61, 241, 133, 164, 227, 55, 53, 206, 89, 66, 7, 61, 144,
        180, 165, 36, 88, 103, 91, 92, 160, 196, 94, 147, 36, 78, 142, 70, 38, 145, 26, 186, 199,
        245, 78, 139, 209, 13,
    ];

    // device=1005 seq=1..=1, readings [(400,999999999)] — far-future timestamp
    const HASH_V5: [u8; 32] = [
        157, 123, 109, 147, 135, 55, 123, 134, 162, 77, 158, 30, 80, 119, 247, 165, 28, 249, 223,
        221, 47, 152, 112, 172, 204, 120, 183, 240, 148, 33, 132, 87,
    ];
    const SIG_V5: [u8; 64] = [
        186, 64, 31, 184, 113, 228, 195, 139, 32, 52, 172, 99, 104, 200, 197, 197, 81, 55, 42,
        182, 115, 153, 9, 233, 147, 236, 1, 133, 82, 24, 114, 6, 244, 240, 182, 127, 247, 249, 123,
        180, 194, 241, 185, 13, 145, 191, 111, 46, 86, 102, 123, 216, 169, 184, 116, 35, 14, 255,
        126, 21, 62, 202, 189, 2,
    ];

    // device=1006 seq=1..=1, readings [(999999,100000)] — implausible value
    const HASH_V6: [u8; 32] = [
        2, 164, 186, 103, 113, 89, 151, 98, 43, 63, 204, 2, 249, 239, 148, 153, 142, 126, 253,
        120, 236, 19, 137, 94, 253, 172, 248, 243, 37, 15, 89, 220,
    ];
    const SIG_V6: [u8; 64] = [
        250, 39, 158, 208, 30, 138, 252, 79, 192, 118, 182, 236, 149, 185, 47, 143, 27, 219, 170,
        162, 201, 107, 160, 50, 181, 251, 232, 200, 37, 99, 6, 93, 186, 130, 246, 142, 77, 87, 4,
        56, 156, 152, 24, 40, 154, 71, 45, 183, 123, 37, 31, 107, 106, 115, 44, 195, 244, 235, 235,
        179, 121, 110, 193, 10,
    ];

    // device=1006 seq=2..=2, readings [(999999,100001)]
    const HASH_V7: [u8; 32] = [
        184, 72, 89, 245, 88, 43, 199, 253, 218, 124, 251, 126, 161, 196, 56, 134, 181, 55, 162,
        85, 129, 198, 76, 23, 112, 208, 195, 162, 184, 217, 30, 149,
    ];
    const SIG_V7: [u8; 64] = [
        20, 31, 15, 143, 11, 133, 113, 164, 207, 198, 214, 99, 143, 209, 176, 204, 67, 141, 47,
        83, 245, 185, 78, 211, 93, 29, 211, 200, 164, 47, 141, 226, 219, 106, 227, 133, 20, 199,
        173, 116, 249, 244, 66, 127, 72, 147, 156, 62, 244, 67, 68, 119, 71, 190, 58, 243, 144,
        213, 2, 178, 129, 107, 225, 0,
    ];

    // device=1006 seq=3..=3, readings [(999999,100002)]
    const HASH_V8: [u8; 32] = [
        104, 234, 239, 180, 222, 9, 45, 145, 39, 98, 29, 24, 247, 245, 187, 196, 117, 55, 5, 241,
        79, 226, 28, 36, 101, 250, 67, 27, 135, 216, 175, 195,
    ];
    const SIG_V8: [u8; 64] = [
        210, 103, 168, 84, 110, 9, 145, 41, 157, 51, 137, 36, 54, 53, 194, 182, 72, 120, 94, 23,
        117, 9, 241, 206, 165, 6, 255, 74, 29, 219, 94, 71, 137, 194, 200, 87, 78, 220, 236, 11,
        51, 54, 84, 250, 242, 2, 25, 200, 149, 1, 171, 114, 57, 216, 133, 90, 228, 139, 19, 30,
        179, 76, 57, 14,
    ];

    #[test]
    fn test_unregistered_device_rejected() {
        let (env, _admin, client) = create_test_contract();
        set_ledger_time(&env, NOW);
        let submitter = Address::generate(&env);

        let readings = Vec::from_array(
            &env,
            [RawReading { temperature_celsius_x100: 400, timestamp: NOW }],
        );
        let hash = BytesN::from_array(&env, &[0u8; 32]);
        let sig = BytesN::from_array(&env, &[0u8; 64]);

        let result = client.try_submit_reading_batch(
            &submitter, &9999u64, &1u64, &1u64, &readings, &hash, &sig,
        );
        assert_eq!(result, Err(Ok(ContractError::DeviceNotRegistered)));
    }

    #[test]
    fn test_submitter_mismatch_rejected() {
        let (env, admin, client) = create_test_contract();
        set_ledger_time(&env, NOW);
        let gateway = Address::generate(&env);
        let attacker = Address::generate(&env);
        client.register_device(&admin, &1101u64, &gateway, &pubkey(&env), &1u64);

        let readings = Vec::from_array(
            &env,
            [RawReading { temperature_celsius_x100: 400, timestamp: NOW }],
        );
        let hash = BytesN::from_array(&env, &[0u8; 32]);
        let sig = BytesN::from_array(&env, &[0u8; 64]);

        let result = client.try_submit_reading_batch(
            &attacker, &1101u64, &1u64, &1u64, &readings, &hash, &sig,
        );
        assert_eq!(result, Err(Ok(ContractError::SubmitterMismatch)));
    }

    /// Acceptance criterion: a shipment's breach verdict is judged under the
    /// threshold version pinned at `start_shipment`, and a later
    /// `set_product_threshold` call cannot retroactively change it.
    #[test]
    fn test_signed_batch_verdict_pinned_against_retroactive_threshold_change() {
        let (env, admin, client) = create_test_contract();
        set_ledger_time(&env, NOW);

        let gateway = Address::generate(&env);
        let unit_id = 1u64;
        let product_id = 1u64;

        client.register_device(&admin, &1001u64, &gateway, &pubkey(&env), &unit_id);
        let v1 = client.set_product_threshold(&admin, &product_id, &200i32, &600i32);
        assert_eq!(v1, 1);
        let pinned = client.start_shipment(&admin, &unit_id, &product_id);
        assert_eq!(pinned, 1);

        let readings1 = Vec::from_array(
            &env,
            [
                RawReading { temperature_celsius_x100: 700, timestamp: 100_000 },
                RawReading { temperature_celsius_x100: 300, timestamp: 100_001 },
            ],
        );
        let hash1 = BytesN::from_array(&env, &HASH_V1);
        let sig1 = BytesN::from_array(&env, &SIG_V1);
        let outcome1 = client.submit_reading_batch(
            &gateway, &1001u64, &1u64, &2u64, &readings1, &hash1, &sig1,
        );
        assert_eq!(outcome1, BatchOutcome::Accepted);

        let verdict = client.get_shipment_verdict(&unit_id);
        assert_eq!(verdict.violation_count, 1);
        assert_eq!(verdict.peak_celsius_x100, 700);
        assert_eq!(verdict.detected_at, 100_000);

        // Admin widens the product's threshold *after* the shipment started.
        let v2 = client.set_product_threshold(&admin, &product_id, &0i32, &1000i32);
        assert_eq!(v2, 2);

        // The already-recorded verdict must be untouched by the retroactive change.
        let verdict_after = client.get_shipment_verdict(&unit_id);
        assert_eq!(verdict_after, verdict);

        // A further batch on this shipment is still judged under the pinned v1
        // threshold, not the newly-published (wider) v2.
        let readings2 = Vec::from_array(
            &env,
            [RawReading { temperature_celsius_x100: 650, timestamp: 100_002 }],
        );
        let hash2 = BytesN::from_array(&env, &HASH_V2);
        let sig2 = BytesN::from_array(&env, &SIG_V2);
        let outcome2 = client.submit_reading_batch(
            &gateway, &1001u64, &3u64, &3u64, &readings2, &hash2, &sig2,
        );
        assert_eq!(outcome2, BatchOutcome::Accepted);

        let final_verdict = client.get_shipment_verdict(&unit_id);
        assert_eq!(
            final_verdict.violation_count, 2,
            "650 violates the pinned v1 threshold (200-600) even though v2 (0-1000) would allow it"
        );
        assert_eq!(final_verdict.peak_celsius_x100, 700);

        let evidence = client.get_shipment_evidence(&unit_id);
        assert_eq!(evidence.threshold_version, 1, "evidence anchor must report the pinned version");
        assert_eq!(evidence.batch_count, 2);
        assert_ne!(evidence.chain_hash, BytesN::from_array(&env, &[0u8; 32]));
    }

    /// Acceptance criterion: reused sequence numbers (a replayed batch) are rejected.
    #[test]
    fn test_reused_sequence_number_rejected() {
        let (env, admin, client) = create_test_contract();
        set_ledger_time(&env, NOW);
        let gateway = Address::generate(&env);
        let unit_id = 2u64;
        let product_id = 2u64;

        client.register_device(&admin, &1002u64, &gateway, &pubkey(&env), &unit_id);
        client.set_product_threshold(&admin, &product_id, &200i32, &600i32);
        client.start_shipment(&admin, &unit_id, &product_id);

        let readings = Vec::from_array(
            &env,
            [
                RawReading { temperature_celsius_x100: 400, timestamp: 100_000 },
                RawReading { temperature_celsius_x100: 410, timestamp: 100_001 },
            ],
        );
        let hash = BytesN::from_array(&env, &HASH_V3);
        let sig = BytesN::from_array(&env, &SIG_V3);

        let first = client.submit_reading_batch(
            &gateway, &1002u64, &1u64, &2u64, &readings, &hash, &sig,
        );
        assert_eq!(first, BatchOutcome::Accepted);

        // Attacker replays the exact same (validly signed) batch.
        let replay = client.try_submit_reading_batch(
            &gateway, &1002u64, &1u64, &2u64, &readings, &hash, &sig,
        );
        assert_eq!(replay, Err(Ok(ContractError::InvalidSequence)));
    }

    /// Attack test: a batch "signed" by a device other than the one it claims
    /// to be from must be rejected. `ed25519_verify` traps on failure, so
    /// this surfaces as a panic rather than a typed error.
    #[test]
    #[should_panic]
    fn test_forged_device_signature_rejected() {
        let (env, admin, client) = create_test_contract();
        set_ledger_time(&env, NOW);
        let gateway = Address::generate(&env);
        let unit_id = 3u64;
        let product_id = 3u64;

        client.register_device(&admin, &1003u64, &gateway, &pubkey(&env), &unit_id);
        client.set_product_threshold(&admin, &product_id, &200i32, &600i32);
        client.start_shipment(&admin, &unit_id, &product_id);

        let readings = Vec::from_array(
            &env,
            [RawReading { temperature_celsius_x100: 400, timestamp: 100_000 }],
        );
        // Hash correctly commits to the payload; signature does not.
        let hash = BytesN::from_array(&env, &HASH_V4);
        let forged_sig = BytesN::from_array(&env, &[0u8; 64]);

        client.submit_reading_batch(
            &gateway, &1003u64, &1u64, &1u64, &readings, &hash, &forged_sig,
        );
    }

    #[test]
    fn test_readings_hash_mismatch_rejected() {
        let (env, admin, client) = create_test_contract();
        set_ledger_time(&env, NOW);
        let gateway = Address::generate(&env);
        let unit_id = 4u64;
        let product_id = 4u64;

        client.register_device(&admin, &1003u64, &gateway, &pubkey(&env), &unit_id);
        client.set_product_threshold(&admin, &product_id, &200i32, &600i32);
        client.start_shipment(&admin, &unit_id, &product_id);

        let readings = Vec::from_array(
            &env,
            [RawReading { temperature_celsius_x100: 400, timestamp: 100_000 }],
        );
        let wrong_hash = BytesN::from_array(&env, &[9u8; 32]);
        let sig = BytesN::from_array(&env, &SIG_V4); // signed over the *correct* hash, not this one

        let result = client.try_submit_reading_batch(
            &gateway, &1003u64, &1u64, &1u64, &readings, &wrong_hash, &sig,
        );
        assert_eq!(result, Err(Ok(ContractError::ReadingsHashMismatch)));
    }

    #[test]
    fn test_timestamp_out_of_window_rejected() {
        let (env, admin, client) = create_test_contract();
        set_ledger_time(&env, NOW);
        let gateway = Address::generate(&env);
        let unit_id = 5u64;
        let product_id = 5u64;

        client.register_device(&admin, &1005u64, &gateway, &pubkey(&env), &unit_id);
        client.set_product_threshold(&admin, &product_id, &200i32, &600i32);
        client.start_shipment(&admin, &unit_id, &product_id);

        let readings = Vec::from_array(
            &env,
            [RawReading { temperature_celsius_x100: 400, timestamp: 999_999_999 }],
        );
        let hash = BytesN::from_array(&env, &HASH_V5);
        let sig = BytesN::from_array(&env, &SIG_V5);

        let outcome = client.submit_reading_batch(
            &gateway, &1005u64, &1u64, &1u64, &readings, &hash, &sig,
        );
        assert_eq!(outcome, BatchOutcome::RejectedTimestamp);

        // A rejected-timestamp batch must never reach the shipment verdict.
        let result = client.try_get_shipment_verdict(&unit_id);
        assert_eq!(result, Err(Ok(ContractError::ShipmentVerdictNotFound)));
    }

    /// Attack test: an attempt to mask a real breach with an out-of-range
    /// sensor value is excluded from the verdict, and repeated attempts
    /// quarantine the device.
    #[test]
    fn test_implausible_readings_excluded_and_quarantine_device_after_threshold() {
        let (env, admin, client) = create_test_contract();
        set_ledger_time(&env, NOW);
        let gateway = Address::generate(&env);
        let unit_id = 6u64;
        let product_id = 6u64;

        client.register_device(&admin, &1006u64, &gateway, &pubkey(&env), &unit_id);
        client.set_product_threshold(&admin, &product_id, &200i32, &600i32);
        client.start_shipment(&admin, &unit_id, &product_id);

        let submit_implausible = |seq: u64, ts: u64, hash: &[u8; 32], sig: &[u8; 64]| {
            let readings = Vec::from_array(
                &env,
                [RawReading { temperature_celsius_x100: 999_999, timestamp: ts }],
            );
            let hash = BytesN::from_array(&env, hash);
            let sig = BytesN::from_array(&env, sig);
            client.submit_reading_batch(&gateway, &1006u64, &seq, &seq, &readings, &hash, &sig)
        };

        let outcome1 = submit_implausible(1, 100_000, &HASH_V6, &SIG_V6);
        assert_eq!(outcome1, BatchOutcome::RejectedImplausible);
        let device1 = client.get_device(&1006u64);
        assert_eq!(device1.implausible_count, 1);
        assert!(!device1.quarantined);

        let outcome2 = submit_implausible(2, 100_001, &HASH_V7, &SIG_V7);
        assert_eq!(outcome2, BatchOutcome::RejectedImplausible);
        let device2 = client.get_device(&1006u64);
        assert_eq!(device2.implausible_count, 2);
        assert!(!device2.quarantined);

        let outcome3 = submit_implausible(3, 100_002, &HASH_V8, &SIG_V8);
        assert_eq!(outcome3, BatchOutcome::RejectedImplausible);
        let device3 = client.get_device(&1006u64);
        assert_eq!(device3.implausible_count, 3);
        assert!(device3.quarantined, "device must be quarantined after 3 implausible batches");

        // The masked-breach attempt never reached the verdict.
        let result = client.try_get_shipment_verdict(&unit_id);
        assert_eq!(result, Err(Ok(ContractError::ShipmentVerdictNotFound)));

        // Once quarantined, further batches are rejected outright.
        let readings = Vec::from_array(
            &env,
            [RawReading { temperature_celsius_x100: 400, timestamp: 100_003 }],
        );
        let hash = BytesN::from_array(&env, &[0u8; 32]);
        let sig = BytesN::from_array(&env, &[0u8; 64]);
        let result = client.try_submit_reading_batch(
            &gateway, &1006u64, &4u64, &4u64, &readings, &hash, &sig,
        );
        assert_eq!(result, Err(Ok(ContractError::DeviceQuarantined)));
    }

    #[test]
    fn test_device_already_registered_rejected() {
        let (env, admin, client) = create_test_contract();
        let gateway = Address::generate(&env);
        client.register_device(&admin, &2000u64, &gateway, &pubkey(&env), &1u64);

        let result =
            client.try_register_device(&admin, &2000u64, &gateway, &pubkey(&env), &1u64);
        assert_eq!(result, Err(Ok(ContractError::DeviceAlreadyRegistered)));
    }

    #[test]
    fn test_submit_reading_batch_requires_shipment_started() {
        let (env, admin, client) = create_test_contract();
        set_ledger_time(&env, NOW);
        let gateway = Address::generate(&env);
        client.register_device(&admin, &2001u64, &gateway, &pubkey(&env), &7u64);
        // No set_product_threshold / start_shipment call for unit 7.

        let readings = Vec::from_array(
            &env,
            [RawReading { temperature_celsius_x100: 400, timestamp: 100_000 }],
        );
        let hash = BytesN::from_array(&env, &[0u8; 32]);
        let sig = BytesN::from_array(&env, &[0u8; 64]);

        let result = client.try_submit_reading_batch(
            &gateway, &2001u64, &1u64, &1u64, &readings, &hash, &sig,
        );
        assert_eq!(result, Err(Ok(ContractError::ShipmentNotStarted)));
    }
}

// ── Upgradeability & versioned storage schema (#31) ───────────────────────────
//
// Invariant: after an upgrade, the new binary must be able to read every
// prior storage schema version until `migrate` has completed. Absence of the
// stored schema version means schema 1.

/// Code version compiled into this binary. Bump on every release.
pub const CONTRACT_VERSION: u32 = 1;

/// Storage schema version this binary writes. Bump only together with a
/// version-gated transformation in `migrate`.
pub const TARGET_SCHEMA_VERSION: u32 = 1;

const SCHEMA_VERSION_KEY: soroban_sdk::Symbol = soroban_sdk::symbol_short!("SCHEMA_V");
