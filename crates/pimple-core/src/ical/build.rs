//! Serialize a `CreateEventRequest` into a well-formed `.ics` document.

use std::fmt::Write;

use jiff::Timestamp;
use uuid::Uuid;

use crate::error::{CoreError, Result};
use crate::event::CreateEventRequest;
use crate::time::EventTime;

/// Build a minimal RFC 5545 VCALENDAR + VEVENT for `req`. Returns `(ics_text, uid)`.
///
/// # Errors
///
/// Returns [`CoreError::Conversion`] if a `Zoned` event time has no IANA timezone name.
pub fn build_ics(req: &CreateEventRequest) -> Result<(String, String)> {
    let uid = Uuid::new_v4().to_string();
    let now_utc = format_utc(Timestamp::now());

    let mut buf = String::new();
    write_line(&mut buf, "BEGIN:VCALENDAR");
    write_line(&mut buf, "VERSION:2.0");
    write_line(&mut buf, "PRODID:-//pimple//pimple v0.1//EN");
    write_line(&mut buf, "BEGIN:VEVENT");
    writeln_kv(&mut buf, "UID", &uid);
    writeln_kv(&mut buf, "DTSTAMP", &now_utc);
    write_dt(&mut buf, "DTSTART", &req.start)?;
    write_dt(&mut buf, "DTEND", &req.end)?;
    writeln_kv(&mut buf, "SUMMARY", &escape_text(&req.summary));
    if let Some(d) = &req.description {
        writeln_kv(&mut buf, "DESCRIPTION", &escape_text(d));
    }
    if let Some(l) = &req.location {
        writeln_kv(&mut buf, "LOCATION", &escape_text(l));
    }
    if let Some(rrule) = &req.rrule {
        writeln_kv(&mut buf, "RRULE", &rrule.line);
    }
    write_line(&mut buf, "END:VEVENT");
    write_line(&mut buf, "END:VCALENDAR");
    Ok((buf, uid))
}

fn write_line(buf: &mut String, line: &str) {
    buf.push_str(line);
    buf.push_str("\r\n");
}

fn writeln_kv(buf: &mut String, key: &str, value: &str) {
    let _ = writeln_kv_folded(buf, key, value);
}

/// Append `KEY:VALUE\r\n`, folding lines longer than 75 octets per RFC 5545 §3.1.
fn writeln_kv_folded(buf: &mut String, key: &str, value: &str) -> std::fmt::Result {
    let mut line = String::with_capacity(key.len() + 1 + value.len());
    write!(&mut line, "{key}:{value}")?;
    let bytes = line.as_bytes();
    if bytes.len() <= 75 {
        buf.push_str(&line);
        buf.push_str("\r\n");
        return Ok(());
    }
    // Fold at 75-byte boundaries; each continuation starts with a single space.
    let mut start = 0;
    let mut first = true;
    while start < bytes.len() {
        let end = (start + if first { 75 } else { 74 }).min(bytes.len());
        // Find a UTF-8 boundary at or before `end`.
        let mut cut = end;
        while cut > start && (bytes[cut.saturating_sub(1)] & 0b1100_0000) == 0b1000_0000 {
            cut -= 1;
        }
        let chunk = std::str::from_utf8(&bytes[start..cut]).map_err(|_| std::fmt::Error)?;
        if !first {
            buf.push(' ');
        }
        buf.push_str(chunk);
        buf.push_str("\r\n");
        start = cut;
        first = false;
    }
    Ok(())
}

fn write_dt(buf: &mut String, key: &str, t: &EventTime) -> Result<()> {
    match t {
        EventTime::AllDay { date } => {
            let v = format!(
                "{:04}{:02}{:02}",
                i32::from(date.year()),
                date.month().unsigned_abs(),
                date.day().unsigned_abs(),
            );
            writeln_kv(buf, &format!("{key};VALUE=DATE"), &v);
        }
        EventTime::Floating { datetime } => {
            let v = format_floating(datetime);
            writeln_kv(buf, key, &v);
        }
        EventTime::Utc { instant } => {
            let v = format_utc(*instant);
            writeln_kv(buf, key, &v);
        }
        EventTime::Zoned { zoned } => {
            let tz = zoned
                .time_zone()
                .iana_name()
                .ok_or_else(|| CoreError::Conversion("zoned has no IANA tz".into()))?;
            let v = format_floating(&zoned.datetime());
            writeln_kv(buf, &format!("{key};TZID={tz}"), &v);
        }
    }
    Ok(())
}

fn format_utc(t: Timestamp) -> String {
    let z = t.to_zoned(jiff::tz::TimeZone::UTC);
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

fn format_floating(dt: &jiff::civil::DateTime) -> String {
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
