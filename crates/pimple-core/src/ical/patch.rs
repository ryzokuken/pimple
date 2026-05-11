//! Targeted mutations on raw `.ics` text.
//!
//! [`patch_ics`] applies a sequence of [`IcsMutation`] operations and returns
//! the resulting `.ics` text. Lines that no mutation touches round-trip
//! verbatim — including unknown properties (`X-APPLE-CALENDAR-COLOR`, etc.),
//! `VALARM` sub-components, and `VTIMEZONE` blocks. This is the load-bearing
//! primitive behind edit and delete (v0.2 Task 5/6).
//!
//! The model used internally is intentionally shallow: each VEVENT is a list
//! of property lines plus opaque sub-component blocks. Anything we don't need
//! to mutate stays as a string, preserving exact bytes through round-trip.

use std::collections::HashMap;

use jiff::Timestamp;
use jiff::civil::{Date as JiffDate, DateTime as JiffDateTime};
use jiff::tz::TimeZone;

use crate::error::{CoreError, Result};
use crate::event::{OverrideInstance, RRuleSpec};
use crate::time::EventTime;

/// A single, targeted mutation on a VCALENDAR text.
///
/// `patch_ics` accepts a slice of mutations and applies them in order. Each
/// mutation targets either the master VEVENT (the one with no `RECURRENCE-ID`)
/// or the list of override VEVENTs.
#[derive(Debug, Clone)]
pub enum IcsMutation {
    /// Replace or insert the master VEVENT's `SUMMARY` line.
    SetMasterSummary(String),
    /// Set or remove the master VEVENT's `DESCRIPTION`. `None` removes the line.
    SetMasterDescription(Option<String>),
    /// Set or remove the master VEVENT's `LOCATION`. `None` removes the line.
    SetMasterLocation(Option<String>),
    /// Replace the master VEVENT's `DTSTART`.
    SetMasterStart(EventTime),
    /// Replace the master VEVENT's `DTEND`.
    SetMasterEnd(EventTime),
    /// Set or remove the master VEVENT's `RRULE`. `None` removes the line.
    SetMasterRRule(Option<RRuleSpec>),
    /// Append an `EXDATE` line to the master VEVENT.
    AddExdate(EventTime),
    /// Append an override VEVENT block (with `RECURRENCE-ID`) after the master.
    /// The override inherits the master's `UID`.
    AddOverrideVEvent(OverrideInstance),
    /// Modify the master `RRULE` so its `UNTIL` is at or before the given
    /// instant. No-op if the master has no `RRULE`. Per RFC 5545 §3.3.10
    /// `UNTIL` is emitted in UTC form for non-floating cutoffs.
    TruncateRRuleUntil(EventTime),
    /// Remove every override VEVENT whose `RECURRENCE-ID` resolves to at-or-
    /// after the given instant (compared as absolute timestamps).
    RemoveOverridesAtOrAfter(EventTime),
}

/// Apply `mutations` (in order) to `raw` and return the resulting `.ics` text.
///
/// # Errors
///
/// Returns [`CoreError::IcalParse`] if `raw` cannot be parsed as VCALENDAR,
/// if the master VEVENT cannot be identified, or if a mutation references a
/// timezone unknown to jiff.
pub fn patch_ics(raw: &str, mutations: &[IcsMutation]) -> Result<String> {
    let mut model = parse(raw)?;
    for m in mutations {
        apply(&mut model, m)?;
    }
    Ok(emit(&model))
}

// ───── Internal model ────────────────────────────────────────────────────────

struct Model {
    /// Lines after `BEGIN:VCALENDAR` and before the master VEVENT
    /// (`VERSION`, `PRODID`, full `VTIMEZONE` blocks, etc.). Preserved
    /// verbatim modulo folding.
    pre_master: Vec<String>,
    master: VEvent,
    overrides: Vec<VEvent>,
    /// Lines after the last VEVENT and before `END:VCALENDAR`. Usually empty.
    post_events: Vec<String>,
}

struct VEvent {
    items: Vec<VEventItem>,
}

enum VEventItem {
    /// Unfolded property line: `KEY[;PARAMS]:VALUE`.
    Property(String),
    /// Sub-component (`VALARM`, etc.). Body lines are preserved unparsed.
    SubComponent {
        begin: String,
        body: Vec<String>,
        end: String,
    },
}

// ───── Parsing ───────────────────────────────────────────────────────────────

