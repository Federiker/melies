//! Repeatable timings of narration and trailers, as manuzio's `mnz-bench` measured them
//! (docs/performance.md):
//!
//! ```sh
//! cargo run --release -p melies --example bench -- audiobook [--cuda RUNTIME]
//! cargo run --release -p melies --example bench -- trailer [--images DIR --music FILE]
//! ```
//!
//! `audiobook` narrates the Introduction and Chapters I and II of manuzio's sample book (the texts
//! manuzio read, `melies_parity::Inputs`) with `bf_emma` at 0.9 into MP3s; `trailer` renders three
//! scenes with music and fades, from manuzio's `01.jpg`, `02.jpg` and a music file when given
//! (`--images ../manuzio/assets/samples --music ../manuzio/assets/audios/fiction_hopeful.opus`),
//! else from the generated parity inputs. Each prints the time of every stage, the peak memory
//! (Windows) and the SHA-256 of each file and of its decoded audio. Not a test: it narrates
//! minutes of audio. The tools and the Kokoro files are those of the tests (`MELIES_TOOLS_DIR`,
//! `MELIES_SPEECH_DIR`); work files go to `tmp/bench`.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_precision_loss,
    reason = "a benchmark: a failure is a panic"
)]

use kempelen::{Language, Voice, assets::SpeechData};
use kuleshov::{Tool, Tools, audio_duration};
use melies::{
    narration::{NarrationOptions, NarrationProgress, Narrator, Text},
    trailer::{
        BASE_SPEED, Movement, Step, Trailer, TrailerInput, TrailerProgress, render_with_narrator,
    },
};
use std::{
    fmt::Write as _,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

const VOICE: Voice = Voice::BfEmma;
const LANGUAGE: Language = Language::English;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn folder(variable: &str, fallback: &str) -> PathBuf {
    std::env::var_os(variable).map_or_else(|| root().join(fallback), |dir| root().join(dir))
}

/// A clean work folder under `tmp/bench`, which is gitignored.
fn work_dir(name: &str) -> PathBuf {
    let dir = std::path::absolute(root().join("tmp").join("bench").join(name)).unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Named stage times, printed in the order they were recorded.
#[derive(Default)]
struct Stages(Vec<(String, Duration)>);

impl Stages {
    fn add(&mut self, name: &str, time: Duration) {
        match self.0.iter_mut().find(|(known, _)| known == name) {
            Some((_, total)) => *total += time,
            None => self.0.push((name.to_owned(), time)),
        }
    }

    fn print(&self) {
        for (name, time) in &self.0 {
            println!("  {name:<28} {:>9.2} s", time.as_secs_f64());
        }
    }
}

/// Highest memory the process has used so far (peak working set on Windows), in MB.
#[cfg(windows)]
#[expect(
    unsafe_code,
    non_snake_case,
    reason = "the one Windows call that reports it"
)]
fn peak_memory_mb() -> Option<f64> {
    #[repr(C)]
    #[derive(Default)]
    struct Counters {
        cb: u32,
        PageFaultCount: u32,
        PeakWorkingSetSize: usize,
        WorkingSetSize: usize,
        QuotaPeakPagedPoolUsage: usize,
        QuotaPagedPoolUsage: usize,
        QuotaPeakNonPagedPoolUsage: usize,
        QuotaNonPagedPoolUsage: usize,
        PagefileUsage: usize,
        PeakPagefileUsage: usize,
    }
    unsafe extern "system" {
        fn GetCurrentProcess() -> isize;
        fn K32GetProcessMemoryInfo(process: isize, counters: *mut Counters, cb: u32) -> i32;
    }
    let mut counters = Counters {
        cb: u32::try_from(size_of::<Counters>()).ok()?,
        ..Default::default()
    };
    // SAFETY: the pseudo-handle of the current process needs no closing, and `counters` is a
    // properly sized, writable structure whose size is passed alongside it.
    let ok =
        unsafe { K32GetProcessMemoryInfo(GetCurrentProcess(), &raw mut counters, counters.cb) };
    (ok != 0).then(|| counters.PeakWorkingSetSize as f64 / 1_048_576.0)
}

#[cfg(not(windows))]
fn peak_memory_mb() -> Option<f64> {
    None
}

fn print_peak_memory() {
    match peak_memory_mb() {
        Some(mb) => println!("  {:<28} {mb:>9.0} MB", "peak memory"),
        None => println!("  peak memory: not measured on this platform"),
    }
}

fn hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes)
        .iter()
        .fold(String::new(), |mut text, byte| {
            let _ = write!(text, "{byte:02x}");
            text
        })
}

/// Print the SHA-256 of `file` and of its decoded audio (and frames, for a video).
fn report_file(tools: &Tools, file: &Path, name: &str, video: bool) {
    let decoded = |args: &[&str]| {
        let output = tools
            .command(Tool::Ffmpeg)
            .args(["-v", "error", "-i"])
            .arg(file)
            .args(args)
            .output()
            .unwrap();
        hex(&output.stdout)
    };
    println!("  {name}");
    println!("    file    {}", hex(&std::fs::read(file).unwrap()));
    println!(
        "    audio   {}",
        decoded(&["-map", "0:a:0", "-f", "s16le", "-"])
    );
    if video {
        println!(
            "    frames  {}",
            decoded(&["-map", "0:v:0", "-f", "framemd5", "-"])
        );
    }
}

