#![allow(clippy::float_cmp, reason = "the pauses and lengths are exact values")]

use super::*;
use crate::narration::Profile;

fn text(title: Option<&str>, subtitle: Option<&str>, body: &str) -> Text {
    Text {
        notice: None,
        title: title.map(str::to_string),
        subtitle: subtitle.map(str::to_string),
        body: body.into(),
    }
}

#[test]
fn the_title_is_spoken_as_it_is_given() {
    let plan = plan(
        &text(Some(" Chapter 3 "), None, "Text."),
        &NarrationOptions::default(),
    );
    assert_eq!(plan[0].text, "Chapter 3");
    assert_eq!(plan[0].kind, SegmentKind::Title);
    let plan = plan_of(&text(Some("第3章"), None, "Text."));
    assert_eq!(plan[0].text, "第3章");
}

fn plan_of(text: &Text) -> Vec<Segment> {
    plan(text, &NarrationOptions::default())
}

#[test]
fn subtitle_and_text_follow_with_their_padding() {
    let options = NarrationOptions::default();
    let plan = plan(
        &text(Some("Chapter 1"), Some("The Cyclone"), "One.\nTwo."),
        &options,
    );
    let kinds: Vec<_> = plan.iter().map(|s| s.kind).collect();
    assert_eq!(
        kinds,
        [SegmentKind::Title, SegmentKind::Subtitle, SegmentKind::Body]
    );
    assert_eq!(plan[0].padding, options.heading_pause);
    assert_eq!(plan[1].padding, options.heading_pause);
    assert_eq!(plan[2].padding, options.closing_pause);
    assert_eq!(plan[2].text, "One.\nTwo.");
}

#[test]
fn blank_pieces_are_skipped() {
    let plan = plan_of(&text(Some("Chapter I"), Some("  "), " \n "));
    // A blank subtitle and body leave only the title.
    assert_eq!(plan.len(), 1);
    assert_eq!(plan[0].kind, SegmentKind::Title);
    assert!(plan_of(&text(None, None, "")).is_empty());
}

#[test]
fn the_ai_notice_opens_the_narration_and_blank_notices_are_left_out() {
    let notice = notice_segment("  This audiobook is read by an artificial voice. ").unwrap();
    assert_eq!(notice.kind, SegmentKind::Notice);
    assert_eq!(
        notice.text,
        "This audiobook is read by an artificial voice."
    );
    assert_eq!(notice.padding, NOTICE_PADDING);
    assert!(notice_segment("   ").is_none());

    let mut labelled = text(Some("Chapter 1"), None, "Text.");
    labelled.notice = Some(" Read by an artificial voice. ".into());
    let plan = plan_of(&labelled);
    assert_eq!(
        Some(plan[0].clone()),
        notice_segment("Read by an artificial voice.")
    );
    labelled.notice = Some(" ".into());
    assert_eq!(plan_of(&labelled)[0].kind, SegmentKind::Title);
}

#[test]
fn ornaments_have_nothing_to_say_but_any_letter_or_digit_does() {
    for ornament in [
        "* * *",
        "***",
        "⁂",
        "~ ~ ~",
        "# # #",
        "• • •",
        "———",
        "…",
        "§",
    ] {
        assert!(!is_speakable(ornament), "{ornament}");
    }
    for text in ["I", "7", "Hello", "第三章", "Ωμέγα", "* 1 *"] {
        assert!(is_speakable(text), "{text}");
    }
}

#[test]
fn a_scene_break_ends_a_body_piece_and_is_never_read() {
    let options = NarrationOptions::default();
    let plan = plan(
        &text(
            Some("Chapter 1"),
            None,
            "* * *\nOne.\nTwo.\n* * *\n⁂\n\nThree.\n  \n~ ~ ~\n",
        ),
        &options,
    );
    let body: Vec<_> = plan
        .iter()
        .filter(|segment| segment.kind == SegmentKind::Body)
        .collect();
    // A break at the start or the end, or several in a row, make no extra pause.
    assert_eq!(body.len(), 2);
    assert_eq!(body[0].text, "One.\nTwo.");
    assert_eq!(body[1].text, "Three.");
    assert!(
        body.iter()
            .all(|segment| segment.padding == options.closing_pause)
    );
    assert!(plan.iter().all(|segment| is_speakable(&segment.text)));
}

#[test]
fn a_text_of_ornaments_speaks_its_title_and_an_ornament_title_nothing() {
    let plan = plan_of(&text(Some("Chapter 3"), None, "* * *\n⁂"));
    assert_eq!(plan.len(), 1);
    assert_eq!(plan[0].text, "Chapter 3");
    assert!(plan_of(&text(Some("* * *"), Some("— —"), "* * *")).is_empty());
}

#[test]
fn the_retail_profile_keeps_the_closing_pause_within_the_limits() {
    let options = NarrationOptions {
        profile: Profile::Retail,
        closing_pause: 7.0,
        ..NarrationOptions::default()
    };
    let plan = plan(&text(Some("Chapter 1"), None, "One."), &options);
    assert_eq!(plan[1].padding, crate::narration::retail::MAX_PAD_SECS);
}
