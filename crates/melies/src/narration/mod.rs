//! Narrating one text: a plan of spoken pieces and pauses, the speech written into one WAV as it
//! comes, and one encode into an MP3.

pub mod label;
mod narrator;
mod options;
mod plan;
mod progress;
pub mod retail;

pub use label::Label;
pub use narrator::Narrator;
pub use options::{NarrationOptions, Profile};
pub use plan::{
    NOTICE_PADDING, REVISION, Segment, SegmentKind, Text, is_speakable, notice_segment, plan,
};
pub use progress::NarrationProgress;

pub(crate) use narrator::check_cancelled;
