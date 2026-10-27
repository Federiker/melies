//! The recording shim. Copied as `ffmpeg` and `ffprobe` into the folder the code under test takes
//! for its tools, it appends its arguments to `calls.jsonl` next to itself, then runs the real
//! tool named in `real-tools.txt` with the same arguments and standard streams, and exits with its
//! code, as kuleshov's `kuleshov-record` does.
//!
//! Besides the contents of list files, it records each existing WAV among the arguments, or named in
//! a list, by size and SHA-256, under the same `lists`: the narration's WAV is gone once the MP3 is
//! encoded, and must be the same bit for bit.

use kuleshov_parity::{CALLS_FILE, REAL_TOOLS_FILE, executable, hidden};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    env,
    fmt::Write as _,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::ExitCode,
};

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("melies-record: {error}");
            ExitCode::from(101)
        }
    }
}

fn run() -> Result<ExitCode, String> {
    let exe = env::current_exe().map_err(|error| error.to_string())?;
    let dir = exe.parent().ok_or("the shim has no folder")?;
    let tool = exe
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or("the shim has no name")?
        .to_owned();
    let real = fs::read_to_string(dir.join(REAL_TOOLS_FILE))
        .map_err(|error| format!("{REAL_TOOLS_FILE}: {error}"))?;
    let real = PathBuf::from(real.trim()).join(executable(&tool));
    let args: Vec<_> = env::args_os().skip(1).collect();

    let mut lists = BTreeMap::new();
    for arg in &args {
        let path = Path::new(arg);
        let extension = path
            .extension()
            .map(|extension| extension.to_string_lossy().to_ascii_lowercase());
        match extension.as_deref() {
            Some("txt" | "ffconcat") => {
                if let Ok(text) = fs::read_to_string(path) {
                    for file in listed_files(&text) {
                        record_wav(&mut lists, &file);
                    }
                    lists.insert(arg.to_string_lossy().into_owned(), text);
                }
            }
            Some("wav") => record_wav(&mut lists, &arg.to_string_lossy()),
            _ => {}
        }
    }
    let call = json!({
        "tool": tool,
        "args": args.iter().map(|arg| arg.to_string_lossy()).collect::<Vec<_>>(),
        "lists": lists,
    });
    fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join(CALLS_FILE))
        .and_then(|mut file| writeln!(file, "{call}"))
        .map_err(|error| format!("{CALLS_FILE}: {error}"))?;

    let status = hidden(&real)
        .args(&args)
        .status()
        .map_err(|error| format!("{}: {error}", real.display()))?;
    Ok(status
        .code()
        .and_then(|code| u8::try_from(code).ok())
        .map_or(ExitCode::FAILURE, ExitCode::from))
}

/// The files a concat list names (`file '…'`, with `'\''` for a quote, as kuleshov writes them).
fn listed_files(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| line.trim().strip_prefix("file '")?.strip_suffix('\''))
        .map(|path| path.replace("'\\''", "'"))
        .collect()
}

/// Record the WAV at `path`, if it is one and exists, by size and SHA-256.
fn record_wav(lists: &mut BTreeMap<String, String>, path: &str) {
    let is_wav = Path::new(path)
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("wav"));
    if is_wav && let Ok(bytes) = fs::read(path) {
        lists.insert(path.to_owned(), describe(&bytes));
    }
}

/// A file's size and SHA-256, as they are recorded.
fn describe(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes)
        .iter()
        .fold(String::with_capacity(64), |mut text, byte| {
            let _ = write!(text, "{byte:02x}");
            text
        });
    format!("{} bytes, SHA-256 {digest}", bytes.len())
}
