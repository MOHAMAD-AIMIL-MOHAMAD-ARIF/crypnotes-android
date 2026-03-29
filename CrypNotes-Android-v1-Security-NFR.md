# CrypNotes Android v1 — Security Non-Functional Requirements

|Security NFR|Description|
|-|-|
|Zero-knowledge content protection|The system shall ensure note bodies and attachment contents are encrypted so that plaintext content is not readable outside the unlocked vault context.|
|Symmetric envelope encryption|The system shall encrypt notes and attachments with a DEK using an AEAD construction, with XChaCha20-Poly1305 as the preferred algorithm for v1 content/blobs.|
|Password-based key wrapping|The system shall derive a KEK from master password + random salt using Argon2id and use that KEK to wrap/unwrap the DEK.|
|Fixed Argon2id baseline hardness|The Argon2id configuration for v1 shall be fixed to memory 64 MiB, iterations 3, parallelism 1 (non-adaptive across device tiers) to provide consistent password-hardening behavior.|
|Device-bound quick unlock key|The system shall generate a per-device DK in Android Keystore and use it only to wrap/unwrap DEK for local quick unlock when enabled by policy.|
|Keystore-gated local DEK recovery|The persisted local wrapped DEK (`local\_wrapped\_dek`) shall only be decryptable through Android Keystore policy gates (biometric and/or device credential as configured).|
|Credential fallback and recovery path|If biometric unlock fails or DK becomes invalidated, the system shall provide password-based DEK recovery and allow re-establishing biometric quick unlock with a newly generated DK.|
|Vault lock and memory zeroization|On vault lock (including app background/exit per lock timer policy), the app shall zeroize decrypted in-memory note/cache material to minimize plaintext residency.|
|Log/telemetry secret exclusion|Application logs, crash reports, and analytics shall never contain decrypted note content, attachment-sensitive metadata, passwords, keys/secrets, content-derived local search tokens, or any plaintext vault material.|
|Screenshot privacy control|The app shall provide a user setting to enable/disable secure-screen behavior (`FLAG\_SECURE`) to control capture visibility in recents/screenshots.|
|At-rest local database encryption|Local structured storage shall use SQLCipher encryption for data at rest in the Room-backed database.|
|Encrypted attachment blob persistence|Attachment originals and thumbnails shall be stored as encrypted blobs; when notes are trashed, encrypted blobs persist until trash-expiry hard deletion.|
|Independent versioning for secure evolution|Document schema version, note payload schema version, and encryption container version shall be independently versioned to support secure backward/forward compatibility and migration control.|



