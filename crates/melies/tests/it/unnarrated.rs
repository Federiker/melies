//! Trailers without narration: each scene as long as its text takes to read, silent but for the
//! music, and the voice never loaded.

use crate::{
    support::{Folder, tools},
    trailer::{Fixture, probe, sound_level, step},
};
use kempelen::assets::SpeechData;
use kuleshov::audio_duration;
use melies::trailer::{
    Movement, OUTPUT_NAME, Trailer, WORK_DIR, render_trailer, render_with_narrator,
};
use std::{
    fs,
    sync::atomic::{AtomicBool, Ordering},
};

/// The 54 characters of the steps' text at 14 a second, padded and rounded up to whole frames.
const SCENE_FRAMES: u32 = 146;

#[test]
fn a_trailer_without_narration_lasts_as_long_as_its_texts_take_to_read_with_the_music() {
    let fx = Fixture::new();
    let trailer = Trailer {
        steps: vec![step(Movement::ZoomIn), step(Movement::LeftToRight)],
        narrate: false,
        music_volume: 0.5,
        ..Default::default()
    };
    let music = fx.music();
    let mut narrated = false;
    let video = render_with_narrator(
        &tools(),
        &fx.input(&trailer, Some(&music), None),
        |_, _| {
            narrated = true;
            Ok::<(), std::io::Error>(())
        },
        |_| {},
    )
    .unwrap();
    assert!(!narrated, "the narrator was called");
    assert_eq!(
        probe(&video, "stream=nb_frames", "v:0"),
        (2 * SCENE_FRAMES).to_string()
    );
    let seconds = f64::from(2 * SCENE_FRAMES) / 30.0;
    let (video_len, audio_len) = (
        probe(&video, "stream=duration", "v:0")
            .parse::<f64>()
            .unwrap(),
        audio_duration(&tools(), &video).unwrap(),
    );
    assert!((video_len - seconds).abs() < 0.01, "{video_len}");
    assert!((audio_len - seconds).abs() < 0.1, "{audio_len}");
    // The music plays all the way through, the middle of the scenes included.
    for start in [1.0, 2.4, 6.0] {
        let level = sound_level(&video, start, 0.5);
        assert!(level > -45.0, "no music at {start} s: {level:.1} dBFS");
    }
    assert!(!fx.out().join(WORK_DIR).exists());
}

#[test]
fn a_trailer_without_narration_never_loads_the_voice_and_is_silent_without_music() {
    let fx = Fixture::new();
    let trailer = Trailer {
        steps: vec![step(Movement::ZoomOut)],
        narrate: false,
        speed: 2.0,
        ..Default::default()
    };
    // No model here: loading the voice would fail.
    let nowhere = Folder::new("no-speech");
    let video = render_trailer(
        &tools(),
        &SpeechData::new(nowhere.join("missing")),
        &fx.input(&trailer, None, None),
        |_| {},
    )
    .unwrap();
    assert_eq!(video, fx.out().join(OUTPUT_NAME));
    // 54 characters at 28 a second, padded: 88 frames.
    assert_eq!(probe(&video, "stream=nb_frames", "v:0"), "88");
    assert_eq!(probe(&video, "stream=codec_name", "a:0"), "aac");
    let level = sound_level(&video, 0.5, 2.0);
    assert!(
        level < -70.0,
        "sound without narration or music: {level:.1} dBFS"
    );
}

#[test]
fn a_trailer_without_narration_still_needs_its_texts() {
    let fx = Fixture::new();
    let mut short = step(Movement::ZoomIn);
    short.text = "Too short a text.".into();
    let trailer = Trailer {
        steps: vec![short],
        narrate: false,
        ..Default::default()
    };
    let nowhere = Folder::new("no-speech");
    let error = render_trailer(
        &tools(),
        &SpeechData::new(nowhere.join("missing")),
        &fx.input(&trailer, None, None),
        |_| {},
    )
    .unwrap_err();
    assert_eq!(
        error.to_string(),
        "Step 1 needs at least 20 characters of text"
    );
    assert!(!fx.out().join(WORK_DIR).exists());
}

#[test]
fn cancelling_a_trailer_without_narration_keeps_the_previous_video_and_cleans_up() {
    let fx = Fixture::new();
    fs::write(fx.out().join(OUTPUT_NAME), b"previous").unwrap();
    let trailer = Trailer {
        steps: vec![step(Movement::ZoomIn), step(Movement::ZoomOut)],
        narrate: false,
        ..Default::default()
    };
    let cancel = AtomicBool::new(false);
    let mut seen = Vec::new();
    let error = render_with_narrator(
        &tools(),
        &fx.input(&trailer, None, Some(&cancel)),
        |_, _| Ok::<(), std::io::Error>(()),
        |event| {
            if matches!(
                event,
                melies::trailer::TrailerProgress::SceneFinished { index: 0, .. }
            ) {
                cancel.store(true, Ordering::Relaxed);
            }
            seen.push(event);
        },
    )
    .unwrap_err();
    assert!(error.is_cancelled(), "{error:?}");
    assert_eq!(seen.len(), 2, "the second scene must not start: {seen:?}");
    assert_eq!(fs::read(fx.out().join(OUTPUT_NAME)).unwrap(), b"previous");
    assert!(!fx.out().join(WORK_DIR).exists());
}
