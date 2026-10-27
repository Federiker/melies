#[derive(Debug, Clone, PartialEq, Eq)]
/// Rendering milestones, in the order they happen.
pub enum TrailerProgress {
    SceneStarted { index: usize, total: usize },
    SceneFinished { index: usize, total: usize },
    MixingAudio,
    Stitching,
    Muxing,
    Finished,
}
