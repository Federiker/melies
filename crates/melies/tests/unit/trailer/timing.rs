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

#[test]
fn reading_takes_fourteen_trimmed_characters_a_second_divided_by_the_speed() {
    let text = format!("  {}\n", "a".repeat(70));
    assert_eq!(reading_seconds(&text, 1.0), 5.0);
    assert_eq!(reading_seconds(&text, 2.0), 2.5);
    assert_eq!(reading_seconds(&text, 0.5), 10.0);
    // Characters, not bytes.
    assert_eq!(reading_seconds(&"é".repeat(28), 1.0), 2.0);
}

#[test]
fn reading_takes_at_least_one_and_a_half_seconds() {
    assert_eq!(reading_seconds(&"a".repeat(20), 1.0), MIN_READING_SECONDS);
    assert_eq!(reading_seconds(&"a".repeat(20), 2.0), MIN_READING_SECONDS);
    // 54 characters at speed 1, padded and rounded up to 146 frames.
    let text = "A girl from Kansas is swept away to a land of wonders.";
    assert_eq!(slot_length(reading_seconds(text, 1.0)), 146.0 / 30.0);
}
