use std::sync::Mutex;

use crypnotes_crypto::CryptoParams as InnerCryptoParams;
use crypnotes_lifecycle::TrashPolicy as InnerTrashPolicy;
use crypnotes_reminders::{
    NextTriggerResult as InnerNextTriggerResult, ReminderSpec as InnerReminderSpec,
};
use thiserror::Error;
use zeroize::Zeroize;

uniffi::setup_scaffolding!();

#[derive(Debug, Clone, Copy, uniffi::Enum)]
pub enum LockDelay {
    Immediate,
    Sec30,
    Min1,
    Min2,
    Min5,
}

#[derive(Debug, Clone, Copy, uniffi::Enum)]
pub enum Recurrence {
    None,
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

#[derive(Debug, Clone, Copy, uniffi::Enum)]
pub enum NoteState {
    Active,
    Archived,
    Trashed,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct CryptoParams {
    pub argon_mem_mib: u32,
    pub argon_iters: u32,
    pub argon_lanes: u32,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct ReminderSpec {
    pub wall_clock_epoch_ms: i64,
    pub tzid: String,
    pub recurrence: Recurrence,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct NextTriggerResult {
    pub next_epoch_ms: Option<i64>,
    pub suppressed: bool,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct TrashPolicy {
    pub trashed_at_epoch_ms: i64,
    pub hard_delete_at_epoch_ms: i64,
}

#[derive(Debug, Error, uniffi::Error)]
pub enum CrypNotesError {
    #[error("invalid input")]
    InvalidInput,
    #[error("crypto failure")]
    CryptoFailure,
    #[error("document failure")]
    DocumentFailure,
    #[error("reminder failure")]
    ReminderFailure,
    #[error("internal error")]
    Internal,
}

#[derive(uniffi::Object)]
pub struct VaultCrypto;

impl Default for VaultCrypto {
    fn default() -> Self {
        Self::new()
    }
}

#[uniffi::export]
impl VaultCrypto {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self
    }

    pub fn derive_kek(
        &self,
        password: String,
        salt: Vec<u8>,
        params: CryptoParams,
    ) -> Result<Vec<u8>, CrypNotesError> {
        let inner = InnerCryptoParams {
            argon_mem_mib: params.argon_mem_mib,
            argon_iters: params.argon_iters,
            argon_lanes: params.argon_lanes,
        };

        crypnotes_crypto::derive_kek(&password, &salt, inner)
            .map_err(|_| CrypNotesError::CryptoFailure)
    }

    pub fn wrap_dek_with_kek(&self, dek: Vec<u8>, kek: Vec<u8>) -> Result<Vec<u8>, CrypNotesError> {
        crypnotes_crypto::wrap_dek_with_kek(&dek, &kek).map_err(|_| CrypNotesError::CryptoFailure)
    }

    pub fn unwrap_dek_with_kek(
        &self,
        wrapped_dek: Vec<u8>,
        kek: Vec<u8>,
    ) -> Result<Vec<u8>, CrypNotesError> {
        crypnotes_crypto::unwrap_dek_with_kek(&wrapped_dek, &kek)
            .map_err(|_| CrypNotesError::CryptoFailure)
    }

    pub fn encrypt_payload(
        &self,
        dek: Vec<u8>,
        plaintext: Vec<u8>,
        aad: Vec<u8>,
    ) -> Result<Vec<u8>, CrypNotesError> {
        crypnotes_crypto::encrypt_payload(&dek, &plaintext, &aad)
            .map_err(|_| CrypNotesError::CryptoFailure)
    }

    pub fn decrypt_payload(
        &self,
        dek: Vec<u8>,
        ciphertext: Vec<u8>,
        aad: Vec<u8>,
    ) -> Result<Vec<u8>, CrypNotesError> {
        crypnotes_crypto::decrypt_payload(&dek, &ciphertext, &aad)
            .map_err(|_| CrypNotesError::CryptoFailure)
    }
}

struct SessionState {
    unlocked: bool,
    decrypted_cache: Vec<u8>,
}

#[derive(uniffi::Object)]
pub struct VaultSession {
    state: Mutex<SessionState>,
}

impl Default for VaultSession {
    fn default() -> Self {
        Self::new()
    }
}

#[uniffi::export]
impl VaultSession {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {
            state: Mutex::new(SessionState {
                unlocked: true,
                decrypted_cache: vec![0xAA, 0xBB, 0xCC],
            }),
        }
    }

    pub fn lock_and_zeroize(&self) -> Result<(), CrypNotesError> {
        let mut state = self.state.lock().map_err(|_| CrypNotesError::Internal)?;
        state.decrypted_cache.zeroize();
        state.decrypted_cache.clear();
        state.unlocked = false;
        Ok(())
    }

    pub fn is_unlocked(&self) -> Result<bool, CrypNotesError> {
        let state = self.state.lock().map_err(|_| CrypNotesError::Internal)?;
        Ok(state.unlocked)
    }
}

#[derive(uniffi::Object)]
pub struct NoteRules;

impl Default for NoteRules {
    fn default() -> Self {
        Self::new()
    }
}

#[uniffi::export]
impl NoteRules {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self
    }

    pub fn validate_note_document(&self, canonical_json: String) -> Result<(), CrypNotesError> {
        crypnotes_doc::validate_note_document(&canonical_json)
            .map_err(|_| CrypNotesError::DocumentFailure)
    }

    pub fn validate_note_char_limit(&self, plain_text: String) -> Result<u32, CrypNotesError> {
        crypnotes_doc::validate_note_char_limit(&plain_text)
            .map_err(|_| CrypNotesError::DocumentFailure)
    }

    pub fn derive_display_title(
        &self,
        explicit_title: String,
        canonical_json: String,
    ) -> Result<String, CrypNotesError> {
        crypnotes_doc::derive_display_title(&explicit_title, &canonical_json)
            .map_err(|_| CrypNotesError::DocumentFailure)
    }

    pub fn convert_text_to_checklist(
        &self,
        canonical_json: String,
    ) -> Result<String, CrypNotesError> {
        crypnotes_doc::convert_text_to_checklist(&canonical_json)
            .map_err(|_| CrypNotesError::DocumentFailure)
    }

    pub fn convert_checklist_to_text(
        &self,
        canonical_json: String,
    ) -> Result<String, CrypNotesError> {
        crypnotes_doc::convert_checklist_to_text(&canonical_json)
            .map_err(|_| CrypNotesError::DocumentFailure)
    }
}

#[derive(uniffi::Object)]
pub struct ReminderEngine;

impl Default for ReminderEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[uniffi::export]
impl ReminderEngine {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self
    }

