use serde::{Deserialize, Serialize};

/// One edit of the sample book that a narration case reads (see [`crate::write_book`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Book {
    /// The Introduction and Chapter I, each cut to its first paragraph.
    Short,
    /// The short Introduction with a scene break (`* * *`) and one more sentence, Chapter I of a
    /// lone ornament (`⁂`) and Chapter II with no text at all.
    Specials,
    /// The Introduction and Chapter I, each a paragraph of Chinese, Chapter I headed 第一章 龙卷风.
    Chinese,
}

/// One audiobook run through manuzio, and the same texts narrated by melies.
///
/// Voices and languages are their serialised names (`af_alloy`, `en`), which manuzio's and
/// kempelen's types share.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NarrationCase {
    pub name: &'static str,
    pub book: Book,
    /// How many chapters of the book are narrated (manuzio's `max_chapters`).
    pub chapters: usize,
    pub voice: &'static str,
    pub language: &'static str,
    pub speed: f32,
    /// The retail profile instead of the standard one.
    pub retail: bool,
    /// Silence after the heading and subtitle (manuzio's `pad_chapter`).
    pub heading_pause: f64,
    /// Silence after the body and at a scene break (manuzio's `pad_between_chapters`).
    pub closing_pause: f64,
    /// Sample rate of the MP3 (manuzio's `output_rate`).
    pub sample_rate: u32,
    /// Label the narration as AI: a spoken notice before the first text, tags in every file.
    pub label: bool,
}

/// The narration cases, in the order they run.
pub const NARRATIONS: &[NarrationCase] = &[
    NarrationCase {
        name: "standard",
        book: Book::Short,
        chapters: 2,
        voice: "af_alloy",
        language: "en",
        speed: 0.9,
        retail: false,
        heading_pause: 0.5,
        closing_pause: 1.5,
        sample_rate: 44_100,
        label: false,
    },
    // The retail profile fixes the sample rate and keeps the closing pause within 1 to 5 seconds.
    NarrationCase {
        name: "retail",
        book: Book::Short,
        chapters: 2,
        voice: "af_alloy",
        language: "en",
        speed: 0.9,
        retail: true,
        heading_pause: 0.5,
        closing_pause: 7.0,
        sample_rate: 22_050,
        label: false,
    },
    NarrationCase {
        name: "labelled",
        book: Book::Short,
        chapters: 2,
        voice: "bf_emma",
        language: "en",
        speed: 1.0,
        retail: false,
        heading_pause: 0.8,
        closing_pause: 2.0,
        sample_rate: 48_000,
        label: true,
    },
    NarrationCase {
        name: "ornaments",
        book: Book::Specials,
        chapters: 3,
        voice: "af_alloy",
        language: "en",
        speed: 0.9,
        retail: false,
        heading_pause: 0.5,
        closing_pause: 1.5,
        sample_rate: 44_100,
        label: false,
    },
    NarrationCase {
        name: "chinese",
        book: Book::Chinese,
        chapters: 2,
        voice: "zf_xiaoxiao",
        language: "zh",
        speed: 0.9,
        retail: false,
        heading_pause: 0.5,
        closing_pause: 1.5,
        sample_rate: 44_100,
        label: false,
    },
];

/// One text to narrate, as manuzio hands a chapter to melies: the spoken heading it builds
/// ("Chapter 1", "第1章", or the title), the subtitle, the body with a line break after every
/// paragraph, and the notice before them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Text {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notice: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    pub body: String,
}

/// The AI label's tags: who made the audio, and the identifier of the work.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Label {
    pub producer: String,
    pub content_id: String,
}

/// What manuzio narrated for a case: its texts in order, and its label.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordedNarration {
    pub name: String,
    pub texts: Vec<Text>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<Label>,
}

/// One spoken piece of a plan: `notice`, `heading`, `subtitle` or `body`, its text and the silence
/// after it, in seconds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Segment {
    pub kind: String,
    pub text: String,
    pub padding: f64,
}

