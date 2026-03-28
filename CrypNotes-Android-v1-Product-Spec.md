# CrypNotes Android v1 (MVP) — Finalized Product Specification

## 1\) Product Scope

### 1.1 In-scope (v1 Android MVP)

All requested features **except**:

* **#14 Export/share note as TXT** (out of scope for v1)
* **#16 Encrypted multi-device sync** (out of scope for v1 release milestone)

### 1.2 Delivery versions

* **v1:** Local vault, offline-first, no network sync.
* **v2:** Zero-knowledge sync and backend APIs.

### 1.3 Platform/stack constraints

* Fresh greenfield Android app.
* Native Android UI with Kotlin + Jetpack Compose.
* Shared Rust core to be created from scratch, bound via UniFFI.
* Android first; interfaces should remain iOS/Web-ready.

\---

## 2\) Functional Requirements (v1)

## 2.1 Notes \& organization

1. Light and dark app themes.
2. Per-note background colors.
3. Labels (many-to-many via `Label` + `NoteLabel`).
4. Sorting modes:

   * Date created
   * Date modified
   * Custom/manual order
5. Pin/unpin notes; pinned notes rendered at top.
6. Archive folder.
7. Trash folder with auto-delete after **exactly 30 × 24h** from trash timestamp.

## 2.2 Vault lock/unlock \& session behavior

8. Automatic vault lock when app exits/backgrounds with user-selectable timer:

   * Immediate, 30s, 1m, 2m, 5m
9. Decrypted in-memory note/cache zeroization on lock.
10. Optional quick unlock via biometric/device credential (fingerprint/Face/device PIN/credential) instead of master password when enabled.
11. If biometric fails, fallback to device credential only if setting is enabled.
12. If DK is invalidated (e.g., biometric set changed), recovery flow:
13. User taps “Use password instead”.
14. Derive KEK from password + salt via Argon2id.
15. Obtain `vault\_key\_blob = Encrypt(KEK, DEK || vault\_metadata)` (local cache or server in sync phase).
16. Decrypt blob locally to recover DEK.
17. Re-establish biometric unlock by creating new DK and storing `local\_wrapped\_dek = Encrypt(DK, DEK)`.

## 2.3 Editor \& note content

13. Rich text formatting in v1 UI:
* Bold, italic, underline
* Highlight
* Headings H1/H2/H3
* Checklist
* Bulleted list
* Numbered list
14. Convert text ↔ checklist:
* Conversion applies to all text in the note.
* Revert supported.
15. Undo/Redo during note editing.
16. Title model:
* Explicit optional `title` field.
* If empty, display title derived from **first paragraph from top of note**.
17. Note size limit: **20,000 characters**.

## 2.4 Attachments

18. Image/photo attachments:
* Max file size: **5 MB** hard limit.
* Pre-encryption processing: resize max dimension **1920 px**, JPEG compression \~80–85%.
* Store original encrypted blob + encrypted thumbnail blob.
* Thumbnail target: **256 px**.
19. Audio attachments:
* Max recording duration: **3 minutes** hard-enforced at recorder layer.
20. Attachments remain in storage while note is in Trash; permanent deletion only after trash expiry.

## 2.5 Reminders

21. Reminders are first-class objects (not buried in note metadata).
22. Recurrence options:
* None, Daily, Weekly, Monthly, Yearly
23. Time semantics:
* Timezone-anchored wall-clock recurrence.
* DST handling:

  * Missing local times: shift forward.
  * Repeated local times: trigger once.
24. If note is archived or trashed, associated reminders are disabled.
25. (v2) Reminders sync and trigger per device.

## 2.6 Out-of-scope for v1 release

