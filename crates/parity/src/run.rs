use crate::{NARRATIONS, NarrationCase, TRAILERS, TrailerCase, generate_inputs};
use kuleshov_parity::{Outcome, Session};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

/// The first scenario: the generated trailer inputs (see [`generate_inputs`]).
pub const INPUTS: &str = "inputs";

/// What runs the cases: manuzio's crates in the harness, melies in the parity test.
pub trait Engine {
    /// Narrate `case` with what `out` (created, empty) may hold, and return its MP3 files in
    /// order, one per text that had something to say.
    ///
    /// # Errors
    /// The message of the failure, when the case fails.
    fn narrate(&mut self, case: &NarrationCase, out: &Path) -> Result<Vec<PathBuf>, String>;

    /// Render `case` from the images and music in `inputs` into `out` (created), and return the
    /// video.
    ///
    /// # Errors
    /// The message of the failure, when the case fails.
    fn trailer(&mut self, case: &TrailerCase, inputs: &Path, out: &Path)
    -> Result<PathBuf, String>;
}

/// The scenario that narrates `case`.
pub fn narration_scenario(case: &NarrationCase) -> String {
    format!("narration-{}", case.name)
}

/// The scenario that renders `case`.
pub fn trailer_scenario(case: &TrailerCase) -> String {
    format!("trailer-{}", case.name)
}

/// Run every case through `engine` in `session`, in order: the inputs, the narrations and the
/// trailers. Without `model`, the cases that need Kokoro are left out.
///
/// # Errors
/// When the session cannot record a scenario.
pub fn record(session: &mut Session, engine: &mut impl Engine, model: bool) -> io::Result<()> {
    let shim = session.shim().to_path_buf();
    session.run(INPUTS, |dir| match generate_inputs(&shim, dir) {
        Ok(outputs) => Outcome {
            value: Some(Ok(String::new())),
            outputs,
        },
        Err(error) => Outcome::error(&error),
    })?;
    let inputs = session.dir(INPUTS);
    if model {
        for case in NARRATIONS {
            session.run(&narration_scenario(case), |dir| narrate(engine, case, dir))?;
        }
    }
    for case in TRAILERS.iter().filter(|case| model || !case.model) {
        session.run(&trailer_scenario(case), |dir| {
            let out = dir.join("out");
            match fs::create_dir_all(&out) {
                Ok(()) => Outcome::file(engine.trailer(case, &inputs, &out)),
                Err(error) => Outcome::error(&error),
            }
        })?;
    }
    Ok(())
}

/// Narrate `case` in `dir/out`, then keep only its MP3s, as `dir/000.mp3`, `dir/001.mp3`…: what
/// else each side leaves there (manuzio's archive and parts folder) is not the narration's.
fn narrate(engine: &mut impl Engine, case: &NarrationCase, dir: &Path) -> Outcome {
    let out = dir.join("out");
    let result = fs::create_dir_all(&out)
        .map_err(|error| error.to_string())
        .and_then(|()| engine.narrate(case, &out))
        .and_then(|files| {
            files
                .iter()
                .enumerate()
                .map(|(index, file)| {
                    let copy = dir.join(format!("{index:03}.mp3"));
                    fs::copy(file, &copy)
                        .map(|_| copy)
                        .map_err(|error| format!("{}: {error}", file.display()))
                })
                .collect::<Result<Vec<_>, _>>()
        });
    let _ = fs::remove_dir_all(&out);
    match result {
        Ok(outputs) => Outcome {
            value: Some(Ok(String::new())),
            outputs,
        },
        Err(error) => Outcome::error(&error),
    }
}
