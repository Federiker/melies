/// Output frame size: vertical 9:16, as used by Instagram Reels and similar.
pub const WIDTH: u32 = 1080;
pub const HEIGHT: u32 = 1920;
pub const FPS: u32 = 30;

/// Silence kept before and after each narration, in seconds.
pub(crate) const SCENE_PADDING: f64 = 0.5;
/// Sample rate of the intermediate narration WAVs.
pub(crate) const AUDIO_RATE: u32 = 24_000;
/// Kokoro runs rushed at 1.0; the trailer narrates a little slower, like an audiobook.
pub const BASE_SPEED: f32 = 0.9;

/// Length of a scene slot: narration plus padding on both sides, rounded up to
/// whole frames so audio and video lengths match exactly.
pub(crate) fn slot_length(narration_secs: f64) -> f64 {
    let frames = ((narration_secs + 2.0 * SCENE_PADDING) * f64::from(FPS)).ceil();
    frames / f64::from(FPS)
}

/// Seconds of a fade that covers `share` of `total_secs`.
pub(crate) fn share_of(total_secs: f64, share: f32) -> f64 {
    total_secs * f64::from(share.clamp(0.0, 1.0))
}

#[cfg(test)]
#[path = "../../tests/unit/trailer/timing.rs"]
mod tests;