    pub fn next_trigger(
        &self,
        spec: ReminderSpec,
        now_epoch_ms: i64,
        state: NoteState,
    ) -> Result<NextTriggerResult, CrypNotesError> {
        let inner_spec = InnerReminderSpec {
            wall_clock_epoch_ms: spec.wall_clock_epoch_ms,
            tzid: spec.tzid,
            recurrence: map_recurrence(spec.recurrence),
        };

        let out =
            crypnotes_reminders::next_trigger(&inner_spec, now_epoch_ms, map_reminder_state(state))
                .map_err(|_| CrypNotesError::ReminderFailure)?;

        Ok(map_next_trigger_result(out))
    }
}

#[derive(uniffi::Object)]
pub struct LifecycleRules;

impl Default for LifecycleRules {
    fn default() -> Self {
        Self::new()
    }
}

#[uniffi::export]
impl LifecycleRules {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self
    }

    pub fn compute_trash_policy(&self, trashed_at_epoch_ms: i64) -> TrashPolicy {
        let out: InnerTrashPolicy = crypnotes_lifecycle::compute_trash_policy(trashed_at_epoch_ms);
        TrashPolicy {
            trashed_at_epoch_ms: out.trashed_at_epoch_ms,
            hard_delete_at_epoch_ms: out.hard_delete_at_epoch_ms,
        }
    }

    pub fn is_reminder_suppressed(&self, state: NoteState) -> bool {
        crypnotes_lifecycle::is_reminder_suppressed(map_lifecycle_state(state))
    }
}

#[derive(uniffi::Object)]
pub struct Ids;

impl Default for Ids {
    fn default() -> Self {
        Self::new()
    }
}

#[uniffi::export]
impl Ids {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self
    }

    pub fn new_uuid_v7(&self) -> String {
        crypnotes_ids::new_uuid_v7().to_string()
    }
}

fn map_recurrence(input: Recurrence) -> crypnotes_reminders::Recurrence {
    match input {
        Recurrence::None => crypnotes_reminders::Recurrence::None,
        Recurrence::Daily => crypnotes_reminders::Recurrence::Daily,
        Recurrence::Weekly => crypnotes_reminders::Recurrence::Weekly,
        Recurrence::Monthly => crypnotes_reminders::Recurrence::Monthly,
        Recurrence::Yearly => crypnotes_reminders::Recurrence::Yearly,
    }
}

fn map_reminder_state(input: NoteState) -> crypnotes_reminders::NoteState {
    match input {
        NoteState::Active => crypnotes_reminders::NoteState::Active,
        NoteState::Archived => crypnotes_reminders::NoteState::Archived,
        NoteState::Trashed => crypnotes_reminders::NoteState::Trashed,
    }
}

fn map_lifecycle_state(input: NoteState) -> crypnotes_lifecycle::NoteState {
    match input {
        NoteState::Active => crypnotes_lifecycle::NoteState::Active,
        NoteState::Archived => crypnotes_lifecycle::NoteState::Archived,
        NoteState::Trashed => crypnotes_lifecycle::NoteState::Trashed,
    }
}

fn map_next_trigger_result(input: InnerNextTriggerResult) -> NextTriggerResult {
    NextTriggerResult {
        next_epoch_ms: input.next_epoch_ms,
        suppressed: input.suppressed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vault_session_lock_changes_state() {
        let session = VaultSession::new();
        assert!(session.is_unlocked().unwrap());
        session.lock_and_zeroize().unwrap();
        assert!(!session.is_unlocked().unwrap());
    }

    #[test]
    fn ids_generates_uuid_v7_string() {
        let ids = Ids::new();
        let id = ids.new_uuid_v7();
        let parsed = uuid::Uuid::parse_str(&id).unwrap();
        assert_eq!(parsed.get_version_num(), 7);
    }
}
