//! Vertical trailers built from still images, narration and music.
//!
//! Each [`Step`] becomes a scene: the text is narrated, placed in a silent slot (half a second
//! before and after), and the image is animated for exactly that slot. Scenes are stitched with an
//! optional dip to black, and the narration track, with optional background music, is muxed into
//! `trailer.mp4`. Narration and video stay in sync because every slot is a whole number of frames.

mod error;
mod input;
mod mix;
mod output;
mod progress;
mod render;
mod scene;
mod timing;
mod validate;

pub use error::TrailerError;
pub use input::{Movement, Step, Trailer, TrailerInput};
pub use output::{OUTPUT_NAME, WORK_DIR};
pub use progress::TrailerProgress;
pub use render::{render_trailer, render_with_narrator};
pub use timing::{BASE_SPEED, FPS, HEIGHT, WIDTH};
pub use validate::{MAX_TEXT_CHARS, MIN_TEXT_CHARS, is_plain_file_name, validate};
