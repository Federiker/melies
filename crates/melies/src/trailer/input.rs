use kempelen::{Language, Voice};
use std::{
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
};

/// How a scene's still image is animated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Movement {
    #[default]
    ZoomIn,
    ZoomOut,
    LeftToRight,
    RightToLeft,
}

/// Where the vertical centre of an [`Overlay`] sits in the 1080×1920 frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OverlayPosition {
    /// A quarter of the way down (y = 480).
    Top,
    /// Halfway down (y = 960).
    Middle,
    /// 72 % of the way down (y = 1382), above the captions social apps draw.
    #[default]
    Bottom,
}

/// An image drawn over a scene, such as its text: a PNG the caller makes (melies draws no text),
/// centred across the frame at its natural size and moved against the scene's [`Movement`].
///
/// Its zoom (to 1.2 times its size) or pan (12 % of the frame's width either side of the centre)
/// may crop it a little at the frame's edges, but it is whole at some moment of the scene, being
/// at most [`crate::trailer::MAX_OVERLAY_WIDTH`] by [`crate::trailer::MAX_OVERLAY_HEIGHT`]
/// pixels.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Overlay {
    /// The PNG, with transparency, anywhere on disk.
    pub image: PathBuf,
    pub position: OverlayPosition,
    /// Fade the overlay in at the scene's start and out at its end, over half a second (a quarter
    /// of a shorter scene). The scene's own dip to black darkens it either way.
    pub fade: bool,
}

/// One trailer scene: an image, the text narrated over it, and its movement.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Step {
    /// File name of the image inside the images folder ([`TrailerInput::images_dir`]).
    pub image: Option<String>,
    /// What is narrated, [`crate::trailer::MIN_TEXT_CHARS`] to
    /// [`crate::trailer::MAX_TEXT_CHARS`] characters once trimmed; without narration, it sets the
    /// scene's length.
    pub text: String,
    pub movement: Movement,
    /// What is drawn over the image, if anything.
    pub overlay: Option<Overlay>,
}

/// The scenes, narration, music and transitions of a trailer.
#[derive(Debug, Clone, PartialEq)]
pub struct Trailer {
    pub steps: Vec<Step>,
    /// Narrate the steps' texts (the default). Without narration the voice is never loaded: each
    /// scene lasts as long as its text takes to read ([`crate::trailer::READING_CHARS_PER_SECOND`]
    /// times the speed, at least [`crate::trailer::MIN_READING_SECONDS`], then the usual padding)
    /// and is silent, but for the music.
    pub narrate: bool,
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
            narrate: true,
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
