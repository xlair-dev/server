//! Test fixtures shared across layers.
//!
//! These helpers are exposed via the `test-support` feature so that crates depending on `domain`
//! can compose deterministic aggregates without repeating literals.

pub mod datetime;
pub mod user;
