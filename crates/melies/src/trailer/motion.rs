//! A scene with an overlay: the image's motion, the overlay and the scene's fades in one encode.
//!
//! kuleshov makes the motion of a scene without an overlay, but does not expose its filter, so the
//! motion is written again here, as kuleshov's `motion_filter` builds it for this frame (its
//! defaults: zooms of 1.25, pans at 1.10, a working image four times the frame's size). A test
//! checks that a fully transparent overlay gives kuleshov's frames exactly.

use crate::trailer::{
    Movement, Overlay,
    overlay::overlay_filter,
    timing::{FPS, HEIGHT, WIDTH},
};
use kuleshov::{ClipFades, Tool, Tools, run_cancellable};
use std::{
    fmt::Write as _,
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
};
use uuid::Uuid;

/// The frames of a scene of `seconds`, as kuleshov counts them.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a scene of seconds, a whole number of frames"
)]
pub(crate) fn frame_count(seconds: f64) -> u32 {
    (seconds.max(0.1) * f64::from(FPS)).round().max(1.0) as u32
}

/// kuleshov's motion filter for a 1080×1920 frame at 30 FPS, from the still image to `yuv444p`.
pub(crate) fn motion_filter(movement: Movement, frames: u32) -> String {
    const FACTOR: u32 = 4;
    let zoom = match movement {
        Movement::ZoomIn | Movement::ZoomOut => 1.25,
        Movement::LeftToRight | Movement::RightToLeft => 1.1,
    };
    let position = format!("on/{}", frames.saturating_sub(1).max(1));
    let (z, x) = match movement {
        Movement::ZoomIn => (
            format!("1+({zoom}-1)*{position}"),
            "(iw-iw/zoom)/2".to_owned(),
        ),
        Movement::ZoomOut => (
            format!("{zoom}-({zoom}-1)*{position}"),
            "(iw-iw/zoom)/2".to_owned(),
        ),
        Movement::LeftToRight => (zoom.to_string(), format!("(iw-iw/zoom)*{position}")),
        Movement::RightToLeft => (zoom.to_string(), format!("(iw-iw/zoom)*(1-{position})")),
    };
    let (w, h) = (WIDTH * FACTOR, HEIGHT * FACTOR);
    format!(
        "scale={WIDTH}:{HEIGHT}:force_original_aspect_ratio=increase,\
crop={WIDTH}:{HEIGHT},setsar=1,scale={w}:{h}:flags=bicubic,format=yuv444p,\
zoompan=z='{z}':x='{x}':y='(ih-ih/zoom)/2':d={frames}:s={WIDTH}x{HEIGHT}:fps={FPS}"
    )
}

/// The scene's dips to black, as kuleshov appends them, after the overlay so they darken it too.
pub(crate) fn scene_fades(fades: ClipFades, frames: u32) -> String {
    let mut filter = String::new();
    if fades.fade_in > 0.0 {
        let _ = write!(filter, ",fade=t=in:st=0:d={}", fades.fade_in);
    }
    if fades.fade_out > 0.0 {
        let length = f64::from(frames) / f64::from(FPS);
        let _ = write!(
            filter,
            ",fade=t=out:st={}:d={}",
            (length - fades.fade_out).max(0.0),
            fades.fade_out
        );
    }
    filter
}

/// The whole filter graph of a scene with an overlay, ending in `[v]`.
pub(crate) fn scene_graph(
    movement: Movement,
    overlay: &Overlay,
    fades: Option<ClipFades>,
    frames: u32,
) -> String {
    format!(
        "[0:v]{}[bg];{}{}[v]",
        motion_filter(movement, frames),
        overlay_filter(overlay, movement, frames),
        fades
            .map(|fades| scene_fades(fades, frames))
            .unwrap_or_default()
    )
}

/// Animate `image` for `seconds` with `overlay` over it, into a new MP4 in `work`, encoded as
/// kuleshov encodes a scene: x264's quality 23 with `fades` (the final encode, joined by stream
/// copy), 8 Mbit/s without. A failed or cancelled encode removes its output.
#[expect(
    clippy::too_many_arguments,
    reason = "kuleshov's motion arguments, plus the overlay"
)]
pub(crate) fn make_overlaid_motion(
    tools: &Tools,
    image: &Path,
    movement: Movement,
    overlay: &Overlay,
    seconds: f64,
    fades: Option<ClipFades>,
    work: &Path,
    cancel: Option<&AtomicBool>,
) -> kuleshov::Result<PathBuf> {
    let frames = frame_count(seconds);
    let output = work.join(format!("{}.mp4", Uuid::new_v4()));
    let mut command = tools.command(Tool::Ffmpeg);
    command
        .args(["-y", "-loop", "1", "-framerate"])
        .arg(FPS.to_string())
        .arg("-i")
        .arg(image)
        // Read once, looped in the graph (`overlay_filter`).
        .arg("-framerate")
        .arg(FPS.to_string())
        .arg("-i")
        .arg(&overlay.image)
        .arg("-filter_complex")
        .arg(scene_graph(movement, overlay, fades, frames))
        .args(["-map", "[v]", "-frames:v"])
        .arg(frames.to_string())
        .args(["-c:v", "libx264", "-preset", "medium"])
        .args(if fades.is_some() {
            ["-crf", "23"]
        } else {
            ["-b:v", "8M"]
        })
        .args(["-pix_fmt", "yuv420p", "-an"])
        .arg(&output);
    if let Err(error) = run_cancellable(&mut command, cancel) {
        let _ = std::fs::remove_file(&output);
        return Err(match error {
            kuleshov::Error::Failed {
                program,
                file: None,
                code,
                stderr,
            } => kuleshov::Error::Failed {
                program,
                file: Some(image.to_owned()),
                code,
                stderr,
            },
            error => error,
        });
    }
    Ok(output)
}

#[cfg(test)]
#[path = "../../tests/unit/trailer/motion.rs"]
mod tests;