fn audiobook(tools: &Tools, speech: &SpeechData, cuda: Option<&Path>) {
    let out = work_dir("audiobook");
    let inputs = melies_parity::Inputs::committed();
    let texts: Vec<Text> = ["oz-0", "oz-1", "oz-2"]
        .iter()
        .map(|name| {
            let text = &inputs
                .plans
                .iter()
                .find(|plan| plan.name == *name)
                .unwrap()
                .text;
            Text {
                notice: None,
                title: text.title.clone(),
                subtitle: text.subtitle.clone(),
                body: text.body.clone(),
            }
        })
        .collect();
    println!(
        "audiobook: oz.epub, {} chapters, {VOICE:?}, {}",
        texts.len(),
        if cuda.is_some() { "CUDA" } else { "CPU" }
    );
    let mut stages = Stages::default();
    let start = Instant::now();
    let mut narrator = Narrator::new(speech, VOICE, 0.9, LANGUAGE, cuda).unwrap();
    stages.add("model load", start.elapsed());
    let (mut segments, mut files) = (0, Vec::new());
    for text in &texts {
        let mut mark = Instant::now();
        let mp3 = narrator
            .narrate(
                tools,
                text,
                &NarrationOptions::default(),
                &out,
                None,
                |event| {
                    if let NarrationProgress::SegmentFinished { .. } = event {
                        segments += 1;
                        stages.add("speech and padding", mark.elapsed());
                        mark = Instant::now();
                    }
                },
            )
            .unwrap()
            .unwrap();
        stages.add("chapter encode", mark.elapsed());
        files.push(mp3);
    }
    let wall = start.elapsed().as_secs_f64();
    stages.print();
    println!("  {:<28} {wall:>9.2} s", "total");
    print_peak_memory();
    println!("  segments                     {segments:>9}");
    let mut audio = 0.0;
    for (index, file) in files.iter().enumerate() {
        audio += audio_duration(tools, file).unwrap();
        report_file(tools, file, &format!("chapter {index}"), false);
    }
    println!(
        "  audio                        {audio:>9.1} s ({:.2}x real time)",
        audio / wall
    );
}

fn trailer(tools: &Tools, speech: &SpeechData, images: Option<PathBuf>, music: Option<PathBuf>) {
    let out = work_dir("trailer");
    let (images, names, music) = if let Some(images) = images {
        (images, ["01.jpg", "02.jpg"], music)
    } else {
        let generated = out.join("inputs");
        melies_parity::generate_inputs(&folder("MELIES_TOOLS_DIR", "tools"), &generated).unwrap();
        let music = music.unwrap_or_else(|| generated.join("music.opus"));
        (generated, ["portrait.jpg", "landscape.jpg"], Some(music))
    };
    let images = std::path::absolute(images).unwrap();
    let scenes = [
        (
            names[0],
            Movement::ZoomIn,
            "A girl from Kansas is swept away by a cyclone to a land of wonders and witches.",
        ),
        (
            names[1],
            Movement::LeftToRight,
            "On the yellow brick road she meets a scarecrow, a tin woodman and a cowardly lion.",
        ),
        (
            names[0],
            Movement::ZoomOut,
            "Together they must reach the Emerald City, and find the way home again.",
        ),
    ];
    let trailer = Trailer {
        steps: scenes
            .iter()
            .map(|(image, movement, text)| Step {
                image: Some((*image).into()),
                text: (*text).into(),
                movement: *movement,
            })
            .collect(),
        voice: VOICE,
        fade_seconds: 0.5,
        music_fade_in: 0.1,
        music_fade_out: 0.1,
        ..Default::default()
    };
    let input = TrailerInput {
        trailer: &trailer,
        language: LANGUAGE,
        images_dir: &images,
        music: music.as_deref(),
        out_dir: &out,
        cancel: None,
    };
    println!(
        "trailer: {} scenes with music and fades, {VOICE:?}, CPU, {}",
        scenes.len(),
        images.display()
    );
    let mut stages = Stages::default();
    let start = Instant::now();
    let mut synthesizer =
        kempelen::tts::Synthesizer::new(speech, VOICE, BASE_SPEED * trailer.speed).unwrap();
    synthesizer.read_as(LANGUAGE).unwrap();
    stages.add("model load", start.elapsed());
    let mut mark = Instant::now();
    let mut narration = Duration::ZERO;
    let video = render_with_narrator(
        tools,
        &input,
        |text, wav| {
            let begun = Instant::now();
            let result = synthesizer.synthesize_to_wav(text, wav);
            narration += begun.elapsed();
            result
        },
        |event| {
            let now = Instant::now();
            let name = match event {
                TrailerProgress::SceneStarted { .. } | TrailerProgress::MixingAudio => None,
                TrailerProgress::SceneFinished { .. } => Some("scenes (narration and motion)"),
                TrailerProgress::Stitching => Some("narration and music mix"),
                TrailerProgress::Muxing => Some("stitching (fades)"),
                TrailerProgress::Finished => Some("muxing"),
            };
            if let Some(name) = name {
                stages.add(name, now - mark);
            }
            mark = now;
        },
    )
    .unwrap();
    let wall = start.elapsed().as_secs_f64();
    stages.add("  of which narration", narration);
    stages.print();
    println!("  {:<28} {wall:>9.2} s", "total");
    print_peak_memory();
    report_file(tools, &video, "trailer.mp4", true);
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let value = |flag: &str| {
        args.iter()
            .position(|arg| arg == flag)
            .map(|at| PathBuf::from(&args[at + 1]))
    };
    let tools = Tools::find_in([folder("MELIES_TOOLS_DIR", "tools")]);
    let speech = SpeechData::new(folder("MELIES_SPEECH_DIR", "tools/speech"));
    match args.first().map(String::as_str) {
        Some("audiobook") => audiobook(&tools, &speech, value("--cuda").as_deref()),
        Some("trailer") => trailer(&tools, &speech, value("--images"), value("--music")),
        _ => eprintln!(
            "usage: bench audiobook [--cuda RUNTIME] | trailer [--images DIR --music FILE]"
        ),
    }
}
