CrypNotes Android v1

Solo Developer Implementation Checklist



# Summary

* Build v1 as an offline-first encrypted notes app using the existing Rust crate baseline plus currently scaffolded Android modules.
* Deliver in 7 sequential phases so one developer can ship vertical slices without backtracking.
* Keep v2 items out of scope: sync and TXT export/share.



# Public APIs/Interfaces to Lock Early

* Rust/UniFFI surface will expose stable objects for VaultCrypto, VaultSession, NoteRules, ReminderEngine, LifecycleRules, and Ids, expanded only to cover missing v1 use cases (vault bootstrap/recovery, attachment envelope helpers, richer error mapping).
* crypnotes.udl enums and records must be treated as contract-first and versioned with additive-only changes after Phase 3.
* Android data contracts will use the Room schema doc as source of truth (notes, labels, note\_labels, attachments, reminders, note\_order, vault\_meta, note\_index + FTS + triggers).



# Phase 1: Foundations and Build Wiring

## Rust Tasks

☐ Standardize crate quality gates: fmt, clippy, unit tests across all workspace crates.

☐ Add shared version/constants module for doc schema, payload schema, and encryption container versions.

☐ Ensure UniFFI generation flow is deterministic from crates/crypnotes-ffi/src/crypnotes.udl.

## Android Tasks

☐ Replace minimal app shell with Compose + Navigation host and module wiring from app.

☐ Add required dependencies and plugins: Compose, Room, SQLCipher integration path, Hilt, Coroutines/Flow, WorkManager, Biometric.

☐ Set baseline CI commands for cargo test and Gradle build/test.

## Exit Criteria

☐ Fresh clone builds Rust and Android without manual fixes.

☐ Kotlin bindings generate successfully and compile in :core:bridge.

☐ App launches into a Compose root screen.



# Phase 2: Vault and Cryptography Core

## Rust Tasks

☐ Complete Argon2id/KEK/DEK flows with strict parameter enforcement and error typing.

☐ Implement encrypted envelope helpers for note/attachment payloads with AAD conventions.

☐ Harden lock/zeroization session behavior and add negative tests (tamper, wrong key, invalid salt).

## Android Tasks

☐ Implement keystore-backed DK lifecycle in core:platform:security (create, unwrap, invalidation handling).

☐ Implement vault\_meta persistence and startup vault state machine (first-run create vs unlock).

☐ Build vault UI flow in feature:vault for password unlock, quick unlock toggle, and fallback policy.

## Exit Criteria

☐ User can create vault, lock, unlock with password, and unlock with quick unlock when enabled.

☐ DK invalidation path recovers via password and re-establishes quick unlock.

☐ No plaintext secrets are persisted or logged.



# Phase 3: Local Data Layer and Repositories

## Rust Tasks

☐ Finalize note validation helpers for char limits, title derivation, and checklist conversions.

☐ Finalize lifecycle helpers for exact trash retention math and reminder suppression rules.

☐ Add serialization/compat tests for schema versioned payloads.

## Android Tasks

☐ Implement full Room schema and migration discipline using the provided v1 schema.

☐ Implement encrypted blob store for attachments (original + thumbnail refs).

☐ Implement repositories in core:data for notes, labels, attachments, reminders, vault meta, note index/FTS.

## Exit Criteria

☐ CRUD works for all core entities with encryption-at-rest for payload/blob content.

☐ Local search returns results from FTS-backed index.

☐ Repository tests cover migration and foreign-key cascade behavior.



# Phase 4: Notes, Labels, Editor, and Organization

## Rust Tasks

☐ Expand document rule coverage for supported block types and conversion invariants.

☐ Add property/unit tests for transformation reversibility and title derivation edge cases.

## Android Tasks

☐ Implement notes list/detail/editor in feature:notes with rich formatting controls.

☐ Implement FR organization features: pin/unpin, archive, trash, sorting modes, custom/manual order.

☐ Implement labels feature with many-to-many tagging and filtering.

