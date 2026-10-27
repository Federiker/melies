//! Marking narration as AI-generated.
//!
//! The option does two things: the narration opens with a spoken notice (the caller's sentence,
//! [`crate::narration::Text::notice`]), and the MP3 carries tags that say the voice is synthetic.
//! Some audiobook stores accept human narration only and some jurisdictions require synthetic
//! audio to be labelled; the tags make the origin machine-readable without claiming compliance
//! with any particular rule.

/// The voice model behind the narration.
pub const VOICE_MODEL: &str = "Kokoro-82M";

/// Who made a labelled narration, and which work it belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label {
    /// The program that made it, such as "manuzio".
    pub producer: String,
    /// Identifies the work (a book); the same for each of its files, and on a rerun.
    pub content_id: String,
}

impl Label {
    /// ID3 tags for a labelled MP3, as `(name, value)` pairs for FFmpeg's `-metadata`.
    ///
    /// `comment` is the human-readable line. `AIGC` is a small JSON object (label, producer and a
    /// content identifier) in the style of the provenance field that China's labelling standard
    /// describes; its exact field set has not been checked against the standard text. The producer
    /// and the identifier are written as they are given, so they hold no quotation marks.
    pub fn tags(&self) -> Vec<(String, String)> {
        let Self {
            producer,
            content_id,
        } = self;
        vec![
            (
                "comment".into(),
                format!(
                    "AI-generated narration: synthetic voice ({VOICE_MODEL}), made with {producer}"
                ),
            ),
            (
                "AIGC".into(),
                format!(
                    r#"{{"Label":"1","ContentProducer":"{producer}","ProduceID":"{content_id}"}}"#
                ),
            ),
        ]
    }
}

#[cfg(test)]
#[path = "../../tests/unit/narration/label.rs"]
mod tests;
