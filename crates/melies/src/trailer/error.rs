use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
/// A trailer setting that prevents rendering. Step numbers start at 1.
pub enum TrailerError {
    NoSteps,
    MissingImage { step: usize },
    UnsafeImageName { step: usize },
    ImageNotFound { step: usize },
    TextTooShort { step: usize, min: usize },
    TextTooLong { step: usize, max: usize },
    MusicNotFound,
    InvalidSetting(&'static str),
}

impl fmt::Display for TrailerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoSteps => write!(f, "The trailer needs at least one step"),
            Self::MissingImage { step } => write!(f, "Step {step} has no image"),
            Self::UnsafeImageName { step } => write!(f, "Step {step} has an invalid image name"),
            Self::ImageNotFound { step } => write!(f, "The image of step {step} was not found"),
            Self::TextTooShort { step, min } => {
                write!(f, "Step {step} needs at least {min} characters of text")
            }
            Self::TextTooLong { step, max } => {
                write!(f, "Step {step} has more than {max} characters of text")
            }
            Self::MusicNotFound => write!(f, "The selected music file was not found"),
            Self::InvalidSetting(what) => write!(f, "Invalid trailer setting: {what}"),
        }
    }
}

impl std::error::Error for TrailerError {}
