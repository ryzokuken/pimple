#![allow(clippy::expect_used, clippy::unwrap_used)]

use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;

use pimple_core::ical::from_chrono::{
    chrono_naive_date_to_jiff, chrono_naive_datetime_to_jiff, chrono_utc_to_jiff,
    chrono_zoned_to_jiff,
};
use pimple_core::ical::to_chrono::{
    jiff_date_to_chrono, jiff_datetime_to_chrono, jiff_timestamp_to_chrono_utc,
    jiff_zoned_to_chrono,
};

#[test]
fn date_round_trip() {
    let nd = NaiveDate::from_ymd_opt(2026, 4, 25).unwrap();
    let j = chrono_naive_date_to_jiff(nd).unwrap();
    let back = jiff_date_to_chrono(j);
    assert_eq!(nd, back);
}

#[test]
fn naive_datetime_round_trip() {
    let ndt = NaiveDate::from_ymd_opt(2026, 4, 25)
        .unwrap()
        .and_hms_opt(15, 30, 0)
        .unwrap();
    let j = chrono_naive_datetime_to_jiff(ndt).unwrap();
    let back = jiff_datetime_to_chrono(j);
    assert_eq!(ndt, back);
}

#[test]
fn utc_round_trip() {
    let utc = Utc.with_ymd_and_hms(2026, 4, 25, 15, 30, 0).unwrap();
    let j = chrono_utc_to_jiff(utc).unwrap();
    let back = jiff_timestamp_to_chrono_utc(j);
    assert_eq!(utc, back);
}

#[test]
fn zoned_round_trip_preserves_tz() {
    let tz: Tz = "Europe/Berlin".parse().unwrap();
    let dt = tz
        .with_ymd_and_hms(2026, 4, 25, 15, 30, 0)
        .single()
        .unwrap();
    let j = chrono_zoned_to_jiff(dt).unwrap();
    let back = jiff_zoned_to_chrono(&j).unwrap();
    assert_eq!(dt, back);
}

use proptest::prelude::*;

proptest! {
    #[test]
    fn date_round_trip_prop(
        year in 1970i32..2100,
        month in 1u32..=12,
        day in 1u32..=28,
    ) {
        let nd = NaiveDate::from_ymd_opt(year, month, day).unwrap();
        let j = chrono_naive_date_to_jiff(nd).unwrap();
        let back = jiff_date_to_chrono(j);
        prop_assert_eq!(nd, back);
    }

    #[test]
    fn utc_round_trip_prop(
        secs in -2_000_000_000i64..2_000_000_000,
        nanos in 0u32..1_000_000_000,
    ) {
        let dt = DateTime::<Utc>::from_timestamp(secs, nanos).unwrap();
        let j = chrono_utc_to_jiff(dt).unwrap();
        let back = jiff_timestamp_to_chrono_utc(j);
        prop_assert_eq!(dt, back);
    }
}
