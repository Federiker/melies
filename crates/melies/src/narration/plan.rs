use crate::narration::NarrationOptions;

/// Bump when what melies makes of a text changes what is heard: the plan, the pauses or the
/// encode. A caller that keeps finished narrations (manuzio's resume) puts it in its fingerprint,
/// so narrations made the old way are not reused.
///
/// Revision 1: ornaments are no longer read and a scene break became a pause (manuzio's
/// `PLAN_REVISION` 1).
pub const REVISION: u32 = 1;

/// What to narrate: the notice that the voice is synthetic, the title and the subtitle (each
/// followed by the heading pause) and the body (followed by the closing pause).
///
/// The title is spoken as it is given: a caller builds it ("Chapter 3", "第3章", or the chapter's
/// name). The body has a line break after every paragraph, which the synthesizer treats as a
/// passage; a paragraph with no letter or digit is an ornament (see [`is_speakable`]).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Text {
    pub notice: Option<String>,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub body: String,
}

/// Which part of a [`Text`] a [`Segment`] speaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentKind {
    /// The spoken notice that the voice is synthetic (the "label as AI" option).
    Notice,
    Title,
    Subtitle,
    Body,
}

/// One spoken piece of a text and the silence that follows it.
#[derive(Debug, Clone, PartialEq)]
pub struct Segment {
    pub kind: SegmentKind,
    pub text: String,
    /// Silence appended after the speech, in seconds.
    pub padding: f64,
}

/// Silence after the notice that announces a synthetic voice, in seconds.
pub const NOTICE_PADDING: f64 = 1.0;

/// The notice piece that opens a narration read by a synthetic voice, or `None` for blank text.
pub fn notice_segment(text: &str) -> Option<Segment> {
    let text = text.trim();
    (!text.is_empty()).then(|| Segment {
        kind: SegmentKind::Notice,
        text: text.to_string(),
        padding: NOTICE_PADDING,
    })
}

/// Whether `text` has anything to say: a letter or a digit, in any script.
///
/// Lines without one (`* * *`, `⁂`, `~ ~ ~`, `———`) are ornaments: a phonemizer would read them as
/// "asterisk asterisk asterisk", or produce no sound at all.
pub fn is_speakable(text: &str) -> bool {
    text.chars().any(char::is_alphanumeric)
}

/// Split a text into the pieces the narration speaks, in order, with the pauses of `options` (as
/// its profile has them: see [`NarrationOptions`]).
///
/// The notice comes first, then the title and the subtitle, then the body. Pieces without
/// speakable text are left out. Ornament paragraphs are never read: one between paragraphs (a
/// scene break) ends a body piece, so it is heard as a pause as long as the closing one.
pub fn plan(text: &Text, options: &NarrationOptions) -> Vec<Segment> {
    let options = options.effective();
    let mut plan: Vec<Segment> = text
        .notice
        .as_deref()
        .and_then(notice_segment)
        .into_iter()
        .collect();
    let headings = [
        (SegmentKind::Title, text.title.clone()),
        (SegmentKind::Subtitle, text.subtitle.clone()),
    ];
    plan.extend(headings.into_iter().filter_map(|(kind, text)| {
        let text = text?.trim().to_string();
        is_speakable(&text).then_some(Segment {
            kind,
            text,
            padding: options.heading_pause,
        })
    }));
    plan.extend(scenes(&text.body).into_iter().map(|text| Segment {
        kind: SegmentKind::Body,
        text,
        padding: options.closing_pause,
    }));
    plan
}

/// The text cut at its ornament paragraphs, without them and without blank lines.
fn scenes(text: &str) -> Vec<String> {
    let mut scenes = Vec::new();
    let mut scene: Vec<&str> = Vec::new();
    for line in text.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if is_speakable(line) {
            scene.push(line);
        } else if !scene.is_empty() {
            scenes.push(scene.join("\n"));
            scene.clear();
        }
    }
    if !scene.is_empty() {
        scenes.push(scene.join("\n"));
    }
    scenes
}

#[cfg(test)]
#[path = "../../tests/unit/narration/plan.rs"]
mod tests;
