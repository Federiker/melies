//! Narrates a text into an MP3 and renders vertical trailers, on top of kuleshov (FFmpeg) and
//! kempelen (Kokoro speech).
//!
//! - [`narration`]: a [`narration::Narrator`] speaks an optional notice, title and subtitle and a
//!   body into one WAV, written as the speech comes, each piece followed by its silence, and
//!   encodes it once into an MP3 (variable bit rate, or the retail profile's constant 192 kbit/s
//!   with room tone), tagged as AI narration when asked.
//! - [`trailer`]: still images, narration and music become a 1080×1920, 30 FPS `trailer.mp4`,
//!   every scene a whole number of frames.
//!
//! Every function that runs FFmpeg takes kuleshov's [`kuleshov::Tools`] first; speech comes from
//! the caller's [`kempelen::assets::SpeechData`]. Errors are one enum, [`Error`].

mod error;
pub mod narration;
pub mod trailer;

pub use error::{Error, Result};

#[cfg(test)]
#[path = "../tests/unit/support.rs"]
mod support;
