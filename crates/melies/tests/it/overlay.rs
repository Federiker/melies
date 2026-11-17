//! Overlays: drawn over the image in the scene's own encode, moved against it, faded, darkened by
//! the scene's dips to black, and leaving kuleshov's motion untouched.

use crate::{
    support::tools,
    trailer::{Fixture, frame_brightness, probe, step, tone},
};
use kuleshov::Tool;
use melies::{
    Error,
    trailer::{
        Movement, OUTPUT_NAME, Overlay, OverlayPosition, Step, Trailer, WORK_DIR,
        render_with_narrator,
    },
};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};

/// An 800×200 transparent PNG in the fixture's folder with a 600×150 box of `colour` in its
/// middle (`white`, or `white@0` for a box as transparent as the rest).
fn overlay_png(fx: &Fixture, name: &str, colour: &str) -> PathBuf {
    let path = fx.images.join(name);
    let status = tools()
        .command(Tool::Ffmpeg)
        .args(["-v", "error", "-y", "-f", "lavfi", "-i"])
        .arg(format!(
            "color=c={colour}:size=600x150,format=rgba,pad=800:200:100:25:color=black@0"
        ))
        .args(["-frames:v", "1"])
        .arg(&path)
        .status()
        .unwrap();
    assert!(status.success());
    path
}

fn with_overlay(movement: Movement, image: &Path, position: OverlayPosition, fade: bool) -> Step {
    Step {
        overlay: Some(Overlay {
            image: image.to_owned(),
            position,
            fade,
        }),
        ..step(movement)
    }
}

/// Render `steps` (narrated by a one-second tone) into a folder of their own; the video's path.
fn render(fx: &Fixture, name: &str, steps: Vec<Step>, fade_seconds: f32) -> PathBuf {
    let trailer = Trailer {
        steps,
        fade_seconds,
        ..Default::default()
    };
    let video =
        render_with_narrator(&tools(), &fx.input(&trailer, None, None), tone, |_| {}).unwrap();
    let kept = fx.images.join(name);
    fs::rename(video, &kept).unwrap();
    assert!(!fx.out().join(WORK_DIR).exists());
    kept
}

/// The grey levels of a `width`×`height` area at (`x`, `y`) of the video's frame at `time`.
fn area(video: &Path, time: f64, (x, y, width, height): (u32, u32, u32, u32)) -> Vec<u8> {
    let out = tools()
        .command(Tool::Ffmpeg)
        .args(["-v", "error", "-ss"])
        .arg(time.to_string())
        .arg("-i")
        .arg(video)
        .args(["-frames:v", "1", "-vf"])
        .arg(format!("crop={width}:{height}:{x}:{y},format=gray"))
        .args(["-f", "rawvideo", "-"])
        .output()
        .unwrap();
    assert_eq!(
        out.stdout.len(),
        (width * height) as usize,
        "no frame at {time} s"
    );
    out.stdout
}

/// The mean difference of grey level between the same area of two videos at `time`.
fn difference(a: &Path, b: &Path, time: f64, at: (u32, u32, u32, u32)) -> f64 {
    let (a, b) = (area(a, time, at), area(b, time, at));
    a.iter()
        .zip(&b)
        .map(|(a, b)| f64::from(a.abs_diff(*b)))
        .sum::<f64>()
        / a.len() as f64
}

/// The decoded pictures' MD5.
fn pictures(video: &Path) -> String {
    let out = tools()
        .command(Tool::Ffmpeg)
        .args(["-v", "error", "-i"])
        .arg(video)
        .args(["-map", "0:v", "-f", "md5", "-"])
        .output()
        .unwrap();
    String::from_utf8(out.stdout).unwrap()
}

/// The middle of the box of an overlay in the middle of the frame, when it is not moving.
const BOX: (u32, u32, u32, u32) = (300, 900, 480, 120);

#[test]
fn an_overlay_is_drawn_over_the_image_where_its_position_says() {
    let fx = Fixture::new();
    let png = overlay_png(&fx, "text.png", "white");
    let plain = render(&fx, "plain.mp4", vec![step(Movement::ZoomIn)], 0.0);
    let middle = render(
        &fx,
        "middle.mp4",
        vec![with_overlay(
            Movement::ZoomIn,
            &png,
            OverlayPosition::Middle,
            false,
        )],
        0.0,
    );
    assert_eq!(probe(&middle, "stream=nb_frames", "v:0"), "60");
    // The box covers the middle from the first frame to the last, zooming from 1.2 times its size
    // to its own; the rest of the picture is the image's.
    for time in [0.0, 1.0, 1.95] {
        let covered = difference(&plain, &middle, time, BOX);
        assert!(covered > 40.0, "no overlay at {time} s: {covered:.1}");
    }
    let above = difference(&plain, &middle, 1.0, (300, 300, 480, 120));
    assert!(above < 4.0, "the overlay spilt above itself: {above:.1}");

    let top = render(
        &fx,
        "top.mp4",
        vec![with_overlay(
            Movement::ZoomIn,
            &png,
            OverlayPosition::Top,
            false,
        )],
        0.0,
    );
    let (at_top, in_middle) = (
        difference(&plain, &top, 1.0, (300, 420, 480, 120)),
        difference(&plain, &top, 1.0, BOX),
    );
    assert!(at_top > 40.0, "no overlay at the top: {at_top:.1}");
    assert!(in_middle < 4.0, "an overlay in the middle: {in_middle:.1}");
}

