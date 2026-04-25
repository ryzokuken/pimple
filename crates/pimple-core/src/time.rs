//! Datetime representation matching iCalendar's four DATE-TIME forms.

use jiff::civil::{Date, DateTime as PlainDateTime};
use jiff::{Timestamp, Zoned};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// All four iCalendar DATE-TIME forms modeled explicitly.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/ipc/types/")]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EventTime {
    /// `DATE` value — wall-clock date with no time component.
    AllDay {
        #[ts(type = "string")]
        date: Date,
    },
    /// `DATE-TIME` with neither `TZID=` parameter nor trailing `Z`.
    Floating {
        #[ts(type = "string")]
        datetime: PlainDateTime,
    },
    /// `DATE-TIME` with trailing `Z` (UTC).
    Utc {
        #[ts(type = "string")]
        instant: Timestamp,
    },
    /// `DATE-TIME` with `TZID=` parameter.
    Zoned {
        #[ts(type = "string")]
        zoned: Zoned,
    },
}

impl EventTime {
    /// Convert to a `jiff::Timestamp` for ordering / range queries.
    ///
    /// Floating times are interpreted in the system zone.
    ///
    /// # Panics
    ///
    /// Panics if the system timezone database is unavailable or the conversion
    /// produces an out-of-range timestamp (should not happen in practice).
    #[must_use]
    #[expect(
        clippy::expect_used,
        reason = "system zone failure is unrecoverable here"
    )]
    pub fn to_timestamp(&self) -> Timestamp {
        match self {
            Self::AllDay { date } => date
                .at(0, 0, 0, 0)
                .to_zoned(jiff::tz::TimeZone::system())
                .expect("system zone")
                .timestamp(),
            Self::Floating { datetime } => datetime
                .to_zoned(jiff::tz::TimeZone::system())
                .expect("system zone")
                .timestamp(),
            Self::Utc { instant } => *instant,
            Self::Zoned { zoned } => zoned.timestamp(),
        }
    }
}
