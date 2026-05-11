//! iCalendar parsing.
//!
//! Internally uses the `icalendar` crate, which exposes chrono-based types.
//! Conversion to jiff happens before the function returns; nothing outside
//! this module sees chrono.

use chrono::TimeZone as _;
use icalendar::{Calendar, CalendarComponent, CalendarDateTime, Component, DatePerhapsTime};
use sha2::{Digest, Sha256};

use crate::error::{CoreError, Result};
use crate::event::{Event, OverrideInstance, RRuleSpec};
use crate::id::CollectionId;
use crate::time::EventTime;

use super::from_chrono::{
    chrono_naive_date_to_jiff, chrono_naive_datetime_to_jiff, chrono_utc_to_jiff,
    chrono_zoned_to_jiff,
};

/// Parse a `.ics` document into a single `Event`.
///
/// Multi-VEVENT files (master + RECURRENCE-ID overrides) collapse into one
/// `Event` whose `overrides` list captures the modified instances.
///
/// # Errors
/// Returns `CoreError::IcalParse` if the text is not valid iCalendar, if the
/// calendar contains no VEVENT, or if required properties (UID, DTSTART, DTEND,
/// DTSTAMP) are missing or malformed.
pub fn parse_ics(text: &str, collection_id: CollectionId) -> Result<Event> {
    let calendar: Calendar = text.parse().map_err(|e: String| CoreError::IcalParse(e))?;

    let mut master: Option<&icalendar::Event> = None;
    let mut overrides_raw: Vec<&icalendar::Event> = Vec::new();

    for component in &calendar.components {
        if let CalendarComponent::Event(ev) = component {
            if ev.get_recurrence_id().is_some() {
                overrides_raw.push(ev);
            } else if master.is_some() {
                return Err(CoreError::IcalParse(
                    "multiple master VEVENTs (no RECURRENCE-ID) in a single file".into(),
                ));
            } else {
                master = Some(ev);
            }
        }
    }

    let master = master.ok_or_else(|| CoreError::IcalParse("no VEVENT in calendar".into()))?;

    let uid = master
        .property_value("UID")
        .ok_or_else(|| CoreError::IcalParse("VEVENT missing UID".into()))?
        .to_owned();

    let summary = master.property_value("SUMMARY").unwrap_or("").to_owned();
    let description = master.property_value("DESCRIPTION").map(unescape_text);
    let location = master.property_value("LOCATION").map(unescape_text);

    let start = parse_datetime(
        master
            .get_start()
            .ok_or_else(|| CoreError::IcalParse(format!("VEVENT {uid} missing DTSTART")))?,
    )?;
    let end = parse_datetime(
        master
            .get_end()
            .ok_or_else(|| CoreError::IcalParse(format!("VEVENT {uid} missing DTEND")))?,
    )?;

    let rrule = master.property_value("RRULE").map(|line| RRuleSpec {
        line: line.to_owned(),
    });

    let exdates = master
        .multi_properties()
        .get("EXDATE")
        .map(|props| {
            props
                .iter()
                .filter_map(DatePerhapsTime::from_property)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
        .into_iter()
        .map(parse_datetime)
        .collect::<Result<Vec<_>>>()?;

    // The icalendar crate parses DTSTAMP/CREATED/LAST-MODIFIED into
    // chrono::DateTime<Utc> for us via the Component trait methods.
    let created_at = master
        .get_created()
        .or_else(|| master.get_timestamp())
        .ok_or_else(|| CoreError::IcalParse(format!("VEVENT {uid} missing DTSTAMP")))
        .and_then(chrono_utc_to_jiff)?;

    let modified_at = master
        .get_last_modified()
        .map(chrono_utc_to_jiff)
        .transpose()?
        .unwrap_or(created_at);

    let overrides = overrides_raw
        .into_iter()
        .map(parse_override)
        .collect::<Result<Vec<_>>>()?;

    let raw_hash = hex::encode(Sha256::digest(text.as_bytes()));

    Ok(Event {
        uid,
        collection_id,
        summary,
        description,
        location,
        start,
        end,
        rrule,
        exdates,
        overrides,
        created_at,
        modified_at,
        raw_hash,
        raw_ics: text.to_owned(),
    })
}

fn parse_override(ev: &icalendar::Event) -> Result<OverrideInstance> {
    let recurrence_id = ev
        .get_recurrence_id()
        .ok_or_else(|| CoreError::IcalParse("override missing RECURRENCE-ID".into()))
        .and_then(parse_datetime)?;
    let start = parse_datetime(
        ev.get_start()
            .ok_or_else(|| CoreError::IcalParse("override VEVENT missing DTSTART".into()))?,
    )?;
    let end = parse_datetime(
        ev.get_end()
            .ok_or_else(|| CoreError::IcalParse("override VEVENT missing DTEND".into()))?,
    )?;
    Ok(OverrideInstance {
        recurrence_id,
        start,
        end,
        summary: ev.property_value("SUMMARY").map(str::to_owned),
        description: ev.property_value("DESCRIPTION").map(unescape_text),
        location: ev.property_value("LOCATION").map(unescape_text),
    })
}

fn parse_datetime(dpt: DatePerhapsTime) -> Result<EventTime> {
    match dpt {
        DatePerhapsTime::Date(d) => Ok(EventTime::AllDay {
            date: chrono_naive_date_to_jiff(d)?,
        }),
        DatePerhapsTime::DateTime(dt) => match dt {
            CalendarDateTime::Floating(ndt) => Ok(EventTime::Floating {
                datetime: chrono_naive_datetime_to_jiff(ndt)?,
            }),
            CalendarDateTime::Utc(u) => Ok(EventTime::Utc {
                instant: chrono_utc_to_jiff(u)?,
            }),
            CalendarDateTime::WithTimezone { date_time, tzid } => {
                let tz: chrono_tz::Tz = tzid
                    .parse()
                    .map_err(|e| CoreError::IcalParse(format!("unknown TZID {tzid}: {e}")))?;
                let zoned = tz.from_local_datetime(&date_time).single().ok_or_else(|| {
                    CoreError::IcalParse(format!(
                        "ambiguous local datetime {date_time} in {tzid} (DST gap or fold)"
                    ))
                })?;
                Ok(EventTime::Zoned {
                    zoned: chrono_zoned_to_jiff(zoned)?,
                })
            }
        },
    }
}

/// Unescape RFC 5545 TEXT escaping.
///
/// `\n` / `\N` → newline; `\\` → backslash; `\,` → comma; `\;` → semicolon.
fn unescape_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n' | 'N') => out.push('\n'),
                Some('\\') | None => out.push('\\'),
                Some(',') => out.push(','),
                Some(';') => out.push(';'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}