#[test]
fn an_overlay_travels_against_the_pan_of_its_image() {
    let fx = Fixture::new();
    let png = overlay_png(&fx, "text.png", "white");
    let plain = render(&fx, "plain.mp4", vec![step(Movement::LeftToRight)], 0.0);
    let panned = render(
        &fx,
        "panned.mp4",
        vec![with_overlay(
            Movement::LeftToRight,
            &png,
            OverlayPosition::Middle,
            false,
        )],
        0.0,
    );
    // The image pans left to right, so the box (600 pixels wide) starts 130 pixels right of the
    // centre, at x = 370 to 970, and ends 130 pixels left of it, at about 110 to 710.
    let (left, right) = ((130, 900, 60, 100), (890, 900, 60, 100));
    let start = (
        difference(&plain, &panned, 0.0, left),
        difference(&plain, &panned, 0.0, right),
    );
    let end = (
        difference(&plain, &panned, 1.95, left),
        difference(&plain, &panned, 1.95, right),
    );
    assert!(start.0 < 4.0 && start.1 > 25.0, "at the start: {start:?}");
    assert!(end.0 > 25.0 && end.1 < 4.0, "at the end: {end:?}");
}

#[test]
fn a_fading_overlay_fades_in_and_out_and_the_scene_dips_darken_it() {
    let fx = Fixture::new();
    let png = overlay_png(&fx, "text.png", "white");
    let movements = [Movement::ZoomOut, Movement::RightToLeft, Movement::ZoomIn];
    let plain = render(
        &fx,
        "plain.mp4",
        movements.iter().map(|&m| step(m)).collect(),
        0.4,
    );
    // The first scene's overlay fades, the second's does not, the third scene has none; the
    // scenes dip to black between them.
    let faded = render(
        &fx,
        "faded.mp4",
        vec![
            with_overlay(movements[0], &png, OverlayPosition::Middle, true),
            with_overlay(movements[1], &png, OverlayPosition::Middle, false),
            step(movements[2]),
        ],
        0.4,
    );
    assert_eq!(probe(&faded, "stream=nb_frames", "v:0"), "180");
    let first = difference(&plain, &faded, 0.0, BOX);
    let middle = difference(&plain, &faded, 1.0, BOX);
    assert!(
        first < 4.0,
        "the overlay was there from the start: {first:.1}"
    );
    assert!(middle > 40.0, "no overlay halfway: {middle:.1}");
    // The second scene starts from black, its overlay too, though it does not fade itself.
    let dark = area(&faded, 2.0, BOX);
    let dark = dark.iter().map(|&v| f64::from(v)).sum::<f64>() / dark.len() as f64;
    assert!(dark < 40.0, "the dip left the overlay bright: {dark:.0}");
    let second = difference(&plain, &faded, 3.0, BOX);
    assert!(second > 40.0, "no overlay in the second scene: {second:.1}");
    let third = difference(&plain, &faded, 5.0, BOX);
    assert!(third < 4.0, "the third scene has an overlay: {third:.1}");
    assert!(frame_brightness(&faded, 5.0) > 40.0);
}

#[test]
fn a_transparent_overlay_leaves_kuleshovs_motion_and_fades_exactly() {
    let fx = Fixture::new();
    let clear = overlay_png(&fx, "clear.png", "white@0");
    for (index, (movements, fade)) in [
        ([Movement::ZoomIn, Movement::LeftToRight], 0.4),
        ([Movement::RightToLeft, Movement::ZoomOut], 0.0),
    ]
    .into_iter()
    .enumerate()
    {
        let plain = render(
            &fx,
            &format!("plain-{index}.mp4"),
            movements.iter().map(|&m| step(m)).collect(),
            fade,
        );
        let overlaid = render(
            &fx,
            &format!("overlaid-{index}.mp4"),
            movements
                .iter()
                .map(|&m| with_overlay(m, &clear, OverlayPosition::Bottom, true))
                .collect(),
            fade,
        );
        assert_eq!(pictures(&plain), pictures(&overlaid), "{movements:?}");
    }
}

#[test]
fn an_overlay_ffmpeg_cannot_decode_fails_the_animation_of_its_step() {
    let fx = Fixture::new();
    fs::write(fx.out().join(OUTPUT_NAME), b"previous").unwrap();
    // A PNG header (900×200) with nothing after it.
    let broken = fx.images.join("broken.png");
    let mut bytes = b"\x89PNG\r\n\x1a\n\0\0\0\x0dIHDR".to_vec();
    bytes.extend(900_u32.to_be_bytes());
    bytes.extend(200_u32.to_be_bytes());
    fs::write(&broken, bytes).unwrap();
    let trailer = Trailer {
        steps: vec![
            step(Movement::ZoomIn),
            with_overlay(Movement::ZoomOut, &broken, OverlayPosition::Top, false),
        ],
        ..Default::default()
    };
    let error =
        render_with_narrator(&tools(), &fx.input(&trailer, None, None), tone, |_| {}).unwrap_err();
    assert!(
        matches!(error, Error::Animation { step: 2, .. }),
        "{error:?}"
    );
    assert_eq!(error.file(), Some(fx.images.join("portrait.jpg").as_path()));
    assert_eq!(fs::read(fx.out().join(OUTPUT_NAME)).unwrap(), b"previous");
    assert!(!fx.out().join(WORK_DIR).exists());
}

#[test]
fn cancelling_a_scene_with_an_overlay_keeps_the_previous_trailer_and_cleans_up() {
    let fx = Fixture::new();
    fs::write(fx.out().join(OUTPUT_NAME), b"previous").unwrap();
    let png = overlay_png(&fx, "text.png", "white");
    let trailer = Trailer {
        steps: vec![with_overlay(
            Movement::ZoomIn,
            &png,
            OverlayPosition::Bottom,
            true,
        )],
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
        "FFmpeg must have opened its output"
    );
    assert!(result.unwrap_err().is_cancelled());
    assert_eq!(fs::read(fx.out().join(OUTPUT_NAME)).unwrap(), b"previous");
    assert!(!fx.out().join(WORK_DIR).exists());
}
