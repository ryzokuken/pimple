//! iCalendar parsing, expansion, and serialization.
//!
//! This module is the only place in the crate that depends on `chrono` and
//! `chrono-tz`. The `from_chrono` / `to_chrono` submodules bridge to and from
//! `jiff` types for the rest of the crate.

pub mod expand;
pub mod from_chrono;
pub mod parse;
pub mod to_chrono;
