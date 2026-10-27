use super::*;
use crate::support::{Folder, tools};

#[test]
fn no_fades_means_no_fade_filters() {
    let filter = music_filter(0.3, 0.0, 0.0, 20.0);
    assert!(!filter.contains("afade"));
    assert!(filter.contains("volume=0.3"));
    assert!(filter.contains("atrim=0:20"));
}

#[test]
fn fade_out_starts_before_the_end() {
    let filter = music_filter(0.5, 2.0, 4.0, 20.0);
    assert!(filter.contains("afade=t=in:st=0:d=2"));
    assert!(filter.contains("afade=t=out:st=16:d=4"));
    // A fade longer than the trailer starts at its beginning.
    assert!(music_filter(0.5, 0.0, 30.0, 20.0).contains("afade=t=out:st=0:d=30"));
}

#[test]
fn narration_is_not_normalised_down() {
    assert!(music_filter(0.5, 0.0, 0.0, 5.0).contains("normalize=0"));
}

fn mix(volume: f32, fade: f64) -> MusicMix {
    MusicMix {
        volume,
        fade_in: fade,
        fade_out: fade,
    }
}

#[test]
fn music_mix_is_sample_aligned_pcm_without_an_intermediate_aac_encode() {
    let tools = tools();
    let dir = Folder::new("mix");
    let voice = dir.join("voice.wav");
    let status = tools
        .command(Tool::Ffmpeg)
        .args([
            "-v",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            "aevalsrc=0.75|0.75:d=2",
            "-ar",
            "24000",
        ])
        .arg(&voice)
        .status()
        .unwrap();
    assert!(status.success());
    let mixed =
        mix_music_with_cancel(&tools, &voice, &voice, &mix(0.25, 0.2), 2.0, &dir, None).unwrap();
    let probe = tools
        .command(Tool::Ffprobe)
        .args([
            "-v",
            "error",
            "-select_streams",
            "a:0",
            "-show_entries",
            "stream=codec_name,sample_rate,channels,duration",
            "-of",
            "csv=p=0",
        ])
        .arg(&mixed)
        .output()
        .unwrap();
    assert!(probe.status.success());
    assert_eq!(
        String::from_utf8(probe.stdout).unwrap().trim(),
        "pcm_f32le,44100,2,2.000000"
    );
    let loud =
        mix_music_with_cancel(&tools, &voice, &voice, &mix(1.0, 0.0), 2.0, &dir, None).unwrap();
    let samples = tools
        .command(Tool::Ffmpeg)
        .args(["-v", "error", "-i"])
        .arg(loud)
        .args(["-f", "f32le", "pipe:1"])
        .output()
        .unwrap();
    assert!(samples.status.success());
    let peak = samples
        .stdout
        .chunks_exact(4)
        .map(|sample| f32::from_le_bytes(sample.try_into().unwrap()).abs())
        .fold(0.0_f32, f32::max);
    assert!(
        peak > 1.4,
        "intermediate mixing must preserve headroom instead of clipping: {peak}"
    );
}

#[test]
fn a_failed_mix_names_the_music_and_leaves_no_file() {
    let tools = tools();
    let dir = Folder::new("mix-fail");
    let music = dir.join("not-music.opus");
    std::fs::write(&music, b"not audio").unwrap();
    let error =
        mix_music_with_cancel(&tools, &music, &music, &mix(0.3, 0.0), 2.0, &dir, None).unwrap_err();
    assert!(matches!(error, Error::Music { .. }), "{error:?}");
    assert_eq!(error.file(), Some(music.as_path()));
    assert_eq!(std::fs::read_dir(&*dir).unwrap().count(), 1);

    let cancelled = AtomicBool::new(true);
    let error = mix_music_with_cancel(
        &tools,
        &music,
        &music,
        &mix(0.3, 0.0),
        2.0,
        &dir,
        Some(&cancelled),
    )
    .unwrap_err();
    assert!(error.is_cancelled());
}