fn parse(raw: &str) -> Result<Model> {
    let lines = unfold(raw);
    let mut iter = lines.into_iter();

    let mut pre_master: Vec<String> = Vec::new();
    let mut master: Option<VEvent> = None;
    let mut overrides: Vec<VEvent> = Vec::new();
    let mut post_events: Vec<String> = Vec::new();
    let mut saw_calendar_open = false;
    let mut saw_calendar_close = false;

    while let Some(line) = iter.next() {
        if !saw_calendar_open {
            if line.starts_with("BEGIN:VCALENDAR") {
                saw_calendar_open = true;
            }
            continue;
        }
        if line.starts_with("END:VCALENDAR") {
            saw_calendar_close = true;
            break;
        }
        if line.starts_with("BEGIN:VEVENT") {
            let block = parse_vevent(&mut iter)?;
            if has_property(&block, "RECURRENCE-ID") {
                overrides.push(block);
            } else if master.is_some() {
                return Err(CoreError::IcalParse(
                    "patch_ics: multiple master VEVENTs (no RECURRENCE-ID) in one file".into(),
                ));
            } else {
                master = Some(block);
            }
        } else if let Some(other) = line.strip_prefix("BEGIN:") {
            let name = other.to_owned();
            let mut block_lines = vec![line];
            loop {
                let next = iter.next().ok_or_else(|| {
                    CoreError::IcalParse(format!("patch_ics: unterminated {name}"))
                })?;
                let is_end = next
                    .strip_prefix("END:")
                    .is_some_and(|rest| rest.trim() == name);
                block_lines.push(next);
                if is_end {
                    break;
                }
            }
            if master.is_some() {
                post_events.extend(block_lines);
            } else {
                pre_master.extend(block_lines);
            }
        } else if master.is_some() {
            post_events.push(line);
        } else {
            pre_master.push(line);
        }
    }

    if !saw_calendar_open || !saw_calendar_close {
        return Err(CoreError::IcalParse(
            "patch_ics: missing BEGIN:VCALENDAR / END:VCALENDAR".into(),
        ));
    }
    let master =
        master.ok_or_else(|| CoreError::IcalParse("patch_ics: no master VEVENT".into()))?;

    Ok(Model {
        pre_master,
        master,
        overrides,
        post_events,
    })
}

fn parse_vevent<I: Iterator<Item = String>>(iter: &mut I) -> Result<VEvent> {
    let mut items: Vec<VEventItem> = Vec::new();
    loop {
        let line = iter
            .next()
            .ok_or_else(|| CoreError::IcalParse("patch_ics: unterminated VEVENT".into()))?;
        if line.starts_with("END:VEVENT") {
            return Ok(VEvent { items });
        }
        if let Some(other) = line.strip_prefix("BEGIN:") {
            let name = other.to_owned();
            let begin = line;
            let mut body = Vec::new();
            let end = loop {
                let next = iter.next().ok_or_else(|| {
                    CoreError::IcalParse(format!("patch_ics: unterminated {name} inside VEVENT"))
                })?;
                if next
                    .strip_prefix("END:")
                    .is_some_and(|rest| rest.trim() == name)
                {
                    break next;
                }
                body.push(next);
            };
            items.push(VEventItem::SubComponent { begin, body, end });
        } else {
            items.push(VEventItem::Property(line));
        }
    }
}

// ───── Mutations ─────────────────────────────────────────────────────────────

fn apply(model: &mut Model, m: &IcsMutation) -> Result<()> {
    match m {
        IcsMutation::SetMasterSummary(s) => {
            set_or_insert(
                &mut model.master,
                "SUMMARY",
                &format!("SUMMARY:{}", escape_text(s)),
            );
        }
        IcsMutation::SetMasterDescription(opt) => match opt {
            Some(s) => set_or_insert(
                &mut model.master,
                "DESCRIPTION",
                &format!("DESCRIPTION:{}", escape_text(s)),
            ),
            None => remove_property(&mut model.master, "DESCRIPTION"),
        },
        IcsMutation::SetMasterLocation(opt) => match opt {
            Some(s) => set_or_insert(
                &mut model.master,
                "LOCATION",
                &format!("LOCATION:{}", escape_text(s)),
            ),
            None => remove_property(&mut model.master, "LOCATION"),
        },
        IcsMutation::SetMasterStart(t) => {
            let line = format_dt_property("DTSTART", t)?;
            set_or_insert(&mut model.master, "DTSTART", &line);
        }
        IcsMutation::SetMasterEnd(t) => {
            let line = format_dt_property("DTEND", t)?;
            set_or_insert(&mut model.master, "DTEND", &line);
        }
        IcsMutation::SetMasterRRule(opt) => match opt {
            Some(r) => set_or_insert(&mut model.master, "RRULE", &format!("RRULE:{}", r.line)),
            None => remove_property(&mut model.master, "RRULE"),
        },
        IcsMutation::AddExdate(t) => {
            let line = format_dt_property("EXDATE", t)?;
            append_before_subcomponents(&mut model.master, &line);
        }
        IcsMutation::AddOverrideVEvent(o) => {
            let uid = master_uid(&model.master)?;
            let block = synthesise_override_vevent(&uid, o)?;
            model.overrides.push(block);
        }
        IcsMutation::TruncateRRuleUntil(cutoff) => {
            truncate_rrule_until(&mut model.master, cutoff);
        }
        IcsMutation::RemoveOverridesAtOrAfter(cutoff) => {
            let cutoff_ts = cutoff.to_timestamp();
            let mut kept: Vec<VEvent> = Vec::new();
            for ev in model.overrides.drain(..) {
                match recurrence_id_timestamp(&ev) {
                    Ok(Some(ts)) if ts >= cutoff_ts => {}
                    _ => kept.push(ev),
                }
            }
            model.overrides = kept;
        }
    }
    Ok(())
}

