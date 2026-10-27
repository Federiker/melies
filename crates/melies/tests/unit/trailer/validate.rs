use super::*;
use crate::{support::Folder, trailer::Step};
use std::fs;

const TEXT: &str = "A girl from Kansas is swept away to a land of wonders.";

fn folder() -> Folder {
    let dir = Folder::new("validate");
    fs::write(dir.join("a.jpg"), b"x").unwrap();
    dir
}

fn trailer(text: &str, image: Option<&str>) -> Trailer {
    Trailer {
        steps: vec![Step {
            image: image.map(String::from),
            text: text.into(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

#[test]
fn a_complete_trailer_is_valid() {
    let dir = folder();
    assert_eq!(validate(&trailer(TEXT, Some("a.jpg")), &dir, None), Ok(()));
}

#[test]
fn needs_at_least_one_step() {
    let mut t = trailer(TEXT, Some("a.jpg"));
    t.steps.clear();
    assert_eq!(validate(&t, &folder(), None), Err(TrailerError::NoSteps));
}

#[test]
fn reports_the_failing_step_number() {
    let dir = folder();
    let mut t = trailer(TEXT, Some("a.jpg"));
    t.steps.push(Step {
        image: None,
        text: TEXT.into(),
        ..Default::default()
    });
    assert_eq!(
        validate(&t, &dir, None),
        Err(TrailerError::MissingImage { step: 2 })
    );
}

#[test]
fn rejects_paths_outside_the_images_folder() {
    let dir = folder();
    for name in ["../a.jpg", "sub/a.jpg", "C:\\a.jpg", "..", ""] {
        assert_eq!(
            validate(&trailer(TEXT, Some(name)), &dir, None),
            Err(TrailerError::UnsafeImageName { step: 1 }),
            "{name}"
        );
    }
    assert!(is_plain_file_name("peacock.jpg"));
    assert!(!is_plain_file_name("."));
}

#[test]
fn rejects_missing_image_files() {
    assert_eq!(
        validate(&trailer(TEXT, Some("nope.jpg")), &folder(), None),
        Err(TrailerError::ImageNotFound { step: 1 })
    );
}

#[test]
fn enforces_text_length_on_trimmed_characters() {
    let dir = folder();
    assert_eq!(
        validate(&trailer("  too short  ", Some("a.jpg")), &dir, None),
        Err(TrailerError::TextTooShort {
            step: 1,
            min: MIN_TEXT_CHARS
        })
    );
    let long = "x".repeat(MAX_TEXT_CHARS + 1);
    assert_eq!(
        validate(&trailer(&long, Some("a.jpg")), &dir, None),
        Err(TrailerError::TextTooLong {
            step: 1,
            max: MAX_TEXT_CHARS
        })
    );
    let exact = "é".repeat(MAX_TEXT_CHARS);
    assert_eq!(
        validate(&trailer(&exact, Some("a.jpg")), &dir, None),
        Ok(())
    );
    let shortest = format!("  {}  ", "a".repeat(MIN_TEXT_CHARS));
    assert_eq!(
        validate(&trailer(&shortest, Some("a.jpg")), &dir, None),
        Ok(())
    );
}

#[test]
fn rejects_missing_music() {
    let dir = folder();
    assert_eq!(
        validate(
            &trailer(TEXT, Some("a.jpg")),
            &dir,
            Some(&dir.join("m.opus"))
        ),
        Err(TrailerError::MusicNotFound)
    );
}

#[test]
fn rejects_out_of_range_numbers() {
    let dir = folder();
    let base = trailer(TEXT, Some("a.jpg"));
    for (tweak, message) in [
        (
            (|t: &mut Trailer| t.music_volume = 1.5) as fn(&mut Trailer),
            "music volume must be between 0 and 100%",
        ),
        (
            |t: &mut Trailer| t.fade_seconds = 2.0,
            "scene fade must be between 0 and 1 second",
        ),
        (
            |t: &mut Trailer| t.speed = 0.0,
            "speed must be between 0 and 2",
        ),
        (
            |t: &mut Trailer| t.speed = 2.5,
            "speed must be between 0 and 2",
        ),
        (
            |t: &mut Trailer| t.music_fade_in = f32::NAN,
            "music fades must be between 0 and 100%",
        ),
        (
            |t: &mut Trailer| {
                t.music_fade_in = 0.6;
                t.music_fade_out = 0.6;
            },
            "music fades together exceed the video length",
        ),
    ] {
        let mut t = base.clone();
        tweak(&mut t);
        assert_eq!(
            validate(&t, &dir, None),
            Err(TrailerError::InvalidSetting(message))
        );
    }
}

#[test]
fn the_messages_name_the_step() {
    for (error, message) in [
        (TrailerError::NoSteps, "The trailer needs at least one step"),
        (
            TrailerError::MissingImage { step: 2 },
            "Step 2 has no image",
        ),
        (
            TrailerError::UnsafeImageName { step: 1 },
            "Step 1 has an invalid image name",
        ),
        (
            TrailerError::ImageNotFound { step: 3 },
            "The image of step 3 was not found",
        ),
        (
            TrailerError::TextTooShort { step: 1, min: 40 },
            "Step 1 needs at least 40 characters of text",
        ),
        (
            TrailerError::TextTooLong { step: 1, max: 200 },
            "Step 1 has more than 200 characters of text",
        ),
        (
            TrailerError::MusicNotFound,
            "The selected music file was not found",
        ),
        (
            TrailerError::InvalidSetting("speed must be between 0 and 2"),
            "Invalid trailer setting: speed must be between 0 and 2",
        ),
    ] {
        assert_eq!(error.to_string(), message);
    }
}
