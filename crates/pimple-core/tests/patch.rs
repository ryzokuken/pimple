//! Tests for `pimple_core::ical::patch`.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use jiff::Timestamp;
use jiff::civil::Date;
use pimple_core::EventTime;
use pimple_core::event::{OverrideInstance, RRuleSpec};
use pimple_core::ical::patch::{IcsMutation, patch_ics};

const WITH_VALARM: &str = include_str!("fixtures/ics/with_valarm.ics");
const RECURRING_WITH_OVERRIDE: &str = include_str!("fixtures/ics/recurring_with_override.ics");
const UTC_FIXTURE: &str = include_str!("fixtures/ics/utc.ics");
const ALL_DAY_FIXTURE: &str = include_str!("fixtures/ics/all_day.ics");

// ─── Round-trip preservation ─────────────────────────────────────────────────

#[test]
fn set_master_summary_preserves_valarm_subcomponent() {
    let out = patch_ics(
        WITH_VALARM,
        &[IcsMutation::SetMasterSummary("Edited title".into())],
    )
    .unwrap();

    assert!(out.contains("BEGIN:VALARM"));
    assert!(out.contains("ACTION:DISPLAY"));
    assert!(out.contains("DESCRIPTION:Standup soon"));
    assert!(out.contains("TRIGGER:-PT15M"));
    assert!(out.contains("END:VALARM"));
}

#[test]
fn set_master_summary_preserves_x_vendor_property() {
    let out = patch_ics(
        WITH_VALARM,
        &[IcsMutation::SetMasterSummary("Edited title".into())],
    )
    .unwrap();
    assert!(out.contains("X-APPLE-CALENDAR-COLOR:#FF5733"));
}

#[test]
fn set_master_summary_replaces_in_place() {
    let out = patch_ics(
        WITH_VALARM,
        &[IcsMutation::SetMasterSummary("Edited title".into())],
    )
    .unwrap();
    assert!(out.contains("SUMMARY:Edited title"));
    assert!(!out.contains("SUMMARY:Daily standup"));
}

#[test]
fn set_master_summary_preserves_uid_and_dtstamp() {
    let out = patch_ics(
        WITH_VALARM,
        &[IcsMutation::SetMasterSummary("Edited title".into())],
    )
    .unwrap();
    assert!(out.contains("UID:meeting-with-alarm"));
    assert!(out.contains("DTSTAMP:20260101T000000Z"));
}

#[test]
fn no_mutations_returns_input_byte_for_byte_modulo_line_endings() {
    let out = patch_ics(WITH_VALARM, &[]).unwrap();
    let norm_in: String = WITH_VALARM.replace("\r\n", "\n").trim().into();
    let norm_out: String = out.replace("\r\n", "\n").trim().into();
    assert_eq!(norm_out, norm_in);
}

// ─── SetMasterDescription / SetMasterLocation ────────────────────────────────

#[test]
fn set_master_description_replaces_existing() {
    let out = patch_ics(
        WITH_VALARM,
        &[IcsMutation::SetMasterDescription(Some(
            "New description".into(),
        ))],
    )
    .unwrap();
    assert!(out.contains("DESCRIPTION:New description"));
    assert!(!out.contains("DESCRIPTION:Regular team sync"));
    // VALARM has its own DESCRIPTION — must not be touched.
    assert!(out.contains("DESCRIPTION:Standup soon"));
}

#[test]
fn set_master_description_to_none_removes_line() {
    let out = patch_ics(WITH_VALARM, &[IcsMutation::SetMasterDescription(None)]).unwrap();
    assert!(!out.contains("DESCRIPTION:Regular team sync"));
    // VALARM DESCRIPTION must still be there.
    assert!(out.contains("DESCRIPTION:Standup soon"));
}

#[test]
fn set_master_location_inserts_when_absent() {
    let out = patch_ics(
        UTC_FIXTURE,
        &[IcsMutation::SetMasterLocation(Some(
            "Conference room".into(),
        ))],
    )
    .unwrap();
    assert!(out.contains("LOCATION:Conference room"));
}

// ─── SetMasterStart / SetMasterEnd ───────────────────────────────────────────

