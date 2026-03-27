// Unlicense — cochranblock.org
// Contributors: GotEmCoach, KOVA, Claude Opus 4.6

//! Pocket Server — your website lives on your phone.
//! A compiled Rust web server that runs as an Android foreground service
//! with a bold kiosk-style dashboard showing live stats.

pub mod server;
pub mod stats;
pub mod tunnel;

#[cfg(target_os = "android")]
pub mod android;
