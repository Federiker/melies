use super::*;
use crate::trailer::OverlayPosition;

#[test]
fn frames_are_counted_as_kuleshov_counts_them() {
    assert_eq!(frame_count(146.0 / 30.0), 146);
    assert_eq!(frame_count(2.0), 60);
    // kuleshov makes a clip of at least a tenth of a second.
    assert_eq!(frame_count(0.0), 3);
}

#[test]
fn the_motion_is_kuleshovs() {
    assert_eq!(
        motion_filter(Movement::ZoomIn, 90),
        "scale=1080:1920:force_original_aspect_ratio=increase,crop=1080:1920,setsar=1,\
scale=4320:7680:flags=bicubic,format=yuv444p,zoompan=z='1+(1.25-1)*on/89':x='(iw-iw/zoom)/2':\
y='(ih-ih/zoom)/2':d=90:s=1080x1920:fps=30"
    );
    assert!(
        motion_filter(Movement::ZoomOut, 90)
            .contains("zoompan=z='1.25-(1.25-1)*on/89':x='(iw-iw/zoom)/2':")
    );
    assert!(
        motion_filter(Movement::LeftToRight, 90)
            .contains("zoompan=z='1.1':x='(iw-iw/zoom)*on/89':")
    );
    assert!(
        motion_filter(Movement::RightToLeft, 1)
            .contains("zoompan=z='1.1':x='(iw-iw/zoom)*(1-on/1)':y='(ih-ih/zoom)/2':d=1:")
    );
}

#[test]
fn scene_fades_are_kuleshovs() {
    let fades = |fade_in, fade_out| ClipFades { fade_in, fade_out };
    assert_eq!(scene_fades(fades(0.0, 0.0), 90), "");
    assert_eq!(scene_fades(fades(0.4, 0.0), 90), ",fade=t=in:st=0:d=0.4");
    assert_eq!(
        scene_fades(fades(0.4, 0.5), 90),
        ",fade=t=in:st=0:d=0.4,fade=t=out:st=2.5:d=0.5"
    );
    assert_eq!(scene_fades(fades(0.0, 1.0), 15), ",fade=t=out:st=0:d=1");
}

#[test]
fn the_scene_fades_come_after_the_overlay_so_they_darken_it() {
    let overlay = Overlay {
        image: "text.png".into(),
        position: OverlayPosition::Middle,
        fade: false,
    };
    let graph = scene_graph(
        Movement::LeftToRight,
        &overlay,
        Some(ClipFades {
            fade_in: 0.4,
            fade_out: 0.0,
        }),
        90,
    );
    assert!(graph.starts_with("[0:v]scale=1080:1920:"));
    assert!(graph.contains(
        ":fps=30[bg];[1:v]loop=loop=-1:size=1,setpts=N/30/TB,format=yuva444p[ov];[bg][ov]overlay="
    ));
    assert!(graph.ends_with(":format=yuv444,fade=t=in:st=0:d=0.4[v]"));
    assert!(
        scene_graph(Movement::ZoomIn, &overlay, None, 90).ends_with("y='960-h/2':format=yuv444[v]")
    );
}
