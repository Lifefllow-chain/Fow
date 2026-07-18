# Contract Events (matching / temperature / reputation / analytics)

Tracks issue #53: bringing the `matching`, `temperature`, `reputation`, and
`analytics` contracts to full event coverage, so every state-mutating
entrypoint is visible to the backend indexer
(`backend/src/contract-event-indexer/`) and downstream reconciliation /
transparency-dashboard consumers.

`payments`, `coordinator`, `inventory`, and `requests` already have their own
event surfaces (inventory's is `contracts/inventory/src/events.rs`) and are
out of scope here — this catalog only covers the four contracts above.

## Envelope

Every event published by these four contracts uses the **same** first-topic
envelope:

```
(domain: Symbol, event: Symbol, schema_version: u32)
```

- `domain` — a short, contract-scoped symbol (`anlytcs`, `match`, `rep`, `temp`).
- `event` — the specific event name within that domain.
- `schema_version` — a `u32`, currently `1` for every event below. Bump the
  version for a given contract's events (in that contract's `events.rs`)
  only when a payload's field set changes; it is unrelated to
  `TARGET_SCHEMA_VERSION` (the contract's on-chain *storage* schema version
  used by `migrate`).

The event data (second `publish` argument) is a single typed
`#[contracttype]` struct per event, defined in that contract's `types.rs`
and constructed in that contract's `events.rs`.

Each contract's `events.rs` module follows the same shape as
`inventory/src/events.rs`: one `emit_*` free function per event, taking the
`Env` plus whatever fields are needed to construct the payload.

## Coverage rule

Every state-mutating entrypoint in these four contracts emits exactly one
event on success (multiple, for entrypoints that fan out into a shared
recalculation — see reputation below) and none on a rejected call. Read-only
query functions, and `pause`-gated internal helpers, emit nothing.

---

## `analytics` (domain: `anlytcs`)

| Entrypoint | Event | Payload struct |
|---|---|---|
| `initialize` | `init` | `InitializedEvent { admin, initialized_at }` |
| `set_reporting_period` | `per_chg` | `ReportingPeriodChangedEvent { old_period_type, new_period_type, new_duration_secs, changed_at }` |
| `record_donation` | `metric` | `MetricRecordedEvent { period_index, metric: MetricKind::Donation, lifetime_total, recorded_at }` |
| `record_request` | `metric` | `MetricRecordedEvent { period_index, metric: MetricKind::Request, lifetime_total, recorded_at }` |
| `record_delivery` | `metric` | `MetricRecordedEvent { period_index, metric: MetricKind::Delivery, lifetime_total, recorded_at }` |
| `record_payment_released` | `pay_rel` | `PaymentReleasedRecordedEvent { period_index, amount, lifetime_count, lifetime_volume, recorded_at }` |
| `upgrade` | `upgraded` | `UpgradedEvent { new_wasm_hash }` |
| `migrate` | `migrated` | `MigratedEvent { new_schema_version }` |

## `matching` (domain: `match`)

| Entrypoint | Event | Payload struct |
|---|---|---|
| `initialize` | `init` | `InitializedEvent { admin, inventory_contract, requests_contract, initialized_at }` |
| `pause` | `pause` | `PauseChangedEvent { admin, paused: true, changed_at }` |
| `unpause` | `pause` | `PauseChangedEvent { admin, paused: false, changed_at }` |
| `match_request` (and transitively `match_multiple_requests`, which calls it once per request) | `matched` | `MatchComputedEvent { request_id, matched_unit_ids, total_matched_ml, remaining_ml, partial_fulfillment, matched_at }` |
| `upgrade` | `upgraded` | `UpgradedEvent { new_wasm_hash }` |
| `migrate` | `migrated` | `MigratedEvent { new_schema_version }` |

`match_request` doesn't mutate the matching contract's own storage (it's a
stateless cross-contract query), but it's this contract's core business
transition — the reason the issue calls out `matching` as having zero
events — so it's covered like any other entrypoint.

## `reputation` (domain: `rep`)