fn set_or_insert(block: &mut VEvent, name: &str, new_line: &str) {
    for item in &mut block.items {
        if let VEventItem::Property(line) = item
            && extract_property_name(line) == name
        {
            new_line.clone_into(line);
            return;
        }
    }
    let pos = first_subcomponent_index(block).unwrap_or(block.items.len());
    block
        .items
        .insert(pos, VEventItem::Property(new_line.to_owned()));
}

fn remove_property(block: &mut VEvent, name: &str) {
    block.items.retain(|item| match item {
        VEventItem::Property(line) => extract_property_name(line) != name,
        VEventItem::SubComponent { .. } => true,
    });
}

fn append_before_subcomponents(block: &mut VEvent, new_line: &str) {
    let pos = first_subcomponent_index(block).unwrap_or(block.items.len());
    block
        .items
        .insert(pos, VEventItem::Property(new_line.to_owned()));
}

fn first_subcomponent_index(block: &VEvent) -> Option<usize> {
    block
        .items
        .iter()
        .position(|i| matches!(i, VEventItem::SubComponent { .. }))
}

fn truncate_rrule_until(block: &mut VEvent, cutoff: &EventTime) {
    let until_value = format_until_value(cutoff);
    for item in &mut block.items {
        if let VEventItem::Property(line) = item
            && extract_property_name(line) == "RRULE"
            && let Some(value) = line.strip_prefix("RRULE:")
        {
            let mut parts: Vec<String> = Vec::new();
            let mut had_until = false;
            for part in value.split(';') {
                if part.starts_with("UNTIL=") {
                    parts.push(format!("UNTIL={until_value}"));
                    had_until = true;
                } else {
                    parts.push(part.to_owned());
                }
            }
            if !had_until {
                parts.push(format!("UNTIL={until_value}"));
            }
            *line = format!("RRULE:{}", parts.join(";"));
            return;
        }
    }
}

fn master_uid(block: &VEvent) -> Result<String> {
    for item in &block.items {
        if let VEventItem::Property(line) = item
            && extract_property_name(line) == "UID"
            && let Some(v) = line.strip_prefix("UID:")
        {
            return Ok(v.to_owned());
        }
    }
    Err(CoreError::IcalParse(
        "patch_ics: master VEVENT missing UID".into(),
    ))
}

fn recurrence_id_timestamp(block: &VEvent) -> Result<Option<Timestamp>> {
    for item in &block.items {
        if let VEventItem::Property(line) = item
            && extract_property_name(line) == "RECURRENCE-ID"
        {
            return Ok(Some(parse_dt_property(line)?.to_timestamp()));
        }
    }
    Ok(None)
}

fn has_property(block: &VEvent, name: &str) -> bool {
    block.items.iter().any(|item| match item {
        VEventItem::Property(line) => extract_property_name(line) == name,
        VEventItem::SubComponent { .. } => false,
    })
}

fn extract_property_name(line: &str) -> &str {
    let semi = line.find(';').unwrap_or(line.len());
    let colon = line.find(':').unwrap_or(line.len());
    let end = semi.min(colon);
    &line[..end]
}