#[test]
fn set_master_start_changes_dtstart_line() {
    let out = patch_ics(
        UTC_FIXTURE,
        &[IcsMutation::SetMasterStart(EventTime::Utc {
            instant: "2026-05-15T14:00:00Z".parse().unwrap(),
        })],
    )
    .unwrap();
    assert!(out.contains("DTSTART:20260515T140000Z"));
    // The original DTSTART was on a different time/day.
    let counts = out.matches("DTSTART").count();
    assert_eq!(counts, 1);
}

#[test]
fn set_master_end_changes_dtend_line() {
    let out = patch_ics(
        UTC_FIXTURE,
        &[IcsMutation::SetMasterEnd(EventTime::Utc {
            instant: "2026-05-15T15:00:00Z".parse().unwrap(),
        })],
    )
    .unwrap();
    assert!(out.contains("DTEND:20260515T150000Z"));
}

#[test]
fn set_master_start_to_all_day_uses_value_date_param() {
    let out = patch_ics(
        UTC_FIXTURE,
        &[IcsMutation::SetMasterStart(EventTime::AllDay {
            date: Date::constant(2026, 5, 15),
        })],
    )
    .unwrap();
    assert!(out.contains("DTSTART;VALUE=DATE:20260515"));
}

// ─── SetMasterRRule ──────────────────────────────────────────────────────────

#[test]
fn set_master_rrule_replaces_existing() {
    let out = patch_ics(
        WITH_VALARM,
        &[IcsMutation::SetMasterRRule(Some(RRuleSpec {
            line: "FREQ=DAILY;COUNT=10".into(),
        }))],
    )
    .unwrap();
    assert!(out.contains("RRULE:FREQ=DAILY;COUNT=10"));
    assert!(!out.contains("BYDAY=MO,TU,WE,TH,FR"));
}

#[test]
fn set_master_rrule_to_none_removes_line() {
    let out = patch_ics(WITH_VALARM, &[IcsMutation::SetMasterRRule(None)]).unwrap();
    assert!(!out.contains("RRULE:"));
}

#[test]
fn set_master_rrule_inserts_when_absent() {
    let out = patch_ics(
        UTC_FIXTURE,
        &[IcsMutation::SetMasterRRule(Some(RRuleSpec {
            line: "FREQ=WEEKLY".into(),
        }))],
    )
    .unwrap();
    assert!(out.contains("RRULE:FREQ=WEEKLY"));
}

// ─── AddExdate ───────────────────────────────────────────────────────────────

#[test]
fn add_exdate_zoned_appends_line_in_master() {
    let exdate = EventTime::Zoned {
        zoned: "2026-04-22T10:00:00-04:00[America/New_York]"
            .parse()
            .unwrap(),
    };
    let out = patch_ics(WITH_VALARM, &[IcsMutation::AddExdate(exdate)]).unwrap();
    assert!(out.contains("EXDATE;TZID=America/New_York:20260422T100000"));
}

#[test]
fn add_exdate_utc_uses_z_suffix() {
    let out = patch_ics(
        UTC_FIXTURE,
        &[IcsMutation::AddExdate(EventTime::Utc {
            instant: "2026-04-22T14:00:00Z".parse().unwrap(),
        })],
    )
    .unwrap();
    assert!(out.contains("EXDATE:20260422T140000Z"));
}

#[test]
fn add_exdate_all_day_uses_value_date_param() {
    let out = patch_ics(
        ALL_DAY_FIXTURE,
        &[IcsMutation::AddExdate(EventTime::AllDay {
            date: Date::constant(2026, 4, 22),
        })],
    )
    .unwrap();
    assert!(out.contains("EXDATE;VALUE=DATE:20260422"));
}

// ─── TruncateRRuleUntil ──────────────────────────────────────────────────────

#[test]
fn truncate_rrule_until_appends_until_to_existing_rule() {
    let cutoff = EventTime::Utc {
        instant: "2026-05-01T00:00:00Z".parse().unwrap(),
    };
    let out = patch_ics(WITH_VALARM, &[IcsMutation::TruncateRRuleUntil(cutoff)]).unwrap();
    assert!(out.contains("RRULE:"));
    assert!(out.contains("UNTIL=20260501T000000Z"));
    // Original rule body preserved.
    assert!(out.contains("FREQ=WEEKLY"));
    assert!(out.contains("BYDAY=MO,TU,WE,TH,FR"));
}