| Entrypoint | Event | Payload struct |
|---|---|---|
| `initialize` | `init` | `InitializedEvent { admin, initialized_at }` |
| `pause` | `pause` | `PauseChangedEvent { admin, paused: true, changed_at }` |
| `unpause` | `pause` | `PauseChangedEvent { admin, paused: false, changed_at }` |
| `submit_rating` | `rating` then `updated` | `RatingSubmittedEvent { entity_id, score, timestamp }`, then `ScoreUpdatedEvent { entity_id, score }` from the internal `calculate_reputation` recalculation |
| `record_assignment` | `assign` then `updated` | `AssignmentRecordedEvent { entity_id, completed, response_secs, timestamp }`, then `ScoreUpdatedEvent` |
| `flag_fraud` | `fraud` then `updated` | `FraudFlaggedEvent { entity_id, timestamp }`, then `ScoreUpdatedEvent` |
| `apply_penalty` | `pen_new` then `updated` | `PenaltyAppliedEvent { entity_id, penalty_id, violation_type, timestamp }`, then `ScoreUpdatedEvent` |
| `appeal_penalty` | `pen_apl` | `PenaltyAppealedEvent { entity_id, penalty_id }` (no recalculation — an appeal doesn't change the score by itself) |
| `resolve_penalty` | `pen_res` then `updated` | `PenaltyResolvedEvent { entity_id, penalty_id, removed }`, then `ScoreUpdatedEvent` |
| `calculate_reputation` (also callable directly) | `updated` | `ScoreUpdatedEvent { entity_id, score }` |
| `upgrade` | `upgraded` | `UpgradedEvent { new_wasm_hash }` |
| `migrate` | `migrated` | `MigratedEvent { new_schema_version }` |

Several entrypoints emit **two** events because they mutate raw input data
and then trigger the shared `calculate_reputation` recalculation, which has
always published its own event (previously with a non-conformant
`(rep, updated, v1-as-Symbol)` envelope; now upgraded to the standard one).
Both events are always emitted together on any success path — an indexer
replaying events can always pair a `rating`/`assign`/`fraud`/`pen_new`/`pen_res`
event with the `updated` event immediately following it for the same
`entity_id`.

## `temperature` (domain: `temp`)

| Entrypoint | Event | Payload struct |
|---|---|---|
| `initialize` | `init` | `InitializedEvent { admin, initialized_at }` |
| `pause` | `pause` | `PauseChangedEvent { admin, paused: true, changed_at }` |
| `unpause` | `pause` | `PauseChangedEvent { admin, paused: false, changed_at }` |
| `set_threshold` | `thresh` | `ThresholdSetEvent { unit_id, min_celsius_x100, max_celsius_x100, set_at }` |
| `log_reading` | `reading` | `ReadingLoggedEvent { unit_id, temperature_celsius_x100, timestamp, is_violation, consecutive_streak, compromised }` |
| `reset_compromised_status` | `cmp_rst` | `CompromisedStatusResetEvent { unit_id, reset_at }` |
| `set_coordinator` | `coord` | `CoordinatorSetEvent { coordinator, set_at }` |
| `add_oracle` | `oracle` | `OracleAddedEvent { oracle, added_at }` |
| `report_excursion_to_coordinator` | `excursion` | `ExcursionReportedEvent { unit_id, payment_id, violation_count, peak_celsius_x100, detected_at, reported_by }` (previously an untyped, envelope-less `(tmp_excur,)` topic + raw tuple; now upgraded to the standard envelope) |
| `upgrade` | `upgraded` | `UpgradedEvent { new_wasm_hash }` |
| `migrate` | `migrated` | `MigratedEvent { new_schema_version }` |

`ReadingLoggedEvent.compromised` reports the unit's *current* (sticky)
compromised flag, not just whether this specific reading crossed the
3-consecutive-violation threshold — once compromised, a unit stays
compromised (per the existing streak-reset semantics) until an admin calls
`reset_compromised_status`.

---

## Testing note: `upgrade` / `migrate`

Across all four contracts, `upgrade`'s WASM swap requires an
already-installed contract hash (`env.deployer().upload_contract_wasm`),
and `migrate` is a no-op on a fresh contract because
`schema_version() == TARGET_SCHEMA_VERSION == 1` already. Exercising their
*success* path in a unit test requires a real compiled WASM binary and a
bumped `TARGET_SCHEMA_VERSION` respectively — the same constraint that
already applies to every contract in this workspace (only `payments` has a
`#[cfg(feature = "upgrade-rehearsal")]` rehearsal test gated behind a
separate build step; see `scripts/test-upgrade-rehearsal.sh`). The event
coverage tests for `upgrade`/`migrate` in each of the four contracts
therefore cover their guard/rejection branches (proving no event fires on
a rejected call); the emission code on the success path is structurally in
place and reachable once a real upgrade/migration is exercised.
