#![allow(clippy::float_cmp, reason = "the fades are exact values")]

use super::*;
use crate::{
    support::Folder,
    trailer::timing::{HEIGHT, WIDTH},
};
use std::{fs, path::PathBuf};

/// A PNG's signature and header chunk, `width` by `height`, without the rest of the file.
fn png_header(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = b"\x89PNG\r\n\x1a\n\0\0\0\x0dIHDR".to_vec();
    bytes.extend(width.to_be_bytes());
    bytes.extend(height.to_be_bytes());
    bytes.extend([8, 6, 0, 0, 0]);
    bytes
}

fn file(dir: &Folder, name: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, bytes).unwrap();
    path
}

#[test]
fn the_positions_are_a_quarter_the_middle_and_72_per_cent_down() {
    assert_eq!(centre(OverlayPosition::Top), 480);
    assert_eq!(centre(OverlayPosition::Middle), 960);
    assert_eq!(centre(OverlayPosition::Bottom), 1382);
    assert_eq!(OverlayPosition::default(), OverlayPosition::Bottom);
}

#[test]
fn the_largest_overlay_fits_the_frame_when_zoomed_and_halfway_through_a_pan() {
    let (width, height) = (
        f64::from(MAX_OVERLAY_WIDTH) * OVERLAY_ZOOM,
        f64::from(MAX_OVERLAY_HEIGHT) * OVERLAY_ZOOM,
    );
    assert!(width <= f64::from(WIDTH));
    for position in [
        OverlayPosition::Top,
        OverlayPosition::Middle,
        OverlayPosition::Bottom,
    ] {
        let centre = f64::from(centre(position));
        assert!(centre - height / 2.0 >= 0.0, "{position:?}");
        assert!(centre + height / 2.0 <= f64::from(HEIGHT), "{position:?}");
    }
    // At the ends of a pan the widest overlay loses at most 40 pixels to the frame's edge.
    let margin = (WIDTH - MAX_OVERLAY_WIDTH) / 2;
    assert!(OVERLAY_PAN > margin && OVERLAY_PAN - margin <= 40);
}

#[test]
fn reads_the_size_from_the_png_header() {
    let dir = Folder::new("overlay");
    assert_eq!(
        png_size(&file(&dir, "a.png", &png_header(900, 640))),
        Some((900, 640))
    );
    assert_eq!(
        png_size(&file(&dir, "b.png", &png_header(1, 70_000))),
        Some((1, 70_000))
    );
}

#[test]
fn a_file_that_is_not_a_png_has_no_size() {
    let dir = Folder::new("overlay");
    let mut jpeg = png_header(10, 10);
    jpeg[1] = b'J';
    let mut no_header = png_header(10, 10);
    no_header[12] = b'X';
    for (name, bytes) in [
        ("jpeg.png", jpeg),
        ("no-header.png", no_header),
        ("short.png", png_header(10, 10)[..20].to_vec()),
        ("empty-width.png", png_header(0, 10)),
        ("empty-height.png", png_header(10, 0)),
    ] {
        assert_eq!(png_size(&file(&dir, name, &bytes)), None, "{name}");
    }
    assert_eq!(png_size(&dir.join("missing.png")), None);
}

#[test]
fn fades_last_half_a_second_or_a_quarter_of_a_short_scene() {
    assert_eq!(fade_seconds(150), 0.5);
    assert_eq!(fade_seconds(60), 0.5);
    assert_eq!(fade_seconds(30), 0.25);
}

fn overlay(position: OverlayPosition, fade: bool) -> Overlay {
    Overlay {
        image: "text.png".into(),
        position,
        fade,
    }
}

#[test]
fn a_zooming_image_has_its_overlay_zoom_the_other_way() {
    // The image zooms in: the overlay starts at 1.2 times its size and ends at its own.
    assert_eq!(
        overlay_filter(
            &overlay(OverlayPosition::Middle, false),
            Movement::ZoomIn,
            90
        ),
        "[1:v]loop=loop=-1:size=1,setpts=N/30/TB,format=yuva444p,scale=w='iw*(1.2-(1.2-1)*n/89)':h='ih*(1.2-(1.2-1)*n/89)':\
eval=frame[ov];[bg][ov]overlay=x='(W-w)/2':y='960-h/2':format=yuv444"
    );
    // The image zooms out: the overlay starts at its own size and grows.
    assert_eq!(
        overlay_filter(&overlay(OverlayPosition::Top, false), Movement::ZoomOut, 61),
        "[1:v]loop=loop=-1:size=1,setpts=N/30/TB,format=yuva444p,scale=w='iw*(1+(1.2-1)*n/60)':h='ih*(1+(1.2-1)*n/60)':\
eval=frame[ov];[bg][ov]overlay=x='(W-w)/2':y='480-h/2':format=yuv444"
    );
}

#[test]
fn a_panning_image_has_its_overlay_travel_the_other_way() {
    // The image pans left to right: the overlay goes from 130 pixels right of the centre to 130
    // left of it.
    assert_eq!(
        overlay_filter(
            &overlay(OverlayPosition::Bottom, false),
            Movement::LeftToRight,
            90
        ),
        "[1:v]loop=loop=-1:size=1,setpts=N/30/TB,format=yuva444p[ov];[bg][ov]overlay=x='(W-w)/2+130*(1-2*n/89)':y='1382-h/2':\
format=yuv444"
    );
    assert_eq!(
        overlay_filter(
            &overlay(OverlayPosition::Bottom, false),
            Movement::RightToLeft,
            90
        ),
        "[1:v]loop=loop=-1:size=1,setpts=N/30/TB,format=yuva444p[ov];[bg][ov]overlay=x='(W-w)/2-130*(1-2*n/89)':y='1382-h/2':\
format=yuv444"
    );
}

#[test]
fn a_fading_overlay_fades_its_transparency_before_it_is_scaled() {
    assert_eq!(
        overlay_filter(&overlay(OverlayPosition::Top, true), Movement::ZoomIn, 90),
        "[1:v]loop=loop=-1:size=1,setpts=N/30/TB,format=yuva444p,fade=t=in:st=0:d=0.5:alpha=1,fade=t=out:st=2.5:d=0.5:alpha=1,\
scale=w='iw*(1.2-(1.2-1)*n/89)':h='ih*(1.2-(1.2-1)*n/89)':eval=frame[ov];\
[bg][ov]overlay=x='(W-w)/2':y='480-h/2':format=yuv444"
    );
    // A two-second scene fades over half a second, a one-second one over a quarter.
    assert!(
        overlay_filter(
            &overlay(OverlayPosition::Top, true),
            Movement::LeftToRight,
            30
        )
        .starts_with(
            "[1:v]loop=loop=-1:size=1,setpts=N/30/TB,format=yuva444p,fade=t=in:st=0:d=0.25:alpha=1,fade=t=out:st=0.75:d=0.25:alpha=1["
        )
    );
}

#[test]
fn a_one_frame_scene_does_not_divide_by_zero() {
    assert!(
        overlay_filter(&overlay(OverlayPosition::Top, false), Movement::ZoomOut, 1)
            .contains("*n/1)")
    );
}
