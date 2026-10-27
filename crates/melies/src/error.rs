use crate::trailer::TrailerError;
use std::{
    fmt, io,
    path::{Path, PathBuf},
};

/// Everything that can go wrong in melies (docs/errors.md). Each error names the file or the
/// trailer step it is about, where there is one.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The caller asked to stop: FFmpeg was stopped, or the next model chunk or stage never
    /// started, and what had been written was removed.
    #[error("cancelled")]
    Cancelled,
    /// A trailer setting prevents rendering; nothing ran.
    #[error(transparent)]
    Trailer(#[from] TrailerError),
    /// FFmpeg or FFprobe failed.
    #[error(transparent)]
    Media(kuleshov::Error),
    /// The speech model failed, or could not be loaded.
    #[error(transparent)]
    Speech(kempelen::Error),
    /// The narrator of a trailer step (numbered from 1) failed.
    #[error("Failed to narrate step {step}")]
    Narration {
        step: usize,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    /// The image of a trailer step (numbered from 1) could not be animated.
    #[error("Failed to animate the image of step {step}")]
    Animation {
        step: usize,
        #[source]
        source: kuleshov::Error,
    },
    /// FFmpeg could not mix the narration with the `music` file.
    #[error("ffmpeg failed to mix the music")]
    Music {
        music: PathBuf,
        #[source]
        source: kuleshov::Error,
    },
    /// A file or folder could not be created, written or moved.
    #[error("Failed to {action} {}", path.display())]
    File {
        /// What was being done: `create`, `write`.
        action: &'static str,
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

/// The result of every fallible function here.
pub type Result<T, E = Error> = std::result::Result<T, E>;

impl Error {
    /// Whether the caller cancelled the work.
    pub fn is_cancelled(&self) -> bool {
        matches!(self, Self::Cancelled)
    }

    /// The file the error is about, if any.
    pub fn file(&self) -> Option<&Path> {
        match self {
            Self::Media(error) | Self::Animation { source: error, .. } => error.file(),
            Self::Music { music, .. } => Some(music),
            Self::File { path, .. } => Some(path),
            _ => None,
        }
    }

    pub(crate) fn file_error(
        action: &'static str,
        path: impl Into<PathBuf>,
    ) -> impl FnOnce(io::Error) -> Self {
        let path = path.into();
        move |source| Self::File {
            action,
            path,
            source,
        }
    }
}

impl From<kuleshov::Error> for Error {
    fn from(error: kuleshov::Error) -> Self {
        if error.is_cancelled() {
            Self::Cancelled
        } else {
            Self::Media(error)
        }
    }
}

impl From<kempelen::Error> for Error {
    fn from(error: kempelen::Error) -> Self {
        match error {
            kempelen::Error::Interrupted(inner) if inner.is::<Stop>() => Self::Cancelled,
            error => Self::Speech(error),
        }
    }
}

/// What a synthesis callback stops with when the caller cancels; it becomes [`Error::Cancelled`].
#[derive(Debug)]
pub(crate) struct Stop;

impl fmt::Display for Stop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("cancelled")
    }
}

impl std::error::Error for Stop {}

#[cfg(test)]
#[path = "../tests/unit/error.rs"]
mod tests;
