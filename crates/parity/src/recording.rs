use crate::{
    NARRATIONS, PlanCase, RecordedNarration, TRAILERS, narration_scenario, trailer_scenario,
};
use kempelen_parity::Machine;
use kuleshov_parity::Reference;
use serde::{Deserialize, Serialize};
use std::{fs, io, path::Path};

/// The file in `references/` that holds the [`Inputs`]; the runs are in `<platform>.json`.
pub const INPUTS_FILE: &str = "inputs.json";

/// What manuzio read from its sample book for the cases, and planned: the same on every platform.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Inputs {
    /// What recorded them, such as "manuzio 0ec8e8e" (never compared).
    pub source: String,
    /// SHA-256 of the sample book the narration cases edit.
    pub sample_book_sha256: String,
    /// Each narration case's texts and label, in [`NARRATIONS`]'s order.
    pub narrations: Vec<RecordedNarration>,
    /// Plans: the sample book's first chapters, and every text of the narration cases.
    pub plans: Vec<PlanCase>,
}

impl Inputs {
    /// The committed inputs.
    ///
    /// # Panics
    /// When the committed file is not valid: a broken checkout.
    pub fn committed() -> Self {
        #[expect(clippy::expect_used, reason = "the committed file is valid")]
        serde_json::from_str(include_str!("../references/inputs.json"))
            .expect("references/inputs.json")
    }

    /// The recorded texts and label of the narration case `name`.
    pub fn narration(&self, name: &str) -> Option<&RecordedNarration> {
        self.narrations
            .iter()
            .find(|narration| narration.name == name)
    }
}

/// One platform's runs: every scenario as kuleshov's session saw it, and the machine the
/// Kokoro samples belong to.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Recording {
    /// Where the Kokoro scenarios were recorded: they hold only on this kind of machine
    /// (kempelen's docs/parity.md, "The machine").
    pub machine: Machine,
    pub run: Reference,
}

impl Recording {
    /// Read a recording written as JSON.
    ///
    /// # Errors
    /// When the file cannot be read or is not a recording.
    pub fn read(path: &Path) -> io::Result<Self> {
        serde_json::from_str(&fs::read_to_string(path)?).map_err(io::Error::other)
    }
}

/// The scenarios that need Kokoro: every narration, and the trailers it narrates.
pub fn model_scenarios() -> Vec<String> {
    NARRATIONS
        .iter()
        .map(narration_scenario)
        .chain(
            TRAILERS
                .iter()
                .filter(|case| case.model)
                .map(trailer_scenario),
        )
        .collect()
}

/// Everything in `actual` that differs from `expected`, one line each; empty when they agree.
///
/// Without `model`, the scenarios that need Kokoro are left out of `expected` (they were not run,
/// or not on the references' machine); the rest is kuleshov's comparison: the same calls, results,
/// outputs and leftover files (docs/parity.md).
pub fn compare(expected: &Recording, actual: &Reference, model: bool) -> Vec<String> {
    let mut expected = expected.run.clone();
    if !model {
        let skipped = model_scenarios();
        expected
            .scenarios
            .retain(|scenario| !skipped.contains(&scenario.name));
    }
    kuleshov_parity::compare(&expected, actual)
}
