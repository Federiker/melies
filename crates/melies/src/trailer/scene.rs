use crate::{
    Error, Result,
    narration::check_cancelled,
    trailer::{
        Movement, Step,
        timing::{AUDIO_RATE, FPS, HEIGHT, SCENE_PADDING, WIDTH, slot_length},
    },
};
use kuleshov::{
    ClipFades, Direction, Tools, audio_duration, make_faded_motion_from_still_with_cancel,
    make_motion_from_still_with_cancel, place_in_slot_with_cancel,
};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
};

/// The two halves of a finished scene; both last exactly the same time.
pub(crate) struct Scene {
    pub clip: PathBuf,
    pub audio: PathBuf,
    pub seconds: f64,
}

pub(crate) fn direction(movement: Movement) -> Direction {
    match movement {
        Movement::ZoomIn => Direction::ZoomIn,
        Movement::ZoomOut => Direction::ZoomOut,
        Movement::LeftToRight => Direction::LeftToRight,
        Movement::RightToLeft => Direction::RightToLeft,
    }
}

/// The narrator of a trailer: `narrate(text, wav)` writes the speech of `text` to `wav`.
pub(crate) type Narrate<'a> =
    dyn FnMut(&str, &Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> + 'a;

/// Narrate the step, pad the narration into a frame-aligned slot, and animate the
/// image for that slot. With `fades`, they are applied in the same encode, which is then the
/// final one (the scenes are joined by stream copy).
#[expect(
    clippy::too_many_arguments,
    reason = "manuzio's signature, with the tools"
)]
pub(crate) fn render_scene(
    tools: &Tools,
    index: usize,
    step: &Step,
    image: &Path,
    fades: Option<ClipFades>,
    narrate: &mut Narrate<'_>,
    work: &Path,
    cancel: Option<&AtomicBool>,
) -> Result<Scene> {
    let wav = work.join(format!("narration-{index}.wav"));
    narrate(step.text.trim(), &wav).map_err(|source| Error::Narration {
        step: index + 1,
        source,
    })?;
    check_cancelled(cancel)?;
    let seconds = slot_length(audio_duration(tools, &wav)?);
    let audio = place_in_slot_with_cancel(
        tools,
        &wav,
        SCENE_PADDING,
        seconds,
        AUDIO_RATE,
        work,
        cancel,
    )?;
    let _ = fs::remove_file(&wav);
    let clip = match fades {
        Some(fades) => make_faded_motion_from_still_with_cancel(
            tools,
            image,
            WIDTH,
            HEIGHT,
            seconds,
            direction(step.movement),
            fades,
            FPS,
            work,
            cancel,
        ),
        None => make_motion_from_still_with_cancel(
            tools,
            image,
            WIDTH,
            HEIGHT,
            seconds,
            direction(step.movement),
            None,
            FPS,
            work,
            cancel,
        ),
    }
    .map_err(|source| {
        if source.is_cancelled() {
            Error::Cancelled
        } else {
            Error::Animation {
                step: index + 1,
                source,
            }
        }
    })?;
    Ok(Scene {
        clip,
        audio,
        seconds,
    })
}

#[cfg(test)]
#[path = "../../tests/unit/trailer/scene.rs"]
mod tests;
