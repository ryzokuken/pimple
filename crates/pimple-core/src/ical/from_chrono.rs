//! chrono → jiff conversions.

use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, Timelike, Utc};
use chrono_tz::Tz;
use jiff::civil::{Date as JiffDate, DateTime as JiffDateTime};
use jiff::tz::TimeZone;
use jiff::{Span, Timestamp, Zoned};

use crate::error::{CoreError, Result};

/// Convert `chrono::NaiveDate` to `jiff::civil::Date`.
///
/// # Errors
/// Returns `CoreError::Conversion` if the year is outside jiff's representable range.
#[expect(
    clippy::cast_possible_truncation,
    reason = "chrono month is 1..=12 and day is 1..=31, both fit in i8"
)]
pub fn chrono_naive_date_to_jiff(d: NaiveDate) -> Result<JiffDate> {
    let year = i16::try_from(d.year())
        .map_err(|e| CoreError::Conversion(format!("year out of range: {e}")))?;
    JiffDate::new(year, d.month() as i8, d.day() as i8)
        .map_err(|e| CoreError::Conversion(format!("naive date: {e}")))
}

/// Convert `chrono::NaiveDateTime` to `jiff::civil::DateTime`.
///
/// # Errors
/// Returns `CoreError::Conversion` if the year is outside jiff's representable range.
#[expect(
    clippy::cast_possible_truncation,
    reason = "chrono month/day/hour/minute/second are all within i8 ranges"
)]
pub fn chrono_naive_datetime_to_jiff(dt: NaiveDateTime) -> Result<JiffDateTime> {
    let year = i16::try_from(dt.year())
        .map_err(|e| CoreError::Conversion(format!("year out of range: {e}")))?;
    let nanos = i32::try_from(dt.and_utc().timestamp_subsec_nanos())
        .map_err(|e| CoreError::Conversion(format!("nanos out of range: {e}")))?;
    JiffDateTime::new(
        year,
        dt.month() as i8,
        dt.day() as i8,
        dt.hour() as i8,
        dt.minute() as i8,
        dt.second() as i8,
        nanos,
    )
    .map_err(|e| CoreError::Conversion(format!("naive datetime: {e}")))
}

/// Convert `chrono::DateTime<Utc>` to `jiff::Timestamp`.
///
/// # Errors
/// Returns `CoreError::Conversion` if the timestamp is outside jiff's representable range.
pub fn chrono_utc_to_jiff(dt: DateTime<Utc>) -> Result<Timestamp> {
    let ts = Timestamp::from_second(dt.timestamp())
        .map_err(|e| CoreError::Conversion(format!("timestamp out of range: {e}")))?;
    Ok(ts + Span::new().nanoseconds(i64::from(dt.timestamp_subsec_nanos())))
}

/// Convert `chrono::DateTime<chrono_tz::Tz>` to `jiff::Zoned`, preserving the IANA zone.
///
/// # Errors
/// Returns `CoreError::Conversion` if the zone is unknown to jiff or the timestamp is out of range.
pub fn chrono_zoned_to_jiff(dt: DateTime<Tz>) -> Result<Zoned> {
    let tz_name = dt.timezone().name();
    let tz = TimeZone::get(tz_name)
        .map_err(|e| CoreError::Conversion(format!("unknown tz {tz_name}: {e}")))?;
    let ts = Timestamp::from_second(dt.timestamp())
        .map_err(|e| CoreError::Conversion(format!("timestamp: {e}")))?
        + Span::new().nanoseconds(i64::from(dt.timestamp_subsec_nanos()));
    Ok(ts.to_zoned(tz))
}
