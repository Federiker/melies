//! Vertical trailers built from still images, narration and music.
//!
//! Each [`Step`] becomes a scene: the text is narrated, placed in a silent slot (half a second
//! before and after), and the image is animated for exactly that slot, with the step's
//! [`Overlay`] (a caller's PNG, such as its text) drawn over it in the same encode. Without
//! narration, each slot lasts as long as its text takes to read. Scenes are stitched with an
//! optional dip to black, and the narration track, with optional background music, is muxed into
//! `trailer.mp4`. Narration and video stay in sync because every slot is a whole number of frames.

mod error;
mod input;
mod mix;
mod motion;
mod output;
mod overlay;
mod progress;
mod render;
mod scene;
mod silence;
mod timing;
mod validate;

pub use error::TrailerError;
pub use input::{Movement, Overlay, OverlayPosition, Step, Trailer, TrailerInput};
pub use output::{OUTPUT_NAME, WORK_DIR};
pub use overlay::{MAX_OVERLAY_HEIGHT, MAX_OVERLAY_WIDTH};
pub use progress::TrailerProgress;
pub use render::{render_trailer, render_with_narrator};
pub use timing::{BASE_SPEED, FPS, HEIGHT, MIN_READING_SECONDS, READING_CHARS_PER_SECOND, WIDTH};
pub use validate::{MAX_TEXT_CHARS, MIN_TEXT_CHARS, is_plain_file_name, validate};
