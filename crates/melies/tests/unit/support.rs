//! What the tests share: the FFmpeg they run, the Kokoro files they read, and fresh folders.

use kempelen::assets::SpeechData;
use kuleshov::Tools;
use std::{
    fs,
    path::{Path, PathBuf},
};

/// The folder `variable` names, else `fallback` inside the repository.
fn folder(variable: &str, fallback: &str) -> PathBuf {
    // Relative to the workspace, as mise and a shell at its root give it.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    std::env::var_os(variable).map_or_else(|| root.join(fallback), |dir| root.join(dir))
}

/// The folder holding the `ffmpeg` and `ffprobe` the tests run: `MELIES_TOOLS_DIR`, else the
/// repository's `tools/` (`mise run stage`).
///
/// # Panics
/// When either program is missing there, naming the folder.
pub fn tools_dir() -> PathBuf {
    let dir = folder("MELIES_TOOLS_DIR", "tools");
    for tool in ["ffmpeg", "ffprobe"] {
        let file = dir.join(format!("{tool}{}", std::env::consts::EXE_SUFFIX));
        assert!(
            file.is_file(),
            "no {tool} in {}: set MELIES_TOOLS_DIR or run `mise run stage`",
            dir.display()
        );
    }
    dir
}

/// FFmpeg and FFprobe from [`tools_dir`].
pub fn tools() -> Tools {
    Tools::find_in([tools_dir()])
}

/// The staged Kokoro model and voices: `MELIES_SPEECH_DIR`, else the repository's `tools/speech`
/// (`mise run stage`).
///
/// # Panics
/// When the model is missing there, naming the folder.
#[allow(dead_code, reason = "the unit tests need no model")]
pub fn speech() -> SpeechData {
    let dir = folder("MELIES_SPEECH_DIR", "tools/speech");
    assert!(
        dir.join("models").is_dir(),
        "no Kokoro model in {}: set MELIES_SPEECH_DIR or run `mise run stage`",
        dir.display()
    );
    SpeechData::new(dir)
}

/// A new empty folder, removed when dropped.
pub struct Folder(PathBuf);

impl Folder {
    pub fn new(name: &str) -> Self {
        let dir =
            std::env::temp_dir().join(format!("melies-{name}-{}", uuid::Uuid::new_v4().simple()));
        fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }
}

impl std::ops::Deref for Folder {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.0
    }
}

impl Drop for Folder {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