fn synthesise_override_vevent(uid: &str, ov: &OverrideInstance) -> Result<VEvent> {
    let mut items: Vec<VEventItem> = Vec::new();
    items.push(VEventItem::Property(format!("UID:{uid}")));
    let now_utc = format_utc(Timestamp::now());
    items.push(VEventItem::Property(format!("DTSTAMP:{now_utc}")));
    items.push(VEventItem::Property(format_dt_property(
        "RECURRENCE-ID",
        &ov.recurrence_id,
    )?));
    items.push(VEventItem::Property(format_dt_property(
        "DTSTART", &ov.start,
    )?));
    items.push(VEventItem::Property(format_dt_property("DTEND", &ov.end)?));
    if let Some(s) = &ov.summary {
        items.push(VEventItem::Property(format!("SUMMARY:{}", escape_text(s))));
    }
    if let Some(s) = &ov.description {
        items.push(VEventItem::Property(format!(
            "DESCRIPTION:{}",
            escape_text(s)
        )));
    }
    if let Some(s) = &ov.location {
        items.push(VEventItem::Property(format!("LOCATION:{}", escape_text(s))));
    }
    Ok(VEvent { items })
}

// ───── Emission ──────────────────────────────────────────────────────────────

fn emit(model: &Model) -> String {
    let mut out = String::new();
    out.push_str("BEGIN:VCALENDAR\r\n");
    for line in &model.pre_master {
        push_folded(&mut out, line);
    }
    emit_vevent(&mut out, &model.master);
    for o in &model.overrides {
        emit_vevent(&mut out, o);
    }
    for line in &model.post_events {
        push_folded(&mut out, line);
    }
    out.push_str("END:VCALENDAR\r\n");
    out
}

fn emit_vevent(out: &mut String, block: &VEvent) {
    out.push_str("BEGIN:VEVENT\r\n");
    for item in &block.items {
        match item {
            VEventItem::Property(line) => push_folded(out, line),
            VEventItem::SubComponent { begin, body, end } => {
                push_folded(out, begin);
                for l in body {
                    push_folded(out, l);
                }
                push_folded(out, end);
            }
        }
    }
    out.push_str("END:VEVENT\r\n");
}

fn push_folded(out: &mut String, line: &str) {
    let bytes = line.as_bytes();
    if bytes.len() <= 75 {
        out.push_str(line);
        out.push_str("\r\n");
        return;
    }
    let mut start = 0;
    let mut first = true;
    while start < bytes.len() {
        let limit = if first { 75 } else { 74 };
        let end = (start + limit).min(bytes.len());
        let mut cut = end;
        while cut > start && (bytes[cut.saturating_sub(1)] & 0b1100_0000) == 0b1000_0000 {
            cut -= 1;
        }
        if cut == start {
            cut = end;
        }
        let chunk = std::str::from_utf8(&bytes[start..cut]).unwrap_or(&line[start..cut]);
        if !first {
            out.push(' ');
        }
        out.push_str(chunk);
        out.push_str("\r\n");
        start = cut;
        first = false;
    }
}

fn unfold(raw: &str) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for chunk in raw.split('\n') {
        let trimmed = chunk.strip_suffix('\r').unwrap_or(chunk);
        if let Some(first) = trimmed.chars().next() {
            if (first == ' ' || first == '\t')
                && let Some(last) = lines.last_mut()
            {
                last.push_str(&trimmed[1..]);
                continue;
            }
        }
        if !trimmed.is_empty() {
            lines.push(trimmed.to_owned());
        }
    }
    lines
}

// ───── Datetime parsing for RECURRENCE-ID lookup ─────────────────────────────

fn parse_dt_property(line: &str) -> Result<EventTime> {
    let colon = line
        .find(':')
        .ok_or_else(|| CoreError::IcalParse(format!("malformed property: {line}")))?;
    let head = &line[..colon];
    let value = &line[colon + 1..];

    let mut params: HashMap<&str, &str> = HashMap::new();
    for param in head.split(';').skip(1) {
        if let Some((k, v)) = param.split_once('=') {
            params.insert(k, v);
        }
    }

    if params.get("VALUE") == Some(&"DATE") {
        return Ok(EventTime::AllDay {
            date: parse_yyyymmdd(value)?,
        });
    }

    if let Some(tzid) = params.get("TZID") {
        let dt = parse_floating_str(value)?;
        let tz = TimeZone::get(tzid)
            .map_err(|e| CoreError::IcalParse(format!("unknown TZID {tzid}: {e}")))?;
        let zoned = dt
            .to_zoned(tz)
            .map_err(|e| CoreError::IcalParse(format!("zoned: {e}")))?;
        return Ok(EventTime::Zoned { zoned });
    }

    if let Some(stripped) = value.strip_suffix('Z') {
        let dt = parse_floating_str(stripped)?;
        let zoned = dt
            .to_zoned(TimeZone::UTC)
            .map_err(|e| CoreError::IcalParse(format!("utc: {e}")))?;
        return Ok(EventTime::Utc {
            instant: zoned.timestamp(),
        });
    }

    Ok(EventTime::Floating {
        datetime: parse_floating_str(value)?,
    })
}

