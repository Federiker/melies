use crate::trailer::{Trailer, TrailerError};
use std::path::Path;

/// Fewest characters of narration per scene.
pub const MIN_TEXT_CHARS: usize = 40;
/// Most characters per scene; about 13 seconds of speech, well under 20.
pub const MAX_TEXT_CHARS: usize = 200;

/// Check a trailer before any expensive work starts.
///
/// Images are looked up by file name inside `images_dir`; `music` is the music file, when one is
/// chosen.
pub fn validate(
    trailer: &Trailer,
    images_dir: &Path,
    music: Option<&Path>,
) -> Result<(), TrailerError> {
    if trailer.steps.is_empty() {
        return Err(TrailerError::NoSteps);
    }
    for (index, step) in trailer.steps.iter().enumerate() {
        let step_no = index + 1;
        let name = step
            .image
            .as_deref()
            .ok_or(TrailerError::MissingImage { step: step_no })?;
        if !is_plain_file_name(name) {
            return Err(TrailerError::UnsafeImageName { step: step_no });
        }
        if !images_dir.join(name).is_file() {
            return Err(TrailerError::ImageNotFound { step: step_no });
        }
        let length = step.text.trim().chars().count();
        if length < MIN_TEXT_CHARS {
            return Err(TrailerError::TextTooShort {
                step: step_no,
                min: MIN_TEXT_CHARS,
            });
        }
        if length > MAX_TEXT_CHARS {
            return Err(TrailerError::TextTooLong {
                step: step_no,
                max: MAX_TEXT_CHARS,
            });
        }
    }
    if music.is_some_and(|path| !path.is_file()) {
        return Err(TrailerError::MusicNotFound);
    }
    validate_numbers(trailer)
}

fn validate_numbers(trailer: &Trailer) -> Result<(), TrailerError> {
    let ratio = |value: f32| value.is_finite() && (0.0..=1.0).contains(&value);
    if !(trailer.speed.is_finite() && trailer.speed > 0.0 && trailer.speed <= 2.0) {
        return Err(TrailerError::InvalidSetting(
            "speed must be between 0 and 2",
        ));
    }
    if !ratio(trailer.music_volume) {
        return Err(TrailerError::InvalidSetting(
            "music volume must be between 0 and 100%",
        ));
    }
    if !(trailer.fade_seconds.is_finite() && (0.0..=1.0).contains(&trailer.fade_seconds)) {
        return Err(TrailerError::InvalidSetting(
            "scene fade must be between 0 and 1 second",
        ));
    }
    if !ratio(trailer.music_fade_in) || !ratio(trailer.music_fade_out) {
        return Err(TrailerError::InvalidSetting(
            "music fades must be between 0 and 100%",
        ));
    }
    if trailer.music_fade_in + trailer.music_fade_out > 1.0 {
        return Err(TrailerError::InvalidSetting(
            "music fades together exceed the video length",
        ));
    }
    Ok(())
}

/// A bare file name: no folders, no parent references, no drive prefixes.
pub fn is_plain_file_name(name: &str) -> bool {
    !name.is_empty()
        && name != ".."
        && !name.contains(['/', '\\', ':'])
        && Path::new(name).file_name().is_some_and(|f| f == name)
}

#[cfg(test)]
#[path = "../../tests/unit/trailer/validate.rs"]
mod tests;
