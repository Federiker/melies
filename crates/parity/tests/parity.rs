//! The parity test: every case of `melies_parity`, run through melies over the inputs recorded
//! from manuzio (`references/`, docs/parity.md), must plan the same pieces, call ffmpeg and
//! ffprobe with the same arguments, hand them the same WAVs and write the same files.
//!
//! The cases are those the harness ran through manuzio's crates, one for one: same names, same
//! order, same inputs.
#![expect(
    clippy::unwrap_used,
    reason = "the scenarios are the test: a failure is a panic"
)]

use kempelen::{Language, Voice, assets::SpeechData};
use kuleshov::Tools;
use melies::{
    narration::{Label, NarrationOptions, Narrator, Profile, SegmentKind, Text, plan},
    trailer::{Movement, Step, Trailer, TrailerInput, render_trailer, render_with_narrator},
};
use melies_parity::{
    Engine, Inputs, Machine, NarrationCase, Recording, Segment, Session, TrailerCase, compare,
    record, tone,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// The folder `variable` names (relative to the workspace), else `fallback` inside it.
fn folder(variable: &str, fallback: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    std::env::var_os(variable).map_or_else(|| root.join(fallback), |dir| root.join(dir))
}

/// The real FFmpeg: `MELIES_TOOLS_DIR`, else `tools/` (`mise run stage`).
fn tools_dir() -> PathBuf {
    let dir = folder("MELIES_TOOLS_DIR", "tools");
    assert!(
        dir.join(format!("ffmpeg{}", std::env::consts::EXE_SUFFIX))
            .is_file(),
        "no ffmpeg in {}: set MELIES_TOOLS_DIR or run `mise run stage`",
        dir.display()
    );
    dir
}

/// The staged Kokoro files: `MELIES_SPEECH_DIR`, else `tools/speech` (`mise run stage`).
fn speech() -> SpeechData {
    let dir = folder("MELIES_SPEECH_DIR", "tools/speech");
    assert!(
        dir.join("models").is_dir(),
        "no Kokoro model in {}: set MELIES_SPEECH_DIR or run `mise run stage`",
        dir.display()
    );
    SpeechData::new(dir)
}

/// This platform's committed recording.
fn recording() -> Recording {
    let platform = melies_parity::platform();
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("references")
        .join(format!("{platform}.json"));
    Recording::read(&path).unwrap_or_else(|error| {
        panic!(
            "{}: {error} (no references for {platform}: docs/parity.md)",
            path.display()
        )
    })
}

/// A value from its serialised name, as the cases give it.
fn parse<T: serde::de::DeserializeOwned>(name: &str) -> T {
    serde_json::from_value(serde_json::Value::String(name.to_owned()))
        .unwrap_or_else(|error| panic!("{name:?}: {error}"))
}

/// manuzio's name for a segment kind, as the plans were recorded.
fn kind_name(kind: SegmentKind) -> &'static str {
    match kind {
        SegmentKind::Notice => "notice",
        SegmentKind::Title => "heading",
        SegmentKind::Subtitle => "subtitle",
        SegmentKind::Body => "body",
    }
}

fn text(recorded: &melies_parity::Text) -> Text {
    Text {
        notice: recorded.notice.clone(),
        title: recorded.title.clone(),
        subtitle: recorded.subtitle.clone(),
        body: recorded.body.clone(),
    }
}

/// melies, as manuzio will call it after switching.
struct Melies {
    tools: Tools,
    speech: Option<SpeechData>,
    inputs: Inputs,
}

impl Engine for Melies {
    fn narrate(&mut self, case: &NarrationCase, out: &Path) -> Result<Vec<PathBuf>, String> {
        let recorded = self
            .inputs
            .narration(case.name)
            .ok_or_else(|| format!("no recorded texts for {}", case.name))?;
        let speech = self.speech.as_ref().ok_or("no Kokoro files")?;
        let mut narrator = Narrator::new(
            speech,
            parse::<Voice>(case.voice),
            case.speed,
            parse::<Language>(case.language),
            None,
        )
        .map_err(|error| error.to_string())?;
        let options = NarrationOptions {
            profile: if case.retail {
                Profile::Retail
            } else {
                Profile::Standard
            },
            heading_pause: case.heading_pause,
            closing_pause: case.closing_pause,
            sample_rate: case.sample_rate,
            label: recorded.label.as_ref().map(|label| Label {
                producer: label.producer.clone(),
                content_id: label.content_id.clone(),
            }),
        };
        // manuzio narrates in its work folder, `out/work`.
        let work = out.join("work");
        fs::create_dir_all(&work).map_err(|error| error.to_string())?;
        let mut files = Vec::new();
        for recorded in &recorded.texts {
            let mp3 = narrator
                .narrate(&self.tools, &text(recorded), &options, &work, None, |_| {})
                .map_err(|error| error.to_string())?;
            files.extend(mp3);
        }
        Ok(files)
    }

