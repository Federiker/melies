#![allow(clippy::float_cmp, reason = "the pauses and lengths are exact values")]

use super::*;

#[test]
fn slot_is_padded_and_frame_aligned() {
    let slot = slot_length(2.0);
    assert!(slot >= 3.0);
    let frames = slot * f64::from(FPS);
    assert!((frames - frames.round()).abs() < 1e-9);
    assert!(slot - 3.0 < 1.0 / f64::from(FPS) + 1e-9);
}

#[test]
fn share_is_clamped_to_the_whole_length() {
    assert_eq!(share_of(10.0, 0.25), 2.5);
    assert_eq!(share_of(10.0, 3.0), 10.0);
    assert_eq!(share_of(10.0, -1.0), 0.0);
}
