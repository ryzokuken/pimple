//! Expand an `Event` into concrete `EventInstance`s within a date range.

use chrono::Utc as ChronoUtc;
use jiff::Timestamp;
use rrule::{RRuleSet, Tz as RTz};

use crate::error::{CoreError, Result};
use crate::event::{Event, EventInstance, OverrideInstance};
use crate::time::EventTime;

use super::from_chrono::{chrono_utc_to_jiff, chrono_zoned_to_jiff};
use super::to_chrono::{jiff_timestamp_to_chrono_utc, jiff_zoned_to_chrono};

/// Yield every concrete `EventInstance` of `event` whose start lies in
/// `[range_start, range_end)`. Honors EXDATE and RECURRENCE-ID overrides.
///
/// # Errors
/// Returns `CoreError::IcalParse` if the RRULE string is malformed, the event
/// uses a floating or all-day DTSTART with RRULE (unsupported in v0.1), or an
/// EXDATE lacks a timezone reference.
pub fn expand_in_range(
    event: &Event,
    range_start: Timestamp,
    range_end: Timestamp,
) -> Result<Vec<EventInstance>> {
    let Some(rrule_spec) = event.rrule.as_ref() else {
        return Ok(non_recurring(event, range_start, range_end));
    };

    let dtstart_chrono = match &event.start {
        EventTime::Zoned { zoned } => jiff_zoned_to_chrono(zoned)?,
        EventTime::Utc { instant } => {
            jiff_timestamp_to_chrono_utc(*instant).with_timezone(&chrono_tz::UTC)
        }
        EventTime::AllDay { .. } | EventTime::Floating { .. } => {
            return Err(CoreError::IcalParse(
                "RRULE on a floating or all-day event is not yet supported in v0.1; \
                 wrap the event in a TZID before recurrence"
                    .into(),
            ));
        }
    };
    let rrule_string = format!(
        "DTSTART;TZID={tz}:{stamp}\nRRULE:{line}",
        tz = dtstart_chrono.timezone().name(),
        stamp = dtstart_chrono.format("%Y%m%dT%H%M%S"),
        line = rrule_spec.line,
    );
    let rset: RRuleSet = rrule_string
        .parse()
        .map_err(|e| CoreError::IcalParse(format!("rrule parse: {e}")))?;

    let after = jiff_timestamp_to_chrono_utc(range_start).with_timezone(&RTz::UTC);
    let before = jiff_timestamp_to_chrono_utc(range_end).with_timezone(&RTz::UTC);
    let result = rset.after(after).before(before).all(10_000);
    let raw_occurrences = result.dates;

    let exdates_utc = exdates_as_utc(event)?;
    let duration = duration_between(&event.start, &event.end);

    let mut out: Vec<EventInstance> = Vec::with_capacity(raw_occurrences.len());
    for occ in raw_occurrences {
        let occ_utc = occ.with_timezone(&ChronoUtc);
        if exdates_utc.contains(&occ_utc) {
            continue;
        }
        if let Some(override_inst) = find_override(event, &occ_utc) {
            out.push(instance_from_override(event, override_inst));
        } else {
            let start = occurrence_to_event_time(&event.start, &occ)?;
            let end = shift_event_time(&start, duration);
            out.push(EventInstance {
                event_uid: event.uid.clone(),
                collection_id: event.collection_id.clone(),
                summary: event.summary.clone(),
                description: event.description.clone(),
                location: event.location.clone(),
                start,
                end,
                is_override: false,
            });
        }
    }
    Ok(out)
}

fn non_recurring(
    event: &Event,
    range_start: Timestamp,
    range_end: Timestamp,
) -> Vec<EventInstance> {
    let start_ts = event.start.to_timestamp();
    let end_ts = event.end.to_timestamp();
    if end_ts <= range_start || start_ts >= range_end {
        return Vec::new();
    }
    vec![EventInstance {
        event_uid: event.uid.clone(),
        collection_id: event.collection_id.clone(),
        summary: event.summary.clone(),
        description: event.description.clone(),
        location: event.location.clone(),
        start: event.start.clone(),
        end: event.end.clone(),
        is_override: false,
    }]
}

