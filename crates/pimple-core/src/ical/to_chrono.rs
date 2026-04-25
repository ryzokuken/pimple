//! jiff → chrono conversions.

use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use chrono_tz::Tz;
use jiff::civil::{Date as JiffDate, DateTime as JiffDateTime};
use jiff::{Timestamp, Zoned};

use crate::error::{CoreError, Result};

/// Convert `jiff::civil::Date` to `chrono::NaiveDate`.
///
/// # Panics
/// Never panics in practice: jiff's civil date range fits entirely within chrono's.
#[expect(
    clippy::expect_used,
    clippy::cast_sign_loss,
    reason = "jiff Date always represents a valid civil date within chrono's range"
)]
#[must_use]
pub fn jiff_date_to_chrono(d: JiffDate) -> NaiveDate {
    NaiveDate::from_ymd_opt(
        i32::from(d.year()),
        u32::from(d.month() as u8),
        u32::from(d.day() as u8),
    )
    .expect("jiff Date is always a valid civil date")
}

/// Convert `jiff::civil::DateTime` to `chrono::NaiveDateTime`.
///
/// # Panics
/// Never panics in practice: jiff's civil datetime range fits entirely within chrono's.
#[expect(
    clippy::expect_used,
    clippy::cast_sign_loss,
    reason = "jiff DateTime always represents a valid civil time within chrono's range"
)]
#[must_use]
pub fn jiff_datetime_to_chrono(dt: JiffDateTime) -> NaiveDateTime {
    let date = jiff_date_to_chrono(dt.date());
    let nanos = u32::try_from(dt.subsec_nanosecond()).unwrap_or(0);
    let time = NaiveTime::from_hms_nano_opt(
        u32::from(dt.hour() as u8),
        u32::from(dt.minute() as u8),
        u32::from(dt.second() as u8),
        nanos,
    )
    .expect("jiff DateTime is always a valid civil time");
    NaiveDateTime::new(date, time)
}

/// Convert `jiff::Timestamp` to `chrono::DateTime<Utc>`.
///
/// Uses Euclidean division on the total nanoseconds to produce a chrono-compatible
/// `(secs, subsec_nanos)` pair where `subsec_nanos` is always in `0..1_000_000_000`,
/// even for pre-epoch timestamps where Jiff's `subsec_nanosecond()` is negative.
///
/// # Panics
/// Never panics in practice: jiff's timestamp range fits entirely within chrono's.
#[expect(
    clippy::expect_used,
    clippy::cast_possible_truncation,
    reason = "jiff Timestamp range fits in i64 secs and u32 nanos after Euclidean division"
)]
#[must_use]
pub fn jiff_timestamp_to_chrono_utc(t: Timestamp) -> DateTime<Utc> {
    let total_ns = t.as_nanosecond();
    let secs = total_ns.div_euclid(1_000_000_000) as i64;
    let nanos = total_ns.rem_euclid(1_000_000_000) as u32;
    DateTime::<Utc>::from_timestamp(secs, nanos)
        .expect("jiff Timestamp is always representable as chrono UTC")
}

/// Convert a `jiff::Zoned` to a `chrono::DateTime<chrono_tz::Tz>`, preserving the zone.
///
/// # Errors
/// Returns `CoreError::Conversion` if the zone is not a named IANA zone or the
/// IANA name doesn't parse in `chrono-tz`.
pub fn jiff_zoned_to_chrono(z: &Zoned) -> Result<DateTime<Tz>> {
    let tz_name = z.time_zone().iana_name().ok_or_else(|| {
        CoreError::Conversion("zoned datetime is not in a named IANA zone".into())
    })?;
    let tz: Tz = tz_name
        .parse()
        .map_err(|e| CoreError::Conversion(format!("chrono-tz parse {tz_name}: {e}")))?;
    let utc = jiff_timestamp_to_chrono_utc(z.timestamp());
    Ok(utc.with_timezone(&tz))
}
