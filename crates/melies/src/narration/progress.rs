/// Progress reported while a text is narrated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NarrationProgress {
    /// Narration moved on; reported before every model chunk (a few seconds of speech), so
    /// progress moves within a long body too. `done` of `of` is the share of the text's speech
    /// synthesized so far: segments weigh by their text length, and the current one by its
    /// phonemes done.
    Speaking { done: usize, of: usize },
    /// A spoken piece (see [`crate::narration::Segment`]) and its silence are done.
    SegmentFinished { segment: usize, segments: usize },
}