#[test]
fn truncate_rrule_until_replaces_pre_existing_until() {
    // First add an UNTIL, then truncate with a different one. The result should
    // have only the new UNTIL value.
    let intermediate = patch_ics(
        WITH_VALARM,
        &[IcsMutation::SetMasterRRule(Some(RRuleSpec {
            line: "FREQ=WEEKLY;UNTIL=20271231T000000Z".into(),
        }))],
    )
    .unwrap();
    let out = patch_ics(
        &intermediate,
        &[IcsMutation::TruncateRRuleUntil(EventTime::Utc {
            instant: "2026-05-01T00:00:00Z".parse().unwrap(),
        })],
    )
    .unwrap();
    assert!(out.contains("UNTIL=20260501T000000Z"));
    assert!(!out.contains("20271231"));
}

// ─── AddOverrideVEvent ───────────────────────────────────────────────────────

#[test]
fn add_override_vevent_appends_after_master() {
    let rec_id = EventTime::Zoned {
        zoned: "2026-04-22T10:00:00-04:00[America/New_York]"
            .parse()
            .unwrap(),
    };
    let new_start = EventTime::Zoned {
        zoned: "2026-04-22T11:00:00-04:00[America/New_York]"
            .parse()
            .unwrap(),
    };
    let new_end = EventTime::Zoned {
        zoned: "2026-04-22T11:30:00-04:00[America/New_York]"
            .parse()
            .unwrap(),
    };
    let override_inst = OverrideInstance {
        recurrence_id: rec_id,
        start: new_start,
        end: new_end,
        summary: Some("Standup (moved)".into()),
        description: None,
        location: None,
    };
    let out = patch_ics(
        WITH_VALARM,
        &[IcsMutation::AddOverrideVEvent(override_inst)],
    )
    .unwrap();

    let begin_count = out.matches("BEGIN:VEVENT").count();
    let end_count = out.matches("END:VEVENT").count();
    assert_eq!(begin_count, 2);
    assert_eq!(end_count, 2);
    assert!(out.contains("RECURRENCE-ID;TZID=America/New_York:20260422T100000"));
    assert!(out.contains("SUMMARY:Standup (moved)"));
    assert!(out.contains("UID:meeting-with-alarm")); // Same UID as master.
}

// ─── RemoveOverridesAtOrAfter ────────────────────────────────────────────────

#[test]
fn remove_overrides_at_or_after_strips_matching_override() {
    // recurring_with_override.ics has an override at 2026-04-22T10:00 NY.
    // A cutoff at exactly that instant should remove it (at-or-after).
    let cutoff = EventTime::Zoned {
        zoned: "2026-04-22T10:00:00-04:00[America/New_York]"
            .parse()
            .unwrap(),
    };
    let out = patch_ics(
        RECURRING_WITH_OVERRIDE,
        &[IcsMutation::RemoveOverridesAtOrAfter(cutoff)],
    )
    .unwrap();
    assert_eq!(out.matches("BEGIN:VEVENT").count(), 1);
    assert!(!out.contains("RECURRENCE-ID"));
}

#[test]
fn remove_overrides_at_or_after_keeps_earlier_override() {
    let cutoff = EventTime::Zoned {
        zoned: "2026-05-01T10:00:00-04:00[America/New_York]"
            .parse()
            .unwrap(),
    };
    let out = patch_ics(
        RECURRING_WITH_OVERRIDE,
        &[IcsMutation::RemoveOverridesAtOrAfter(cutoff)],
    )
    .unwrap();
    assert_eq!(out.matches("BEGIN:VEVENT").count(), 2);
    assert!(out.contains("RECURRENCE-ID"));
}

// ─── Composed mutations ──────────────────────────────────────────────────────

#[test]
fn multiple_mutations_apply_in_one_pass() {
    let cutoff_ts: Timestamp = "2026-05-01T00:00:00Z".parse().unwrap();
    let out = patch_ics(
        WITH_VALARM,
        &[
            IcsMutation::SetMasterSummary("Renamed".into()),
            IcsMutation::SetMasterLocation(Some("New room".into())),
            IcsMutation::TruncateRRuleUntil(EventTime::Utc { instant: cutoff_ts }),
        ],
    )
    .unwrap();

    assert!(out.contains("SUMMARY:Renamed"));
    assert!(out.contains("LOCATION:New room"));
    assert!(out.contains("UNTIL=20260501T000000Z"));
    // Untouched scaffolding survives.
    assert!(out.contains("BEGIN:VALARM"));
    assert!(out.contains("X-APPLE-CALENDAR-COLOR:#FF5733"));
}

