//! What keeps melies' output the same as manuzio's: the cases both sides run, the inputs and plans
//! recorded from manuzio's crates, a recording shim that also hashes the WAVs handed to FFmpeg, and
//! the comparison against the references (docs/parity.md).
//!
//! The recording itself is kuleshov's ([`kuleshov_parity::Session`]): every `ffmpeg` and `ffprobe`
//! call with its arguments, and every returned file's bytes and decoded hashes.

mod book;
mod cases;
mod fixtures;
mod recording;
mod run;

pub use book::{SAMPLE_BOOK, write_book};
pub use cases::{
    Book, Label, NARRATIONS, NarrationCase, PlanCase, RecordedNarration, STEP_TEXTS, Segment,
    TRAILERS, Text, TrailerCase, TrailerStep,
};
pub use fixtures::{generate_inputs, tone};
pub use kempelen_parity::Machine;
pub use kuleshov_parity::{Outcome, Reference, Session, compare as compare_runs, platform};
pub use recording::{INPUTS_FILE, Inputs, Recording, compare, model_scenarios};
pub use run::{Engine, INPUTS, narration_scenario, record, trailer_scenario};