* TXT export/share feature (#14)
* Encrypted cross-device sync feature (#16)

\---

## 3\) Security \& Cryptography Specification

## 3.1 Core principles

* Zero-knowledge design for note content and attachments.
* No asymmetric content encryption (no sharing use-case in scope).
* Server-readable metadata permitted in sync phase only where non-sensitive.

## 3.2 Key hierarchy

* **DEK**: Random symmetric key encrypting notes + attachments (AEAD).
* **KEK**: Derived from master password + ramdom generated salt via Argon2id; wraps/unwraps DEK.
* **DK**: Per-device random key in Android Keystore; wraps DEK for local quick unlock.

## 3.3 Cryptographic primitives \& parameters

* Preferred AEAD for content/blobs: **XChaCha20-Poly1305**.
* KDF: **Argon2id** with fixed params:

  * Memory: **64 MiB**
  * Iterations: **3**
  * Parallelism: **1**
  * Non-adaptive across device tiers.

## 3.4 Envelope/format versioning

Version all three independently:

1. Document schema version (editor node tree)
2. Note payload schema version (full serialized note object)
3. Encryption container version (cipher wrapper fields)

## 3.5 Biometric and secure storage (Android)

* DK generated and protected by Android Keystore with biometric/device-credential gating.
* Local persisted `local\_wrapped\_dek` can be unlocked without password when policy allows.

## 3.6 Logging and telemetry policy

Never log or emit:

* Decrypted note/body content
* Attachment-sensitive metadata
* Passwords/keys/secrets
* Content-derived local search tokens
* Any plaintext vault material in logs, crash reports, analytics

## 3.7 UI privacy control

* User setting to toggle secure-screen behavior (`FLAG\_SECURE`) on/off.

\---

## 4\) Data \& Domain Model Specification

## 4.1 Identifier policy

Use **UUIDv7** (Rust-generated only) as durable IDs for:

* Note
* Attachment
* Label
* Reminder
* Block

## 4.2 Notes storage model

* Canonical note body: JSON, schema-constrained node tree (ProseMirror-like model).
* Queryable local index separated from encrypted canonical payload.
* Local searchable index includes decrypted title/body tokens (local-only search).

## 4.3 Relational entities (conceptual)

* `Note` (encrypted payload reference + indexed fields)
* `Attachment` (blob references + metadata + note linkage)
* `Label`
* `NoteLabel` (join table)
* `Reminder` (separate table/object linked to Note)
* `NoteOrder` (custom ordering support, if separate)
* `TrashInfo` / lifecycle timestamps
* `VaultMeta` (non-content metadata as needed)

## 4.4 Lifecycle rules

* Delete note → move to Trash state.
* Trash retention expiry (30×24h) → hard delete note, attachments, reminder associations.
* Archive state suppresses reminder firing.

\---

## 5\) Android Technical Architecture

## 5.1 App stack

* Language: Kotlin
* UI: Jetpack Compose
* Local DB: Room + SQLCipher
* DI: Hilt
* Async: Coroutines + Flow
* Notifications: Android notification stack with scheduled reminder delivery
* Secure keys: Android Keystore

## 5.2 Modular architecture

Pragmatic layered, feature-based modules with clean boundaries:

* `app` (composition root, navigation, DI wiring)
* `feature-notes` (list/detail/editor)
* `feature-labels`
* `feature-reminders`
* `feature-settings`
* `feature-vault`
* `core-ui`
* `core-platform` (keystore, notifications, media capture)
* `core-data` (Room, repositories, migrations)
* `core-rust-bridge` (UniFFI bindings, mappers)
* Rust workspace/crate(s): crypto/domain/schemas/ID generation

Avoid excessive boilerplate-heavy architecture patterns.

## 5.3 Ownership boundaries

* Rust core owns:

  * Crypto and key-handling logic interfaces
  * Canonical document/schema validation
  * UUIDv7 generation
  * (Later) CRDT-like merge logic
* Android owns:

  * UI/UX
  * Local persistence schema + migrations
  * Keystore integration
  * Networking client (v2)

\---

## 6\) Reminder \& Time Semantics

* Reminder modeled separately from notes.
* Stored with explicit timezone context.
* Recurrence evaluated in stored timezone.
* DST missing-time rule: schedule at next valid local time.
* DST repeated-time rule: single trigger only.
* Archived/trashed notes disable reminder execution.

\---

## 7\) Local Persistence \& Migrations

* Room schema versioning independent from Rust schema versions.
* SQLCipher encryption enabled for local DB at rest.
* Blob store for encrypted attachments outside note rows.
* Derived encrypted thumbnails as separate blob objects.
* Migration discipline required for each local schema change.

\---

## 8\) Sync Design Baseline (Post-v1, predefined constraints)

Even though sync is not in v1 release, architecture must reserve for:

* Zero-knowledge encrypted content and attachments.
* Server-readable transport metadata only.
* Local-only search (no plaintext searchable sync index).
* Conflict strategy: **CRDT-like document/block-level merge**, not LWW/naive field merge.
* Reminders synchronized cross-device.

\---

## 9\) Non-Functional Requirements

## 9.1 Offline-first

* Full v1 functionality without network dependency.

## 9.2 Performance/limits

* Note char limit and attachment caps enforced at UI + domain validation layers.
* Auto-lock zeroization must complete promptly at lock event.

## 9.3 Quality gate expectations

Required test categories:

1. Rust unit tests for core logic
2. Selective Android unit/instrumentation tests for critical flows
3. Minimal real crypto test vectors

Property testing: recommended for high-risk logic, not mandatory gate.

## 9.4 Security review

Before v1 is considered complete, run lightweight threat model documenting:

* Assets
* Trust boundaries
* Attacker assumptions
* Primary risks
* Chosen mitigations

## 9.5 CI/CD

* Linting
* Formatting
* Static checks
* Automated tests
* Reproducible development builds
* Release signing optional unless distributed beyond personal use

\---

## 10\) Explicit Requirement Matrix (Feature IDs)

* \#1 Theme: **In v1**
* \#2 Background colors: **In v1**
* \#3 Labels: **In v1**
* \#4 Sorting modes: **In v1**
* \#5 Pinning: **In v1**
* \#6 Archive: **In v1**
* \#7 Trash + 30-day delete: **In v1**
* \#8 Auto-lock + timer + zeroize: **In v1**
* \#9 Biometric/device credential unlock option: **In v1**
* \#10 Rich formatting + constrained JSON document model: **In v1**
* \#11 Convert/revert checklist: **In v1**
* \#12 Image/photo/audio encrypted attachments: **In v1**
* \#13 Undo/Redo: **In v1**
* \#14 TXT export/share: **Out of scope for v1**
* \#15 Reminder notifications + recurrence: **In v1**
* \#16 Encrypted device sync: **Out of scope for v1 release (v2)**

