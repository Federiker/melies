use kempelen::{Language, Voice};
use std::{path::Path, sync::atomic::AtomicBool};

/// How a scene's still image is animated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Movement {
    #[default]
    ZoomIn,
    ZoomOut,
    LeftToRight,
    RightToLeft,
}

/// One trailer scene: an image, the text narrated over it, and its movement.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Step {
    /// File name of the image inside the images folder ([`TrailerInput::images_dir`]).
    pub image: Option<String>,
    /// What is narrated, [`crate::trailer::MIN_TEXT_CHARS`] to
    /// [`crate::trailer::MAX_TEXT_CHARS`] characters once trimmed.
    pub text: String,
    pub movement: Movement,
}

/// The scenes, narration, music and transitions of a trailer.
#[derive(Debug, Clone, PartialEq)]
pub struct Trailer {
    pub steps: Vec<Step>,
    pub voice: Voice,
    /// Speech speed, above 0 and at most 2, times [`crate::trailer::BASE_SPEED`].
    pub speed: f32,
    /// Music level, 0 to 1; the narration is never attenuated.
    pub music_volume: f32,
    /// Seconds of dip-to-black between scenes, 0 to 1; 0 disables it.
    pub fade_seconds: f32,
    /// Share of the final length over which the music fades in, 0 to 1.
    pub music_fade_in: f32,
    /// Share of the final length over which the music fades out, 0 to 1; with the fade-in, at
    /// most the whole length.
    pub music_fade_out: f32,
}

impl Default for Trailer {
    fn default() -> Self {
        Self {
            steps: vec![Step::default()],
            voice: Voice::default(),
            speed: 1.0,
            music_volume: 0.3,
            fade_seconds: 0.0,
            music_fade_in: 0.0,
            music_fade_out: 0.0,
        }
    }
}

/// Everything needed to render a trailer.
pub struct TrailerInput<'a> {
    pub trailer: &'a Trailer,
    /// Language the step texts are written in; the voice pronounces them in it.
    pub language: Language,
    /// Folder holding the step images, referenced by file name.
    pub images_dir: &'a Path,
    /// Background music file, if any; it loops under the whole trailer.
    pub music: Option<&'a Path>,
    /// Receives `trailer.mp4`; an existing file is replaced only on success.
    pub out_dir: &'a Path,
    /// Set to stop the render before the next stage, or FFmpeg at once; the previous video is kept.
    pub cancel: Option<&'a AtomicBool>,
}