fn parse_yyyymmdd(s: &str) -> Result<JiffDate> {
    if s.len() != 8 {
        return Err(CoreError::IcalParse(format!("bad DATE: {s}")));
    }
    let y: i16 = s[0..4]
        .parse()
        .map_err(|e| CoreError::IcalParse(format!("year: {e}")))?;
    let m: i8 = s[4..6]
        .parse()
        .map_err(|e| CoreError::IcalParse(format!("month: {e}")))?;
    let d: i8 = s[6..8]
        .parse()
        .map_err(|e| CoreError::IcalParse(format!("day: {e}")))?;
    JiffDate::new(y, m, d).map_err(|e| CoreError::IcalParse(format!("date: {e}")))
}

fn parse_floating_str(text: &str) -> Result<JiffDateTime> {
    if text.len() != 15 || text.as_bytes()[8] != b'T' {
        return Err(CoreError::IcalParse(format!("bad DATE-TIME: {text}")));
    }
    let year: i16 = text[0..4]
        .parse()
        .map_err(|e| CoreError::IcalParse(format!("year: {e}")))?;
    let month: i8 = text[4..6]
        .parse()
        .map_err(|e| CoreError::IcalParse(format!("month: {e}")))?;
    let day: i8 = text[6..8]
        .parse()
        .map_err(|e| CoreError::IcalParse(format!("day: {e}")))?;
    let hour: i8 = text[9..11]
        .parse()
        .map_err(|e| CoreError::IcalParse(format!("hour: {e}")))?;
    let minute: i8 = text[11..13]
        .parse()
        .map_err(|e| CoreError::IcalParse(format!("minute: {e}")))?;
    let second: i8 = text[13..15]
        .parse()
        .map_err(|e| CoreError::IcalParse(format!("second: {e}")))?;
    JiffDateTime::new(year, month, day, hour, minute, second, 0)
        .map_err(|e| CoreError::IcalParse(format!("datetime: {e}")))
}

// ───── Datetime formatting (mirrors ical::build) ─────────────────────────────

fn format_dt_property(key: &str, t: &EventTime) -> Result<String> {
    Ok(match t {
        EventTime::AllDay { date } => format!("{key};VALUE=DATE:{}", format_date(*date)),
        EventTime::Floating { datetime } => format!("{key}:{}", format_floating(datetime)),
        EventTime::Utc { instant } => format!("{key}:{}", format_utc(*instant)),
        EventTime::Zoned { zoned } => {
            let tz = zoned
                .time_zone()
                .iana_name()
                .ok_or_else(|| CoreError::Conversion("zoned has no IANA timezone name".into()))?;
            format!("{key};TZID={tz}:{}", format_floating(&zoned.datetime()))
        }
    })
}

/// Per RFC 5545 §3.3.10, RRULE `UNTIL` must be DATE or DATE-TIME (UTC for
/// non-floating DTSTART). We always emit UTC form for absolute cutoffs.
fn format_until_value(t: &EventTime) -> String {
    match t {
        EventTime::AllDay { date } => format_date(*date),
        EventTime::Floating { datetime } => format_floating(datetime),
        EventTime::Utc { instant } => format_utc(*instant),
        EventTime::Zoned { zoned } => format_utc(zoned.timestamp()),
    }
}

fn format_date(d: JiffDate) -> String {
    format!(
        "{:04}{:02}{:02}",
        i32::from(d.year()),
        d.month().unsigned_abs(),
        d.day().unsigned_abs(),
    )
}

fn format_floating(dt: &JiffDateTime) -> String {
    format!(
        "{:04}{:02}{:02}T{:02}{:02}{:02}",
        i32::from(dt.year()),
        dt.month().unsigned_abs(),
        dt.day().unsigned_abs(),
        dt.hour().unsigned_abs(),
        dt.minute().unsigned_abs(),
        dt.second().unsigned_abs(),
    )
}

fn format_utc(t: Timestamp) -> String {
    let z = t.to_zoned(TimeZone::UTC);
    format!(
        "{:04}{:02}{:02}T{:02}{:02}{:02}Z",
        i32::from(z.year()),
        z.month().unsigned_abs(),
        z.day().unsigned_abs(),
        z.hour().unsigned_abs(),
        z.minute().unsigned_abs(),
        z.second().unsigned_abs(),
    )
}

fn escape_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            ',' => out.push_str("\\,"),
            ';' => out.push_str("\\;"),
            other => out.push(other),
        }
    }
    out
}