    fn trailer(
        &mut self,
        case: &TrailerCase,
        inputs: &Path,
        out: &Path,
    ) -> Result<PathBuf, String> {
        let trailer = Trailer {
            steps: case
                .steps
                .iter()
                .map(|step| Step {
                    image: Some(step.image.to_owned()),
                    text: step.text.to_owned(),
                    movement: match step.movement {
                        "zoomIn" => Movement::ZoomIn,
                        "zoomOut" => Movement::ZoomOut,
                        "leftToRight" => Movement::LeftToRight,
                        "rightToLeft" => Movement::RightToLeft,
                        other => panic!("unknown movement {other}"),
                    },
                })
                .collect(),
            voice: parse(case.voice),
            speed: case.speed,
            music_volume: case.music_volume,
            fade_seconds: case.fade_seconds,
            music_fade_in: case.music_fade_in,
            music_fade_out: case.music_fade_out,
        };
        let music = inputs.join("music.opus");
        let input = TrailerInput {
            trailer: &trailer,
            language: parse(case.language),
            images_dir: inputs,
            music: case.music.then_some(music.as_path()),
            out_dir: out,
            cancel: None,
        };
        let result = if case.model {
            let speech = self.speech.as_ref().ok_or("no Kokoro files")?;
            render_trailer(&self.tools, speech, &input, |_| {})
        } else {
            render_with_narrator(&self.tools, &input, tone, |_| {})
        };
        result.map_err(|error| error.to_string())
    }
}

/// Record every case through melies (only those without Kokoro unless `model`), and return what
/// differs from the committed recording.
fn run(model: bool) -> Vec<String> {
    let expected = recording();
    let work =
        Path::new(env!("CARGO_TARGET_TMPDIR")).join(if model { "parity-model" } else { "parity" });
    let mut session = Session::new(
        Path::new(env!("CARGO_BIN_EXE_melies-record")),
        &tools_dir(),
        &work,
    )
    .unwrap();
    let mut melies = Melies {
        tools: Tools::find_in([session.shim()]),
        speech: model.then(speech),
        inputs: Inputs::committed(),
    };
    record(&mut session, &mut melies, model).unwrap();
    let actual = session.finish("melies").unwrap();
    let differences = compare(&expected, &actual, model);
    if !differences.is_empty() {
        // For a closer look: the whole recording, beside the work folder.
        let file = work.with_extension("json");
        let recording = Recording {
            machine: Machine::current(),
            run: actual,
        };
        fs::write(&file, serde_json::to_string_pretty(&recording).unwrap()).unwrap();
        eprintln!("melies' recording: {}", file.display());
    }
    differences
}

#[test]
fn the_plans_are_those_manuzio_made() {
    let inputs = Inputs::committed();
    assert!(!inputs.plans.is_empty());
    let mut differences = Vec::new();
    for case in &inputs.plans {
        let options = NarrationOptions {
            heading_pause: case.heading_pause,
            closing_pause: case.closing_pause,
            ..NarrationOptions::default()
        };
        let planned: Vec<Segment> = plan(&text(&case.text), &options)
            .into_iter()
            .map(|segment| Segment {
                kind: kind_name(segment.kind).to_owned(),
                text: segment.text,
                padding: segment.padding,
            })
            .collect();
        if planned != case.segments {
            differences.push(format!(
                "{}: planned {planned:?}, expected {:?}",
                case.name, case.segments
            ));
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn the_trailers_are_those_manuzio_rendered() {
    let differences = run(false);
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
#[ignore = "tier:model narrates every case with Kokoro"]
fn the_narrations_and_the_kokoro_trailer_are_those_manuzio_made() {
    let machine = Machine::current();
    let recorded = recording().machine;
    if machine != recorded {
        eprintln!(
            "skipped: the Kokoro samples were recorded on {recorded:?}, and this is {machine:?} \
(kempelen's docs/parity.md, \"The machine\")"
        );
        return;
    }
    let differences = run(true);
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}
