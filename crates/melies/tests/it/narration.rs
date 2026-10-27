//! Narration with Kokoro (the model tier): the texts are those manuzio read from its sample book
//! for the parity cases (`melies_parity::Inputs`).

use crate::support::{Folder, speech, tools};
use kempelen::{Language, Voice};
use kuleshov::{Tool, analyze_audio};
use melies::narration::{
    Label, NarrationOptions, NarrationProgress, Narrator, Profile, SegmentKind, Text, plan, retail,
};
use melies_parity::Inputs;
use std::{
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

/// The texts manuzio narrated for the parity case `name`.
fn texts(name: &str) -> Vec<Text> {
    Inputs::committed()
        .narration(name)
        .unwrap_or_else(|| panic!("no narration case {name}"))
        .texts
        .iter()
        .map(|text| Text {
            notice: text.notice.clone(),
            title: text.title.clone(),
            subtitle: text.subtitle.clone(),
            body: text.body.clone(),
        })
        .collect()
}

/// The sample book's Chapter I, whole: one long body.
fn long_chapter() -> Text {
    let inputs = Inputs::committed();
    let text = &inputs
        .plans
        .iter()
        .find(|plan| plan.name == "oz-1")
        .expect("the plan of the sample book's Chapter I")
        .text;
    Text {
        notice: None,
        title: text.title.clone(),
        subtitle: text.subtitle.clone(),
        body: text.body.clone(),
    }
}

fn narrator() -> Narrator {
    Narrator::new(&speech(), Voice::AfAlloy, 0.9, Language::English, None).unwrap()
}

/// The longest stretch, in seconds, quieter than -45 dBFS between an audio file's first and last
/// sound (room tone and pauses count as quiet).
fn longest_inner_quiet(file: &Path) -> f64 {
    const RATE: usize = 24_000;
    let out = tools()
        .command(Tool::Ffmpeg)
        .args(["-v", "error", "-i"])
        .arg(file)
        .args(["-ac", "1", "-ar", "24000", "-f", "f32le", "-"])
        .output()
        .unwrap();
    let samples: Vec<f32> = out
        .stdout
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect();
    let loud = |s: &f32| s.abs() > 0.005_6;
    let first = samples.iter().position(loud).expect("the file has sound");
    let last = samples.iter().rposition(loud).unwrap();
    let longest = samples[first..=last]
        .split(loud)
        .map(<[f32]>::len)
        .max()
        .unwrap_or(0);
    longest as f64 / RATE as f64
}

/// The files left in `dir`.
fn files(dir: &Path) -> Vec<String> {
    std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|entry| entry.ok()?.file_name().into_string().ok())
        .collect()
}

#[test]
#[ignore = "tier:model narrates a short text with Kokoro"]
fn the_retail_profile_delivers_files_that_pass_the_stores_technical_limits() {
    let out = Folder::new("retail");
    let options = NarrationOptions {
        profile: Profile::Retail,
        ..NarrationOptions::default()
    };
    let mp3 = narrator()
        .narrate(
            &tools(),
            &texts("standard")[0],
            &options,
            &out,
            None,
            |_| {},
        )
        .unwrap()
        .expect("the Introduction has text");
    let analysis = analyze_audio(&tools(), &mp3).unwrap();
    assert_eq!(retail::problems(&analysis), Vec::new(), "{analysis:?}");
    // Only the MP3 is left, where kuleshov writes it (`audio/`): no WAV, no scratch file.
    assert_eq!(files(&out), ["audio"]);
    assert_eq!(
        files(&out.join("audio")),
        [mp3.file_name().unwrap().to_string_lossy()]
    );
}

#[test]
#[ignore = "tier:model narrates two short texts with Kokoro"]
fn a_labelled_audiobook_opens_with_the_notice_and_tags_every_file() {
    let out = Folder::new("label");
    let options = NarrationOptions {
        label: Some(Label {
            producer: "a test".into(),
            content_id: "abc123".into(),
        }),
        ..NarrationOptions::default()
    };
    let mut narrator = narrator();
    let mut texts = texts("standard");
    texts[0].notice = Some("This audiobook is read by an artificial voice.".into());
    for (index, text) in texts.iter().enumerate() {
        let mut segments = None;
        let mp3 = narrator
            .narrate(&tools(), text, &options, &out, None, |event| {
                if let NarrationProgress::SegmentFinished { segments: all, .. } = event {
                    segments = Some(all);
                }
            })
            .unwrap()
            .unwrap();
        // The notice is one more segment, at the start of the first text only.
        let planned = plan(text, &options);
        assert_eq!(segments, Some(planned.len()));
        assert_eq!(
            planned[0].kind == SegmentKind::Notice,
            index == 0,
            "{planned:?}"
        );
        let probe = tools()
            .command(Tool::Ffprobe)
            .args(["-v", "error", "-show_entries", "format_tags", "-of", "json"])
            .arg(&mp3)
            .output()
            .unwrap();
        let tags = String::from_utf8_lossy(&probe.stdout);
        assert!(
            tags.contains("AI-generated narration"),
            "file {index}: {tags}"
        );
        assert!(tags.contains("made with a test"), "file {index}: {tags}");
        assert!(tags.contains("ContentProducer"), "file {index}: {tags}");
        assert!(tags.contains("abc123"), "file {index}: {tags}");
    }
}

