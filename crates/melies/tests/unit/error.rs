use super::*;

#[test]
fn a_cancelled_tool_or_synthesis_is_a_cancelled_narration() {
    assert!(Error::from(kuleshov::Error::Cancelled).is_cancelled());
    assert!(Error::from(kempelen::Error::interrupted(Stop)).is_cancelled());
    let other = Error::from(kempelen::Error::interrupted(io::Error::other(
        "voice offline",
    )));
    assert!(!other.is_cancelled());
    assert_eq!(other.to_string(), "voice offline");
}

#[test]
fn errors_name_their_file() {
    let error = Error::file_error("write", "out/trailer.mp4")(io::Error::other("disk full"));
    assert_eq!(error.file(), Some(Path::new("out/trailer.mp4")));
    assert_eq!(error.to_string(), "Failed to write out/trailer.mp4");
    let music = Error::Music {
        music: "song.opus".into(),
        source: kuleshov::Error::InvalidInput("bad".into()),
    };
    assert_eq!(music.file(), Some(Path::new("song.opus")));
    assert_eq!(Error::Cancelled.file(), None);
}

#[test]
fn media_and_animation_errors_name_kuleshovs_file() {
    let io = || kuleshov::Error::Io {
        path: "clip.mp4".into(),
        source: io::Error::other("disk full"),
    };
    assert_eq!(Error::from(io()).file(), Some(Path::new("clip.mp4")));
    let animation = Error::Animation {
        step: 2,
        source: io(),
    };
    assert_eq!(animation.file(), Some(Path::new("clip.mp4")));
    assert_eq!(
        animation.to_string(),
        "Failed to animate the image of step 2"
    );
    let speech = Error::from(kempelen::Error::NoSpeech);
    assert!(matches!(speech, Error::Speech(_)));
    assert_eq!(speech.file(), None);
    assert_eq!(Stop.to_string(), "cancelled");
}
