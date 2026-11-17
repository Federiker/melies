use super::*;
use crate::support::{Folder, tools};

#[test]
fn silence_fills_its_slot_to_the_sample() {
    let dir = Folder::new("silence");
    let wav = dir.join("silence.wav");
    write_silence(&wav, 146.0 / 30.0).unwrap();
    let samples = 146 * 800;
    assert_eq!(sample_count(146.0 / 30.0), samples);
    let bytes = std::fs::read(&wav).unwrap();
    assert_eq!(bytes.len(), 44 + 2 * samples as usize);
    assert!(bytes[44..].iter().all(|&byte| byte == 0));
    let seconds = kuleshov::audio_duration(&tools(), &wav).unwrap();
    assert!((seconds - 146.0 / 30.0).abs() < 1e-6, "{seconds}");
}

#[test]
fn a_silence_that_cannot_be_written_names_the_file() {
    let dir = Folder::new("silence");
    let wav = dir.join("missing").join("silence.wav");
    let error = write_silence(&wav, 1.0).unwrap_err();
    assert!(
        matches!(&error, Error::File { action: "write", path, .. } if *path == wav),
        "{error:?}"
    );
}
