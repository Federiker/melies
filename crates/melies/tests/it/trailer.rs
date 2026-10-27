use crate::support::{Folder, speech, tools, tools_dir};
use kuleshov::{Tool, audio_duration};
use melies::{
    Error,
    trailer::{
        Movement, OUTPUT_NAME, Step, Trailer, TrailerInput, TrailerProgress, WORK_DIR,
        render_trailer, render_with_narrator,
    },
};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};

const TEXT: &str = "A girl from Kansas is swept away to a land of wonders.";

/// A folder with the generated images and music (`images/`) and an empty `out/`.
struct Fixture {
    images: PathBuf,
    out: PathBuf,
    _dir: Folder,
}

impl Fixture {
    fn new() -> Self {
        let dir = Folder::new("trailer");
        let (images, out) = (dir.join("images"), dir.join("out"));
        melies_parity::generate_inputs(&tools_dir(), &images).unwrap();
        fs::create_dir_all(&out).unwrap();
        Self {
            images,
            out,
            _dir: dir,
        }
    }
    fn music(&self) -> PathBuf {
        self.images.join("music.opus")
    }
    fn out(&self) -> PathBuf {
        self.out.clone()
    }
    fn input<'a>(
        &'a self,
        trailer: &'a Trailer,
        music: Option<&'a Path>,
        cancel: Option<&'a AtomicBool>,
    ) -> TrailerInput<'a> {
        TrailerInput {
            trailer,
            language: kempelen::Language::default(),
            images_dir: &self.images,
            music,
            out_dir: &self.out,
            cancel,
        }
    }
}

/// A one-second tone instead of speech.
fn tone(_text: &str, wav: &Path) -> std::io::Result<()> {
    let status = tools()
        .command(Tool::Ffmpeg)
        .args(["-y", "-f", "lavfi", "-i", "sine=frequency=330:duration=1.0"])
        .arg(wav)
        .output()?
        .status;
    assert!(status.success());
    Ok(())
}

fn step(movement: Movement) -> Step {
    Step {
        image: Some("portrait.jpg".into()),
        text: TEXT.into(),
        movement,
    }
}

