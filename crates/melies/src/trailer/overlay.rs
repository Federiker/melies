//! The image drawn over a scene ([`Overlay`]): its size limits, where it sits, and the filters
//! that move it against the scene's movement.

use crate::trailer::{
    Movement, Overlay, OverlayPosition,
    timing::{FPS, HEIGHT},
};
use std::{fmt::Write as _, fs::File, io::Read as _, path::Path};

/// The widest overlay, in pixels: the frame's 1080 less a margin of 90 either side, so a pan
/// shows it whole halfway through and a zoom to [`OVERLAY_ZOOM`] keeps it inside the frame.
pub const MAX_OVERLAY_WIDTH: u32 = 900;
/// The tallest overlay, in pixels: a third of the frame.
pub const MAX_OVERLAY_HEIGHT: u32 = 640;

/// How much larger than its natural size a zooming overlay gets.
pub(crate) const OVERLAY_ZOOM: f64 = 1.2;
/// How far a panning overlay starts and ends from the centre, in pixels (12 % of the width).
pub(crate) const OVERLAY_PAN: u32 = 130;
/// The longest fade of an overlay, in seconds; a shorter scene fades over a quarter of its length.
pub(crate) const OVERLAY_FADE: f64 = 0.5;

/// The height of the overlay's vertical centre in the frame.
pub(crate) fn centre(position: OverlayPosition) -> u32 {
    match position {
        OverlayPosition::Top => HEIGHT / 4,
        OverlayPosition::Middle => HEIGHT / 2,
        OverlayPosition::Bottom => HEIGHT * 72 / 100,
    }
}

/// The width and height of the PNG at `path`, from its header; `None` when it is not a PNG or
/// cannot be read.
pub(crate) fn png_size(path: &Path) -> Option<(u32, u32)> {
    const SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n\0\0\0\x0dIHDR";
    let mut header = [0_u8; 24];
    File::open(path).ok()?.read_exact(&mut header).ok()?;
    if &header[..16] != SIGNATURE {
        return None;
    }
    let number = |at: usize| {
        u32::from_be_bytes([header[at], header[at + 1], header[at + 2], header[at + 3]])
    };
    let (width, height) = (number(16), number(20));
    (width > 0 && height > 0).then_some((width, height))
}

/// Seconds over which an overlay fades in and out in a scene of `frames` frames.
pub(crate) fn fade_seconds(frames: u32) -> f64 {
    OVERLAY_FADE.min(f64::from(frames) / f64::from(FPS) / 4.0)
}

/// The filters that turn input 1 (the overlay's PNG, read once) into `[ov]` and draw it over
/// `[bg]` (the moving image, `yuv444p`), for a scene of `frames` frames: the overlay zooms or pans
/// against `movement`, and fades in and out when asked. The graph's output is unlabelled, so the
/// caller can append the scene's own fades.
///
/// The PNG is decoded once and repeated by the `loop` filter, at [`FPS`]: looped by the demuxer
/// instead (`-loop 1`), a PNG FFmpeg cannot decode would keep it waiting for a frame for ever.
pub(crate) fn overlay_filter(overlay: &Overlay, movement: Movement, frames: u32) -> String {
    // Each frame's progress through the scene, 0 to 1 (kuleshov's zoompan uses `on` the same way).
    let progress = format!("n/{}", frames.saturating_sub(1).max(1));
    let mut layer = format!("[1:v]loop=loop=-1:size=1,setpts=N/{FPS}/TB,format=yuva444p");
    if overlay.fade {
        let fade = fade_seconds(frames);
        let length = f64::from(frames) / f64::from(FPS);
        let _ = write!(
            layer,
            ",fade=t=in:st=0:d={fade}:alpha=1,fade=t=out:st={}:d={fade}:alpha=1",
            (length - fade).max(0.0)
        );
    }
    let zoom = OVERLAY_ZOOM;
    let scale = match movement {
        // The image zooms in, so the overlay zooms out to its natural size, and the other way.
        Movement::ZoomIn => Some(format!("{zoom}-({zoom}-1)*{progress}")),
        Movement::ZoomOut => Some(format!("1+({zoom}-1)*{progress}")),
        Movement::LeftToRight | Movement::RightToLeft => None,
    };
    if let Some(scale) = scale {
        let _ = write!(layer, ",scale=w='iw*({scale})':h='ih*({scale})':eval=frame");
    }
    let pan = OVERLAY_PAN;
    let x = match movement {
        Movement::ZoomIn | Movement::ZoomOut => "(W-w)/2".to_owned(),
        // The image pans left to right, so the overlay travels right to left, and the other way.
        Movement::LeftToRight => format!("(W-w)/2+{pan}*(1-2*{progress})"),
        Movement::RightToLeft => format!("(W-w)/2-{pan}*(1-2*{progress})"),
    };
    format!(
        "{layer}[ov];[bg][ov]overlay=x='{x}':y='{}-h/2':format=yuv444",
        centre(overlay.position)
    )
}

#[cfg(test)]
#[path = "../../tests/unit/trailer/overlay.rs"]
mod tests;