fn instance_from_override(event: &Event, ov: &OverrideInstance) -> EventInstance {
    EventInstance {
        event_uid: event.uid.clone(),
        collection_id: event.collection_id.clone(),
        summary: ov.summary.clone().unwrap_or_else(|| event.summary.clone()),
        description: ov.description.clone().or_else(|| event.description.clone()),
        location: ov.location.clone().or_else(|| event.location.clone()),
        start: ov.start.clone(),
        end: ov.end.clone(),
        is_override: true,
    }
}

fn exdates_as_utc(event: &Event) -> Result<Vec<chrono::DateTime<ChronoUtc>>> {
    event
        .exdates
        .iter()
        .map(|t| match t {
            EventTime::Utc { instant } => Ok(jiff_timestamp_to_chrono_utc(*instant)),
            EventTime::Zoned { zoned } => Ok(jiff_timestamp_to_chrono_utc(zoned.timestamp())),
            EventTime::AllDay { .. } | EventTime::Floating { .. } => Err(CoreError::IcalParse(
                "EXDATE without timezone reference cannot be matched to recurrence instants".into(),
            )),
        })
        .collect()
}

fn duration_between(start: &EventTime, end: &EventTime) -> jiff::Span {
    end.to_timestamp() - start.to_timestamp()
}

fn shift_event_time(t: &EventTime, span: jiff::Span) -> EventTime {
    match t {
        EventTime::AllDay { date } => EventTime::AllDay {
            date: date.checked_add(span).unwrap_or(*date),
        },
        EventTime::Floating { datetime } => EventTime::Floating {
            datetime: datetime.checked_add(span).unwrap_or(*datetime),
        },
        EventTime::Utc { instant } => EventTime::Utc {
            instant: instant.checked_add(span).unwrap_or(*instant),
        },
        EventTime::Zoned { zoned } => EventTime::Zoned {
            zoned: zoned.checked_add(span).unwrap_or_else(|_| zoned.clone()),
        },
    }
}

fn occurrence_to_event_time(
    template: &EventTime,
    occurrence: &chrono::DateTime<RTz>,
) -> Result<EventTime> {
    match template {
        EventTime::Zoned { zoned } => {
            let tz_name = zoned
                .time_zone()
                .iana_name()
                .ok_or_else(|| CoreError::Conversion("event has no IANA tz".into()))?;
            let tz: chrono_tz::Tz = tz_name
                .parse()
                .map_err(|e| CoreError::Conversion(format!("chrono-tz parse {tz_name}: {e}")))?;
            let local = occurrence.with_timezone(&tz);
            Ok(EventTime::Zoned {
                zoned: chrono_zoned_to_jiff(local)?,
            })
        }
        EventTime::Utc { .. } => Ok(EventTime::Utc {
            instant: chrono_utc_to_jiff(occurrence.with_timezone(&ChronoUtc))?,
        }),
        EventTime::AllDay { .. } | EventTime::Floating { .. } => Err(CoreError::IcalParse(
            "internal: unexpected non-zoned RRULE expansion".into(),
        )),
    }
}

fn find_override<'a>(
    event: &'a Event,
    occurrence_utc: &chrono::DateTime<ChronoUtc>,
) -> Option<&'a OverrideInstance> {
    for o in &event.overrides {
        let recur_ts = match &o.recurrence_id {
            EventTime::Utc { instant } => *instant,
            EventTime::Zoned { zoned } => zoned.timestamp(),
            _ => continue,
        };
        let recur_chrono = jiff_timestamp_to_chrono_utc(recur_ts);
        if recur_chrono == *occurrence_utc {
            return Some(o);
        }
    }
    None
}