☐ Enforce note 20,000 character limit in UI + repository boundary; support undo/redo.

## Exit Criteria

☐ FR-03, FR-04, FR-05, FR-06, FR-07, FR-13, FR-14, FR-15, FR-16, FR-17 pass functional verification.

☐ Manual order persists across app restart.

☐ Title fallback always derives from first non-empty paragraph when explicit title is empty.



# Phase 5: Attachments (Image/Photo/Audio)

## Rust Tasks

☐ Provide stable attachment encryption/decryption helpers for blob payloads.

☐ Add tests for attachment metadata validation boundaries passed from Android.

## Android Tasks

☐ Implement image ingest pipeline: 5 MB cap, max dimension 1920 px, JPEG \~80-85%, 256 px thumbnail.

☐ Implement audio recorder pipeline with hard 3-minute cutoff.

☐ Persist encrypted attachment blobs and metadata; wire rendering/decryption for note detail.

☐ Enforce trash lifecycle: keep attachment blobs while note is trashed, delete on hard-expiry only.

## Exit Criteria

☐ Image and audio attachments can be added, viewed/played, and reopened after app restart.

☐ Size/duration limits are hard-enforced.

☐ Hard delete removes attachment blobs only after trash expiry.



# Phase 6: Reminders, Time Semantics, and Background Work

## Rust Tasks

☐ Harden recurrence engine for None/Daily/Weekly/Monthly/Yearly with timezone/DST semantics.

☐ Add regression tests for missing local times (shift forward) and repeated times (single trigger).

## Android Tasks

☐ Implement reminder CRUD UI in feature:reminders and note integration points.

☐ Implement scheduler bridge (core:platform:notifications + WorkManager/AlarmManager) for local notifications.

☐ Enforce reminder suppression when note is archived/trashed; resume when restored.

☐ Implement periodic trash-expiry worker (exact 30x24h retention behavior).

## Exit Criteria

☐ FR-21, FR-22, FR-23, FR-24 pass on device tests including DST cases.

☐ Reminder notifications fire once per expected occurrence.

☐ Archived/trashed notes never trigger reminders.



# Phase 7: Security/Privacy Hardening and Release Gate

## Rust Tasks

☐ Audit zeroization paths and error conversions to avoid leaking sensitive state.

☐ Freeze UniFFI API for v1 and tag compatibility snapshot tests.

## Android Tasks

☐ Implement settings for auto-lock timer options and secure-screen (FLAG\_SECURE) toggle.

☐ Validate biometric failure fallback behavior exactly per policy.

☐ Add log-scrubbing/telemetry guardrails to prevent plaintext leaks.

☐ Prepare release build profile, baseline proguard rules, and release checklist artifacts.

## Exit Criteria

☐ All in-scope FRs (FR-01 to FR-24 except out-of-scope items) pass acceptance.

☐ Threat model document completed (assets, trust boundaries, attacker assumptions, mitigations).

☐ Release candidate build is reproducible and test-passing.



# Test Cases and Scenarios

☐ Rust unit tests: crypto roundtrip/tamper, Argon2 params, reminder DST semantics, lifecycle retention exactness, doc transforms.

☐ Rust property tests: reminder suppression invariants and recurrence progression monotonicity.

☐ Android unit tests: repository CRUD/migrations, search index updates, vault state machine transitions.

☐ Android instrumentation tests: unlock flows, editor formatting + undo/redo, attachment limits, reminder scheduling/notification behavior.

☐ Manual QA matrix: light/dark theme, lock timers (0/30/60/120/300s), biometric fallback toggle combinations, archive/trash/restore edge paths.



# Assumptions and Defaults

☐ Single-developer execution with strictly sequential phases; no parallel workstreams assumed.

☐ Current module layout and package roots remain unchanged.

☐ Min SDK stays at 28; target/compile SDK stays at 35 for v1.

☐ SQLCipher remains the DB-at-rest approach for v1.

☐ Out of scope remains unchanged: TXT export/share and encrypted sync.

