use crate::narration::{Label, retail};

/// How the MP3 is delivered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Profile {
    /// A small variable-bitrate MP3.
    #[default]
    Standard,
    /// A file that follows the technical rules audiobook stores publish; see [`crate::narration::retail`].
    Retail,
}

/// How a text is narrated (see [`crate::narration::Narrator::narrate`]); the voice, the speed and
/// the language belong to the [`crate::narration::Narrator`].
#[derive(Debug, Clone, PartialEq)]
pub struct NarrationOptions {
    /// Delivery format of the MP3.
    pub profile: Profile,
    /// Silence after the title and after the subtitle, in seconds.
    pub heading_pause: f64,
    /// Silence after the body, and at each scene break inside it, in seconds.
    pub closing_pause: f64,
    /// Sample rate of the MP3.
    pub sample_rate: u32,
    /// Tag the MP3 as AI-generated narration (see [`crate::narration::label`]); the spoken notice
    /// is the text's own ([`crate::narration::Text::notice`]).
    pub label: Option<Label>,
}

impl NarrationOptions {
    /// These options with the changes the profile requires: the retail profile fixes the sample
    /// rate and keeps the closing pause within the stores' limits.
    pub(crate) fn effective(&self) -> Self {
        let mut options = self.clone();
        if self.profile == Profile::Retail {
            options.sample_rate = retail::SAMPLE_RATE;
            options.closing_pause = options
                .closing_pause
                .clamp(retail::MIN_PAD_SECS, retail::MAX_PAD_SECS);
        }
        options
    }
}

impl Default for NarrationOptions {
    fn default() -> Self {
        Self {
            profile: Profile::Standard,
            heading_pause: 0.5,
            closing_pause: 1.5,
            sample_rate: 44_100,
            label: None,
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/narration/options.rs"]
mod tests;