fn probe(video: &Path, entries: &str, stream: &str) -> String {
    let out = tools()
        .command(Tool::Ffprobe)
        .args([
            "-v",
            "error",
            "-select_streams",
            stream,
            "-show_entries",
            entries,
        ])
        .args(["-of", "csv=p=0"])
        .arg(video)
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// The RMS level in dBFS of the video's sound from `start` for `seconds`.
fn sound_level(video: &Path, start: f64, seconds: f64) -> f64 {
    let out = tools()
        .command(Tool::Ffmpeg)
        .args(["-v", "error", "-ss"])
        .arg(start.to_string())
        .arg("-t")
        .arg(seconds.to_string())
        .arg("-i")
        .arg(video)
        .args(["-map", "0:a:0", "-ac", "1", "-f", "f32le", "-"])
        .output()
        .unwrap();
    let samples: Vec<f32> = out
        .stdout
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect();
    assert!(!samples.is_empty(), "no sound decoded at {start} s");
    let power = samples.iter().map(|s| f64::from(*s).powi(2)).sum::<f64>() / samples.len() as f64;
    10.0 * power.max(1e-12).log10()
}

/// The mean brightness (0-255) of the video's frame at `time`.
fn frame_brightness(video: &Path, time: f64) -> f64 {
    let out = tools()
        .command(Tool::Ffmpeg)
        .args(["-v", "error", "-ss"])
        .arg(time.to_string())
        .arg("-i")
        .arg(video)
        .args([
            "-frames:v",
            "1",
            "-vf",
            "scale=36:64",
            "-pix_fmt",
            "gray",
            "-f",
            "rawvideo",
            "-",
        ])
        .output()
        .unwrap();
    assert!(!out.stdout.is_empty(), "no frame at {time} s");
    out.stdout.iter().map(|&b| f64::from(b)).sum::<f64>() / out.stdout.len() as f64
}

#[test]
fn renders_a_vertical_video_with_narration_music_and_fades() {
    let fx = Fixture::new();
    let trailer = Trailer {
        steps: vec![
            step(Movement::ZoomIn),
            step(Movement::LeftToRight),
            step(Movement::RightToLeft),
        ],
        fade_seconds: 0.4,
        music_fade_in: 0.2,
        music_fade_out: 0.2,
        ..Default::default()
    };
    let music = fx.music();
    let input = fx.input(&trailer, Some(&music), None);
    let mut events = Vec::new();
    let video = render_with_narrator(&tools(), &input, tone, |e| events.push(e)).unwrap();

    assert_eq!(video, fx.out().join(OUTPUT_NAME));
    assert_eq!(probe(&video, "stream=width,height", "v:0"), "1080,1920");
    assert_eq!(probe(&video, "stream=codec_name", "a:0"), "aac");
    assert_eq!(
        probe(&video, "stream=r_frame_rate,nb_frames", "v:0"),
        "30/1,180"
    );
    // 1 s of speech plus two half-second pads, three scenes.
    let length = audio_duration(&tools(), &video).unwrap();
    assert!((length - 6.0).abs() < 0.2, "length was {length}");
    let video_len: f64 = probe(&video, "stream=duration", "v:0").parse().unwrap();
    let audio_len: f64 = probe(&video, "stream=duration", "a:0").parse().unwrap();
    assert!(
        (video_len - audio_len).abs() < 0.1,
        "{video_len} vs {audio_len}"
    );
    assert!(!fx.out().join(WORK_DIR).exists());
    // The music plays under the pads, where the narration is silent (scene 1's tone ends at
    // 1.5 s, scene 2's starts at 2.5 s).
    let pad = sound_level(&video, 1.55, 0.4);
    assert!(pad > -45.0, "no music in the pads: {pad:.1} dBFS");
    // The scenes dip to black between them (0.4 s fades) and are bright in the middle.
    let middle = frame_brightness(&video, 1.0);
    let edge = frame_brightness(&video, 1.95);
    assert!(middle > 40.0, "scene 1 is dark in the middle: {middle:.0}");
    assert!(
        edge < middle / 4.0,
        "no fade at the end of scene 1: {edge:.0} against {middle:.0}"
    );

    assert_eq!(
        events.first(),
        Some(&TrailerProgress::SceneStarted { index: 0, total: 3 })
    );
    assert!(events.contains(&TrailerProgress::SceneFinished { index: 2, total: 3 }));
    assert!(events.contains(&TrailerProgress::Muxing));
    assert_eq!(events.last(), Some(&TrailerProgress::Finished));
}

#[test]
fn renders_without_music_or_fades() {
    let fx = Fixture::new();
    let trailer = Trailer {
        steps: vec![step(Movement::ZoomOut)],
        ..Default::default()
    };
    let input = fx.input(&trailer, None, None);
    let video = render_with_narrator(&tools(), &input, tone, |_| {}).unwrap();
    let length = audio_duration(&tools(), &video).unwrap();
    assert!((length - 2.0).abs() < 0.2, "length was {length}");
    assert_eq!(probe(&video, "stream=codec_name", "a:0"), "aac");
    assert_eq!(
        probe(&video, "stream=r_frame_rate,nb_frames", "v:0"),
        "30/1,60"
    );
    // Without music the pad after the narration is silent, and the scene does not fade out.
    let pad = sound_level(&video, 1.55, 0.4);
    assert!(pad < -70.0, "sound in a silent pad: {pad:.1} dBFS");
    let (middle, end) = (
        frame_brightness(&video, 1.0),
        frame_brightness(&video, 1.95),
    );
    assert!(
        end > middle * 0.8,
        "the scene fades out: {end:.0} against {middle:.0}"
    );
}

#[test]
fn renders_multiple_scenes_without_fades_and_keeps_frame_aligned_audio() {
    let fx = Fixture::new();
    let trailer = Trailer {
        steps: vec![step(Movement::LeftToRight), step(Movement::ZoomIn)],
        ..Default::default()
    };
    let input = fx.input(&trailer, None, None);
    let video = render_with_narrator(&tools(), &input, tone, |_| {}).unwrap();
    assert_eq!(
        probe(&video, "stream=r_frame_rate,nb_frames", "v:0"),
        "30/1,120"
    );
    let video_len: f64 = probe(&video, "stream=duration", "v:0").parse().unwrap();
    let audio_len: f64 = probe(&video, "stream=duration", "a:0").parse().unwrap();
    assert!((video_len - 4.0).abs() < 0.01);
    assert!((video_len - audio_len).abs() < 0.05);
    assert!(!fx.out().join(WORK_DIR).exists());
}

#[test]
fn cancelling_active_motion_keeps_the_previous_trailer_and_cleans_partial_media() {
    let fx = Fixture::new();
    fs::write(fx.out().join(OUTPUT_NAME), b"previous").unwrap();
    let trailer = Trailer {
        steps: vec![step(Movement::ZoomIn)],
        ..Default::default()
    };
    let cancel = std::sync::Arc::new(AtomicBool::new(false));
    let flag = cancel.clone();
    let work = fx.out().join(WORK_DIR);
    let watcher = std::thread::spawn(move || {
        let started = std::time::Instant::now();
        while started.elapsed() < std::time::Duration::from_secs(10) {
            if let Ok(files) = fs::read_dir(&work) {
                let motion_started = files
                    .filter_map(Result::ok)
                    .any(|file| file.path().extension().is_some_and(|ext| ext == "mp4"));
                if motion_started {
                    flag.store(true, Ordering::Relaxed);
                    return true;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        false
    });
    let input = fx.input(&trailer, None, Some(&cancel));
    let result = render_with_narrator(&tools(), &input, tone, |_| {});
    assert!(
        watcher.join().unwrap(),
        "FFmpeg must have opened its motion output"
    );
    assert!(result.unwrap_err().is_cancelled());
    assert_eq!(fs::read(fx.out().join(OUTPUT_NAME)).unwrap(), b"previous");
    assert!(!fx.out().join(WORK_DIR).exists());
}

#[test]
fn a_failing_narration_keeps_the_previous_video_and_cleans_up() {
    let fx = Fixture::new();
    fs::write(fx.out().join(OUTPUT_NAME), b"previous").unwrap();
    let trailer = Trailer {
        steps: vec![step(Movement::ZoomIn)],
        ..Default::default()
    };
    let input = fx.input(&trailer, None, None);
    let error = render_with_narrator(
        &tools(),
        &input,
        |_, _| Err(std::io::Error::other("voice offline")),
        |_| {},
    )
    .unwrap_err();
    assert!(
        matches!(error, Error::Narration { step: 1, .. }),
        "{error:?}"
    );
    let message = format!("{error}: {}", std::error::Error::source(&error).unwrap());
    assert_eq!(message, "Failed to narrate step 1: voice offline");
    assert_eq!(fs::read(fx.out().join(OUTPUT_NAME)).unwrap(), b"previous");
    assert!(!fx.out().join(WORK_DIR).exists());
}

#[test]
fn an_invalid_trailer_fails_before_any_work() {
    let fx = Fixture::new();
    let trailer = Trailer::default();
    let input = fx.input(&trailer, None, None);
    let mut narrated = false;
    let error = render_with_narrator(
        &tools(),
        &input,
        |_, _| {
            narrated = true;
            Ok::<(), Error>(())
        },
        |_| {},
    )
    .unwrap_err();
    assert!(error.to_string().contains("Step 1"), "{error}");
    assert!(matches!(error, Error::Trailer(_)));
    assert!(!narrated);
    assert!(!fx.out().join(WORK_DIR).exists());
}

#[test]
fn a_cancelled_render_stops_and_keeps_the_previous_video() {
    let fx = Fixture::new();
    fs::write(fx.out().join(OUTPUT_NAME), b"previous").unwrap();
    let trailer = Trailer {
        steps: vec![step(Movement::ZoomIn), step(Movement::ZoomOut)],
        ..Default::default()
    };
    let cancel = AtomicBool::new(false);
    let input = fx.input(&trailer, None, Some(&cancel));
    let mut narrated = 0;
    let error = render_with_narrator(
        &tools(),
        &input,
        |text, wav| {
            narrated += 1;
            cancel.store(true, Ordering::Relaxed);
            tone(text, wav)
        },
        |_| {},
    )
    .unwrap_err();
    assert!(error.is_cancelled(), "{error:?}");
    assert_eq!(narrated, 1, "the second scene must not start");
    assert_eq!(
        fs::read(fx.out().join(OUTPUT_NAME)).unwrap(),
        b"previous",
        "the previous video was replaced"
    );
    assert!(
        !fx.out().join(WORK_DIR).exists(),
        "the work folder was left"
    );
}

#[test]
#[ignore = "tier:model narrates a trailer with Kokoro"]
fn renders_a_short_trailer_with_a_kokoro_voice() {
    let fx = Fixture::new();
    let trailer = Trailer {
        steps: vec![step(Movement::ZoomIn)],
        voice: kempelen::Voice::AfAlloy,
        ..Default::default()
    };
    let music = fx.music();
    let input = fx.input(&trailer, Some(&music), None);
    let video = render_trailer(&tools(), &speech(), &input, |_| {}).unwrap();
    // A few seconds of speech, a whole number of frames, and the sound as long as the picture.
    let frames: u32 = probe(&video, "stream=nb_frames", "v:0").parse().unwrap();
    assert!((60..=300).contains(&frames), "{frames} frames");
    let video_len: f64 = probe(&video, "stream=duration", "v:0").parse().unwrap();
    let audio_len: f64 = probe(&video, "stream=duration", "a:0").parse().unwrap();
    assert!(
        (video_len - audio_len).abs() < 0.05,
        "{video_len} vs {audio_len}"
    );
    assert!(!fx.out().join(WORK_DIR).exists());
}