/// A text, the pauses it was planned with, and manuzio's plan of it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanCase {
    pub name: String,
    pub heading_pause: f64,
    pub closing_pause: f64,
    pub text: Text,
    pub segments: Vec<Segment>,
}

/// One trailer scene: an image of the generated inputs (see [`crate::generate_inputs`]), its text
/// and its movement (`zoomIn`, `zoomOut`, `leftToRight`, `rightToLeft`, as manuzio saves them).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrailerStep {
    pub image: &'static str,
    pub text: &'static str,
    pub movement: &'static str,
}

/// One trailer render, through manuzio and through melies.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrailerCase {
    pub name: &'static str,
    /// Narrated by Kokoro; otherwise by [`crate::tone`], which needs no model.
    pub model: bool,
    pub voice: &'static str,
    pub language: &'static str,
    pub speed: f32,
    pub steps: &'static [TrailerStep],
    /// With the generated music.
    pub music: bool,
    pub music_volume: f32,
    pub fade_seconds: f32,
    pub music_fade_in: f32,
    pub music_fade_out: f32,
}

/// The step texts, 40 to 200 characters each.
pub const STEP_TEXTS: [&str; 4] = [
    "A girl from Kansas is swept away to a land of wonders.",
    "She follows the yellow brick road with a scarecrow, a tin woodman and a cowardly lion.",
    "Only the great wizard of the Emerald City can send her home again.",
    "Witches good and wicked, flying monkeys and a city all of green stand between Dorothy and \
the little farmhouse where Aunt Em is waiting for her.",
];

const fn step(image: &'static str, text: usize, movement: &'static str) -> TrailerStep {
    TrailerStep {
        image,
        text: STEP_TEXTS[text],
        movement,
    }
}

const PORTRAIT: &str = "portrait.jpg";
const LANDSCAPE: &str = "landscape.jpg";

/// The trailer cases, in the order they run.
pub const TRAILERS: &[TrailerCase] = &[
    TrailerCase {
        name: "single",
        model: false,
        voice: "af_heart",
        language: "en",
        speed: 1.0,
        steps: &[step(PORTRAIT, 0, "zoomIn")],
        music: false,
        music_volume: 0.3,
        fade_seconds: 0.0,
        music_fade_in: 0.0,
        music_fade_out: 0.0,
    },
    TrailerCase {
        name: "music-fades",
        model: false,
        voice: "af_heart",
        language: "en",
        speed: 1.0,
        steps: &[
            step(PORTRAIT, 0, "zoomIn"),
            step(LANDSCAPE, 1, "leftToRight"),
            step(PORTRAIT, 2, "zoomOut"),
        ],
        music: true,
        music_volume: 0.3,
        fade_seconds: 0.5,
        music_fade_in: 0.1,
        music_fade_out: 0.2,
    },
    TrailerCase {
        name: "scenes",
        model: false,
        voice: "af_heart",
        language: "en",
        speed: 1.0,
        steps: &[
            step(LANDSCAPE, 1, "rightToLeft"),
            step(PORTRAIT, 3, "zoomOut"),
            step(LANDSCAPE, 2, "zoomIn"),
        ],
        music: false,
        music_volume: 0.3,
        fade_seconds: 0.0,
        music_fade_in: 0.0,
        music_fade_out: 0.0,
    },
    TrailerCase {
        name: "music",
        model: false,
        voice: "af_heart",
        language: "en",
        speed: 1.0,
        steps: &[
            step(LANDSCAPE, 0, "zoomOut"),
            step(PORTRAIT, 2, "leftToRight"),
        ],
        music: true,
        music_volume: 0.6,
        fade_seconds: 0.0,
        music_fade_in: 0.0,
        music_fade_out: 0.0,
    },
    TrailerCase {
        name: "kokoro",
        model: true,
        voice: "af_alloy",
        language: "en",
        speed: 1.1,
        steps: &[step(PORTRAIT, 0, "zoomIn")],
        music: true,
        music_volume: 0.25,
        fade_seconds: 0.0,
        music_fade_in: 0.2,
        music_fade_out: 0.3,
    },
];
