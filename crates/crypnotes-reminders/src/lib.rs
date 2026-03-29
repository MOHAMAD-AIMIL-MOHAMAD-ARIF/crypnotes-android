use chrono::{DateTime, Days, Duration, LocalResult, Months, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Recurrence {
    None,
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NoteState {
    Active,
    Archived,
    Trashed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReminderSpec {
    pub wall_clock_epoch_ms: i64,
    pub tzid: String,
    pub recurrence: Recurrence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NextTriggerResult {
    pub next_epoch_ms: Option<i64>,
    pub suppressed: bool,
}

#[derive(Debug, Error)]
pub enum ReminderError {
    #[error("invalid timezone id: {0}")]
    InvalidTimezone(String),
    #[error("invalid epoch milliseconds: {0}")]
    InvalidEpoch(i64),
    #[error("unable to resolve local time after DST gap")]
    UnresolvableLocalTime,
}

pub fn next_trigger(
    spec: &ReminderSpec,
    now_epoch_ms: i64,
    state: NoteState,
) -> Result<NextTriggerResult, ReminderError> {
    if matches!(state, NoteState::Archived | NoteState::Trashed) {
        return Ok(NextTriggerResult {
            next_epoch_ms: None,
            suppressed: true,
        });
    }

    let tz: Tz = spec
        .tzid
        .parse()
        .map_err(|_| ReminderError::InvalidTimezone(spec.tzid.clone()))?;

    let anchor_utc = DateTime::<Utc>::from_timestamp_millis(spec.wall_clock_epoch_ms)
        .ok_or(ReminderError::InvalidEpoch(spec.wall_clock_epoch_ms))?;
    let now_utc = DateTime::<Utc>::from_timestamp_millis(now_epoch_ms)
        .ok_or(ReminderError::InvalidEpoch(now_epoch_ms))?;

    let anchor_local = anchor_utc.with_timezone(&tz);
    let anchor_naive = anchor_local.naive_local();
    let now_local = now_utc.with_timezone(&tz);

    if matches!(spec.recurrence, Recurrence::None) {
        return Ok(NextTriggerResult {
            next_epoch_ms: (anchor_utc >= now_utc).then_some(anchor_utc.timestamp_millis()),
            suppressed: false,
        });
    }

    let start_index = initial_index(
        spec.recurrence,
        anchor_local.naive_local(),
        now_local.naive_local(),
    );

    for index in start_index..(start_index + 200_000) {
        let Some(candidate_naive) = build_candidate(anchor_naive, spec.recurrence, index) else {
            break;
        };

        let candidate_local = resolve_local_datetime(tz, candidate_naive)?;
        let candidate_utc = candidate_local.with_timezone(&Utc);

        if candidate_utc >= now_utc {
            return Ok(NextTriggerResult {
                next_epoch_ms: Some(candidate_utc.timestamp_millis()),
                suppressed: false,
            });
        }
    }

    Ok(NextTriggerResult {
        next_epoch_ms: None,
        suppressed: false,
    })
}

fn initial_index(recurrence: Recurrence, anchor: NaiveDateTime, now: NaiveDateTime) -> u32 {
    if now <= anchor {
        return 0;
    }

    let day_diff = (now.date() - anchor.date()).num_days();
    match recurrence {
        Recurrence::Daily => day_diff.max(0) as u32,
        Recurrence::Weekly => (day_diff.max(0) / 7) as u32,
        Recurrence::Monthly | Recurrence::Yearly | Recurrence::None => 0,
    }
}

fn build_candidate(
    anchor: NaiveDateTime,
    recurrence: Recurrence,
    index: u32,
) -> Option<NaiveDateTime> {
    let date = match recurrence {
        Recurrence::None => {
            if index == 0 {
                anchor.date()
            } else {
                return None;
            }
        }
        Recurrence::Daily => anchor.date().checked_add_days(Days::new(index as u64))?,
        Recurrence::Weekly => anchor
            .date()
            .checked_add_days(Days::new((index as u64).checked_mul(7)?))?,
        Recurrence::Monthly => anchor.date().checked_add_months(Months::new(index))?,
        Recurrence::Yearly => {
            let months = index.checked_mul(12)?;
            anchor.date().checked_add_months(Months::new(months))?
        }
    };

    Some(date.and_time(anchor.time()))
}

fn resolve_local_datetime(tz: Tz, naive: NaiveDateTime) -> Result<DateTime<Tz>, ReminderError> {
    match tz.from_local_datetime(&naive) {
        LocalResult::Single(dt) => Ok(dt),
        LocalResult::Ambiguous(first, _) => Ok(first),
        LocalResult::None => {
            for minutes in 1..=180 {
                let shifted = naive + Duration::minutes(minutes);
                match tz.from_local_datetime(&shifted) {
                    LocalResult::Single(dt) => return Ok(dt),
                    LocalResult::Ambiguous(first, _) => return Ok(first),
                    LocalResult::None => continue,
                }
            }
            Err(ReminderError::UnresolvableLocalTime)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;
    use proptest::prelude::*;

    fn spec_from_local(
        tz: Tz,
        y: i32,
        m: u32,
        d: u32,
        hh: u32,
        mm: u32,
        recurrence: Recurrence,
    ) -> ReminderSpec {
        let local = match tz.with_ymd_and_hms(y, m, d, hh, mm, 0) {
            LocalResult::Single(dt) => dt,
            LocalResult::Ambiguous(first, _) => first,
            LocalResult::None => panic!("unexpected nonexistent local time in test setup"),
        };

        ReminderSpec {
            wall_clock_epoch_ms: local.with_timezone(&Utc).timestamp_millis(),
            tzid: tz.name().to_owned(),
            recurrence,
        }
    }

    #[test]
    fn recurrence_none_returns_none_after_past_trigger() {
        let tz = chrono_tz::UTC;
        let spec = spec_from_local(tz, 2024, 1, 1, 10, 0, Recurrence::None);
        let now = spec.wall_clock_epoch_ms + 1;

        let out = next_trigger(&spec, now, NoteState::Active).unwrap();
        assert!(!out.suppressed);
        assert_eq!(out.next_epoch_ms, None);
    }

    #[test]
    fn reminders_are_suppressed_when_note_archived_or_trashed() {
        let tz = chrono_tz::UTC;
        let spec = spec_from_local(tz, 2024, 1, 1, 10, 0, Recurrence::Daily);

        let archived = next_trigger(&spec, spec.wall_clock_epoch_ms, NoteState::Archived).unwrap();
        let trashed = next_trigger(&spec, spec.wall_clock_epoch_ms, NoteState::Trashed).unwrap();

        assert!(archived.suppressed);
        assert!(trashed.suppressed);
        assert_eq!(archived.next_epoch_ms, None);
        assert_eq!(trashed.next_epoch_ms, None);
    }

    #[test]
    fn dst_missing_local_time_shifts_forward() {
        let tz = chrono_tz::America::New_York;
        let spec = spec_from_local(tz, 2024, 3, 9, 2, 30, Recurrence::Daily);

        let now_local = tz.with_ymd_and_hms(2024, 3, 10, 0, 0, 0).single().unwrap();
        let out = next_trigger(
            &spec,
            now_local.with_timezone(&Utc).timestamp_millis(),
            NoteState::Active,
        )
        .unwrap();

        let next_utc = DateTime::<Utc>::from_timestamp_millis(out.next_epoch_ms.unwrap()).unwrap();
        let next_local = next_utc.with_timezone(&tz);

        assert_eq!(next_local.date_naive().to_string(), "2024-03-10");
        assert_eq!(next_local.hour(), 3);
        assert_eq!(next_local.minute(), 0);
    }

    #[test]
    fn dst_repeated_time_triggers_once() {
        let tz = chrono_tz::America::New_York;
        let spec = spec_from_local(tz, 2024, 11, 2, 1, 30, Recurrence::Daily);

        let ambiguous = tz.from_local_datetime(
            &NaiveDateTime::parse_from_str("2024-11-03 01:30:00", "%Y-%m-%d %H:%M:%S").unwrap(),
        );
        let (first, second) = match ambiguous {
            LocalResult::Ambiguous(a, b) => (a, b),
            _ => panic!("expected ambiguous local time"),
        };

        let now_utc = (first + Duration::minutes(30)).with_timezone(&Utc);
        assert!(second.with_timezone(&Utc) > now_utc);

        let out = next_trigger(&spec, now_utc.timestamp_millis(), NoteState::Active).unwrap();
        let next = DateTime::<Utc>::from_timestamp_millis(out.next_epoch_ms.unwrap())
            .unwrap()
            .with_timezone(&tz);

        assert_eq!(next.date_naive().to_string(), "2024-11-04");
        assert_eq!(next.hour(), 1);
        assert_eq!(next.minute(), 30);
    }

    proptest! {
        #[test]
        fn archived_or_trashed_always_suppresses(reminder_ms in 1_600_000_000_000_i64..1_900_000_000_000_i64) {
            let spec = ReminderSpec {
                wall_clock_epoch_ms: reminder_ms,
                tzid: "UTC".to_owned(),
                recurrence: Recurrence::Weekly,
            };

            let archived = next_trigger(&spec, reminder_ms, NoteState::Archived).unwrap();
            let trashed = next_trigger(&spec, reminder_ms, NoteState::Trashed).unwrap();

            prop_assert!(archived.suppressed);
            prop_assert!(trashed.suppressed);
            prop_assert_eq!(archived.next_epoch_ms, None);
            prop_assert_eq!(trashed.next_epoch_ms, None);
        }
    }
}
