#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use jiff::civil::Date;

use pimple_core::ical::build::build_ics;
use pimple_core::{CollectionId, CreateEventRequest, EventTime};

#[test]
fn builds_minimal_utc_event() {
    let req = CreateEventRequest {
        collection_id: CollectionId::new("personal"),
        summary: "Coffee".into(),
        description: None,
        location: None,
        start: EventTime::Utc {
            instant: "2026-04-25T15:30:00Z".parse().unwrap(),
        },
        end: EventTime::Utc {
            instant: "2026-04-25T16:30:00Z".parse().unwrap(),
        },
        rrule: None,
    };
    let (ics, uid) = build_ics(&req).unwrap();
    assert!(uid.contains('-'), "UID should look like a UUID");
    assert!(ics.starts_with("BEGIN:VCALENDAR"));
    assert!(ics.ends_with("END:VCALENDAR\r\n"));
    assert!(ics.contains("DTSTART:20260425T153000Z"));
    assert!(ics.contains("DTEND:20260425T163000Z"));
    assert!(ics.contains("SUMMARY:Coffee"));
    assert!(ics.contains(&format!("UID:{uid}")));
}

#[test]
fn builds_all_day_event_with_value_date() {
    let req = CreateEventRequest {
        collection_id: CollectionId::new("personal"),
        summary: "Holiday".into(),
        description: None,
        location: None,
        start: EventTime::AllDay {
            date: Date::constant(2026, 4, 25),
        },
        end: EventTime::AllDay {
            date: Date::constant(2026, 4, 26),
        },
        rrule: None,
    };
    let (ics, _uid) = build_ics(&req).unwrap();
    assert!(ics.contains("DTSTART;VALUE=DATE:20260425"));
    assert!(ics.contains("DTEND;VALUE=DATE:20260426"));
}

#[test]
fn escapes_text_fields_per_rfc_5545() {
    let req = CreateEventRequest {
        collection_id: CollectionId::new("personal"),
        summary: "Quarterly review; bring the deck, please.".into(),
        description: Some("Line 1\nLine 2".into()),
        location: None,
        start: EventTime::Utc {
            instant: "2026-04-25T15:30:00Z".parse().unwrap(),
        },
        end: EventTime::Utc {
            instant: "2026-04-25T16:30:00Z".parse().unwrap(),
        },
        rrule: None,
    };
    let (ics, _uid) = build_ics(&req).unwrap();
    assert!(ics.contains("SUMMARY:Quarterly review\\; bring the deck\\, please."));
    assert!(ics.contains("DESCRIPTION:Line 1\\nLine 2"));
}

#[test]
fn round_trips_through_parse() {
    use pimple_core::ical::parse::parse_ics;

    let req = CreateEventRequest {
        collection_id: CollectionId::new("personal"),
        summary: "Round-trip".into(),
        description: Some("Hello\nworld".into()),
        location: Some("Berlin, Germany".into()),
        start: EventTime::Zoned {
            zoned: "2026-04-25T15:30:00[Europe/Berlin]".parse().unwrap(),
        },
        end: EventTime::Zoned {
            zoned: "2026-04-25T16:30:00[Europe/Berlin]".parse().unwrap(),
        },
        rrule: None,
    };
    let (ics, _uid) = build_ics(&req).unwrap();
    let event = parse_ics(&ics, CollectionId::new("personal")).unwrap();
    assert_eq!(event.summary, "Round-trip");
    assert_eq!(event.description.as_deref(), Some("Hello\nworld"));
    assert_eq!(event.location.as_deref(), Some("Berlin, Germany"));
}
