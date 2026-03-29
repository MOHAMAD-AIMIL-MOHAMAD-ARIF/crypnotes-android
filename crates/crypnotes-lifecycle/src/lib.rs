use serde::{Deserialize, Serialize};

pub const TRASH_RETENTION_MS: i64 = 30 * 24 * 60 * 60 * 1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NoteState {
    Active,
    Archived,
    Trashed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrashPolicy {
    pub trashed_at_epoch_ms: i64,
    pub hard_delete_at_epoch_ms: i64,
}

pub fn compute_trash_policy(trashed_at_epoch_ms: i64) -> TrashPolicy {
    TrashPolicy {
        trashed_at_epoch_ms,
        hard_delete_at_epoch_ms: trashed_at_epoch_ms + TRASH_RETENTION_MS,
    }
}

pub fn is_reminder_suppressed(state: NoteState) -> bool {
    matches!(state, NoteState::Archived | NoteState::Trashed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trash_expiry_is_exact_30x24_hours() {
        let trashed_at = 1_700_000_000_000_i64;
        let policy = compute_trash_policy(trashed_at);

        assert_eq!(policy.trashed_at_epoch_ms, trashed_at);
        assert_eq!(
            policy.hard_delete_at_epoch_ms - trashed_at,
            TRASH_RETENTION_MS
        );
    }

    #[test]
    fn reminder_state_suppression_matches_requirements() {
        assert!(!is_reminder_suppressed(NoteState::Active));
        assert!(is_reminder_suppressed(NoteState::Archived));
        assert!(is_reminder_suppressed(NoteState::Trashed));
    }
}
