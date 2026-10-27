//! Stages what the tests read, downloading only what is missing or damaged:
//!
//! ```sh
//! cargo run --release -p melies-stage -- [--ffmpeg-only] [folder]   # mise run stage
//! ```
//!
//! - `ffmpeg` and `ffprobe` of the build kuleshov pins for this platform, in the folder (default
//!   `tools/`, which `MELIES_TOOLS_DIR` names), with kuleshov's `Tools::find_or_download`;
//! - the Kokoro model and voices kempelen pins (about 349 MB), in its `speech/` folder (which
//!   `MELIES_SPEECH_DIR` names), with kempelen's `assets::stage`; not with `--ffmpeg-only` (the
//!   light tier needs no model).
//!
//! Both check every file's SHA-256; nothing else touches the network.

use kuleshov::{Tool, Tools};
use std::{
    path::PathBuf,
    process::ExitCode,
    sync::atomic::{AtomicU64, Ordering},
};

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let ffmpeg_only = args.iter().any(|arg| arg == "--ffmpeg-only");
    let dir = args
        .iter()
        .find(|arg| *arg != "--ffmpeg-only")
        .map_or_else(|| PathBuf::from("tools"), PathBuf::from);
    let mut shown = 0;
    let progress = |done: u64, total: Option<u64>| {
        // A line every 10 MB, and the last one.
        if done >= shown + 10_000_000 || total == Some(done) {
            shown = done;
            match total {
                Some(total) => {
                    eprintln!("FFmpeg: {} of {} MB", done / 1_000_000, total / 1_000_000);
                }
                None => eprintln!("FFmpeg: {} MB", done / 1_000_000),
            }
        }
    };
    match Tools::find_or_download(&dir, progress, None) {
        Ok(tools) => {
            for tool in [Tool::Ffmpeg, Tool::Ffprobe] {
                println!("{tool}: {}", tools.path(tool).display());
            }
        }
        Err(error) => {
            eprintln!("FFmpeg: {error}");
            return ExitCode::FAILURE;
        }
    }
    if ffmpeg_only {
        return ExitCode::SUCCESS;
    }
    let speech = dir.join("speech");
    let shown = AtomicU64::new(0);
    let progress = |done: u64, total: u64| {
        // A line every 50 MB, and the last one.
        let last = shown.load(Ordering::Relaxed);
        if done >= last + 50_000_000 || (done == total && last != total) {
            shown.store(done, Ordering::Relaxed);
            eprintln!("Kokoro: {} of {} MB", done / 1_000_000, total / 1_000_000);
        }
    };
    let staged = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| error.to_string())
        .and_then(|runtime| {
            runtime
                .block_on(kempelen::assets::stage(&speech, progress))
                .map_err(|error| error.to_string())
        });
    match staged {
        Ok(()) => {
            println!("Kokoro: {}", speech.display());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("Kokoro: {error}");
            ExitCode::FAILURE
        }
    }
}
