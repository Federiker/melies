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