#[test]
#[ignore = "tier:model narrates short texts with Kokoro"]
fn ornaments_are_not_read_and_a_chapter_of_ornaments_speaks_its_heading() {
    let out = Folder::new("ornaments");
    let options = NarrationOptions {
        profile: Profile::Retail,
        ..NarrationOptions::default()
    };
    let texts = texts("ornaments");
    let mut narrator = narrator();
    let mut narrate = |text: &Text| {
        narrator
            .narrate(&tools(), text, &options, &out, None, |_| {})
            .unwrap()
            .expect("a heading at least")
    };
    // Chapter I is a lone ornament, Chapter II has no text: each speaks only its heading.
    for text in &texts[1..] {
        let heading = narrate(text);
        // "Chapter one" and the subtitle between the room tone: a few seconds, not a chapter.
        let length = analyze_audio(&tools(), &heading).unwrap().duration_secs;
        assert!(length < 9.0, "{length} s");
    }
    // The Introduction's "* * *" is not read: its plan has no asterisk and breaks the text there
    // with the closing pause, which the MP3 really has.
    let planned = plan(&texts[0], &options);
    assert!(
        planned.iter().all(|segment| !segment.text.contains('*')),
        "{planned:?}"
    );
    let before_break = planned
        .iter()
        .position(|segment| segment.text.contains("Many years later"))
        .expect("the text after the scene break is read")
        - 1;
    assert_eq!(
        planned[before_break].padding, options.closing_pause,
        "{planned:?}"
    );
    let introduction = narrate(&texts[0]);
    let pause = longest_inner_quiet(&introduction);
    assert!(
        pause > options.closing_pause - 0.2,
        "no scene-break pause in the Introduction: the longest is {pause:.2} s"
    );
    // An ornament as the only title says nothing: no file.
    let nothing = Text {
        title: Some("* * *".into()),
        body: "⁂".into(),
        ..Text::default()
    };
    assert_eq!(
        narrator
            .narrate(&tools(), &nothing, &options, &out, None, |_| {})
            .unwrap(),
        None
    );
}

#[test]
#[ignore = "tier:model synthesises part of a long text with Kokoro"]
fn cancelling_inside_a_long_chapter_body_stops_before_the_next_model_chunk() {
    let out = Folder::new("cancel");
    let cancel = AtomicBool::new(false);
    let mut cancelled_at = None;
    let mut speaking = Vec::new();
    let (mut after_cancel, mut last_report, mut longest_chunk) = (0, None, Duration::ZERO);
    let result = narrator().narrate(
        &tools(),
        &long_chapter(),
        &NarrationOptions::default(),
        &out,
        Some(&cancel),
        |event| {
            if let NarrationProgress::Speaking { done, of } = event {
                let now = Instant::now();
                if cancelled_at.is_some() {
                    after_cancel += 1;
                }
                // How long a model chunk takes here: the time between two reports.
                if let Some(last) = last_report {
                    longest_chunk = longest_chunk.max(now - last);
                }
                last_report = Some(now);
                speaking.push((done, of));
                // The body is one long segment; stop once a few of its chunks are done.
                if speaking.len() == 6 {
                    cancel.store(true, Ordering::Relaxed);
                    cancelled_at = Some(now);
                }
            }
        },
    );
    let waited = cancelled_at.expect("the body was being narrated").elapsed();
    let error = result.unwrap_err();
    assert!(error.is_cancelled(), "{error:?}");
    // The next chunk is not started: no progress after the cancel, and no longer a wait than the
    // chunk under way. Allowed: one and a half of the longest chunk seen (other tests may share
    // the processor) plus a second for the clean-up; a second chunk would take about twice.
    assert_eq!(after_cancel, 0, "progress went on after the cancel");
    assert!(longest_chunk > Duration::ZERO, "no chunk was timed");
    assert!(
        waited < longest_chunk * 3 / 2 + Duration::from_secs(1),
        "cancelling took {waited:?}; a chunk takes up to {longest_chunk:?}"
    );
    // Nothing is left: no WAV, no scratch file, no MP3.
    assert_eq!(files(&out), Vec::<String>::new());
    // Progress only moves forward.
    assert!(
        speaking
            .windows(2)
            .all(|pair| pair[0].0 <= pair[1].0 && pair[1].0 < pair[1].1)
    );
}

#[test]
#[ignore = "tier:model loads Kokoro"]
fn a_cancelled_narration_never_starts_and_leaves_nothing() {
    let out = Folder::new("cancelled");
    let cancel = AtomicBool::new(true);
    let error = narrator()
        .narrate(
            &tools(),
            &texts("standard")[0],
            &NarrationOptions::default(),
            &out,
            Some(&cancel),
            |_| panic!("no progress"),
        )
        .unwrap_err();
    assert!(error.is_cancelled());
    assert_eq!(files(&out), Vec::<String>::new());
}

#[test]
#[ignore = "tier:model loads Kokoro"]
fn without_the_gpu_runtime_narration_falls_back_to_the_cpu() {
    let missing = Folder::new("no-cuda");
    let narrator = Narrator::new(
        &speech(),
        Voice::AfAlloy,
        0.9,
        Language::English,
        Some(&missing.join("cuda")),
    )
    .unwrap();
    assert_eq!(narrator.device(), kempelen::tts::Device::Cpu);
}