// ─── Re-parseability via the iCalendar crate (smoke) ─────────────────────────

#[test]
fn patched_output_re_parses_through_the_icalendar_crate() {
    // After any mutation, the result must still be valid iCalendar that our
    // own parser accepts. This guards against producing malformed output that
    // would land on disk and break the next watcher-driven re-parse.
    let out = patch_ics(
        WITH_VALARM,
        &[IcsMutation::SetMasterSummary("Edited".into())],
    )
    .unwrap();
    let event =
        pimple_core::ical::parse::parse_ics(&out, pimple_core::CollectionId::new("test")).unwrap();
    assert_eq!(event.summary, "Edited");
}

// ─── Round-trip property test over the full fixture corpus ───────────────────

#[test]
fn every_fixture_survives_set_summary_round_trip() {
    // For every .ics fixture, applying SetMasterSummary must:
    //   (a) succeed,
    //   (b) produce output that re-parses cleanly,
    //   (c) preserve UID and DTSTART,
    //   (d) reflect the new summary on re-parse.
    let fixtures = [
        ("all_day.ics", ALL_DAY_FIXTURE),
        ("floating.ics", include_str!("fixtures/ics/floating.ics")),
        ("utc.ics", UTC_FIXTURE),
        ("zoned.ics", include_str!("fixtures/ics/zoned.ics")),
        ("weekly.ics", include_str!("fixtures/ics/weekly.ics")),
        (
            "weekly_with_exdate.ics",
            include_str!("fixtures/ics/weekly_with_exdate.ics"),
        ),
        (
            "with_location_and_description.ics",
            include_str!("fixtures/ics/with_location_and_description.ics"),
        ),
        ("with_valarm.ics", WITH_VALARM),
        ("recurring_with_override.ics", RECURRING_WITH_OVERRIDE),
    ];

    let cid = pimple_core::CollectionId::new("rtt");
    for (name, raw) in fixtures {
        let original = pimple_core::ical::parse::parse_ics(raw, cid.clone())
            .unwrap_or_else(|e| panic!("baseline parse of {name} failed: {e}"));

        let patched = patch_ics(
            raw,
            &[IcsMutation::SetMasterSummary(
                "Edited via round-trip".into(),
            )],
        )
        .unwrap_or_else(|e| panic!("patch_ics on {name} failed: {e}"));

        let reparsed =
            pimple_core::ical::parse::parse_ics(&patched, cid.clone()).unwrap_or_else(|e| {
                panic!("re-parse of patched {name} failed: {e}\n--- output ---\n{patched}")
            });

        assert_eq!(reparsed.uid, original.uid, "UID drift on {name}");
        assert_eq!(reparsed.start, original.start, "DTSTART drift on {name}");
        assert_eq!(reparsed.end, original.end, "DTEND drift on {name}");
        assert_eq!(
            reparsed.summary, "Edited via round-trip",
            "summary not updated on {name}"
        );
    }
}

#[test]
fn round_trip_preserves_valarm_block_byte_for_byte() {
    // VALARM body lines must survive a no-op-ish edit unchanged. Specifically
    // the ACTION/TRIGGER lines that we never touch.
    let out = patch_ics(
        WITH_VALARM,
        &[IcsMutation::SetMasterSummary("Edited".into())],
    )
    .unwrap();
    let mut in_valarm = false;
    let mut valarm_lines: Vec<&str> = Vec::new();
    for line in out.lines() {
        if line.trim() == "BEGIN:VALARM" {
            in_valarm = true;
            continue;
        }
        if line.trim() == "END:VALARM" {
            in_valarm = false;
            continue;
        }
        if in_valarm {
            valarm_lines.push(line.trim());
        }
    }
    // Order-preserving check of the VALARM body.
    assert_eq!(
        valarm_lines,
        vec![
            "ACTION:DISPLAY",
            "DESCRIPTION:Standup soon",
            "TRIGGER:-PT15M"
        ]
    );
}
