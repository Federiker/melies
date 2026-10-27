use crate::{
    Error, Result,
    error::Stop,
    narration::{NarrationOptions, NarrationProgress, Profile, Text, plan, retail},
};
use kempelen::{
    Language, Voice,
    assets::SpeechData,
    tts::{Device, LoudnessTarget, Synthesizer, WavWriter, silence_samples},
};
use kuleshov::{CombineOptions, Tools, combine_audio_files_with_options};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};

/// The WAV a narration is spoken into, and the synthesizer's scratch file, inside the output
/// folder; both are removed before [`Narrator::narrate`] returns. (The names are manuzio's, so
/// FFmpeg is given the same arguments.)
const WAV_NAME: &str = "chapter.wav";
const SCRATCH_NAME: &str = "segment.raw";

/// A loaded voice that narrates texts into MP3s, one after the other.
///
/// Loading the model is the slow part, so a whole book is best read with one `Narrator`.
pub struct Narrator {
    synthesizer: Synthesizer,
}

impl Narrator {
    /// Load `voice` from `data`, reading at `speed` (finite and above zero; manuzio's narration
    /// uses 0.9) and pronouncing `language`, whatever the voice's own accent.
    ///
    /// With `cuda_runtime` (see [`kempelen::cuda`]), the model runs on the GPU; when it cannot,
    /// narration falls back to the CPU with a warning in the log.
    pub fn new(
        data: &SpeechData,
        voice: Voice,
        speed: f32,
        language: Language,
        cuda_runtime: Option<&Path>,
    ) -> Result<Self> {
        let mut synthesizer = load_synthesizer(data, voice, speed, cuda_runtime)?;
        synthesizer.read_as(language)?;
        Ok(Self { synthesizer })
    }

    /// Hardware running the model.
    pub fn device(&self) -> Device {
        self.synthesizer.device()
    }

    /// Narrate `text` into an MP3 in `out_dir`, and return its path; `None` when the text has
    /// nothing to say, not even a title (see [`plan`]).
    ///
    /// Every segment is spoken into one WAV, followed by its silence, written as the speech comes so
    /// memory does not grow with the text, then encoded once with `tools` (the profile's encoding,
    /// and the label's tags). The WAV is `out_dir/chapter.wav`, so one narration at a time may use
    /// a folder. Cancellation is checked before every model chunk (a few seconds of speech) and
    /// stops FFmpeg; then the error is [`Error::Cancelled`] and no file is left.
    pub fn narrate(
        &mut self,
        tools: &Tools,
        text: &Text,
        options: &NarrationOptions,
        out_dir: &Path,
        cancel: Option<&AtomicBool>,
        mut progress: impl FnMut(NarrationProgress),
    ) -> Result<Option<PathBuf>> {
        let options = options.effective();
        self.synthesizer.set_loudness(Some(match options.profile {
            Profile::Standard => LoudnessTarget::AUDIOBOOK,
            Profile::Retail => LoudnessTarget::RETAIL,
        }));
        let wav = out_dir.join(WAV_NAME);
        let spoken = self.speak(text, &options, &wav, cancel, &mut progress);
        let result = spoken.and_then(|spoken| {
            if !spoken {
                return Ok(None);
            }
            check_cancelled(cancel)?;
            let mut combine = match options.profile {
                Profile::Standard => CombineOptions::default(),
                Profile::Retail => retail::combine_options(),
            };
            if let Some(label) = &options.label {
                combine.tags = label.tags();
            }
            let mp3 = combine_audio_files_with_options(
                tools,
                std::slice::from_ref(&wav),
                options.sample_rate,
                out_dir,
                &combine,
                cancel,
            )?;
            Ok(Some(mp3))
        });
        let _ = fs::remove_file(&wav);
        result
    }

    /// Speak every segment of `text` into `wav`, each followed by its silence; `false` (and no
    /// file) when there is nothing to say.
    ///
    /// The WAV is written as the speech comes, and cancellation is checked before every model
    /// chunk.
    fn speak(
        &mut self,
        text: &Text,
        options: &NarrationOptions,
        wav: &Path,
        cancel: Option<&AtomicBool>,
        progress: &mut impl FnMut(NarrationProgress),
    ) -> Result<bool> {
        let plan = plan(text, options);
        if plan.is_empty() {
            return Ok(false);
        }
        // Segments weigh by their text length, which follows their speaking time closely enough.
        let weights: Vec<usize> = plan
            .iter()
            .map(|segment| segment.text.chars().count())
            .collect();
        let of = weights.iter().sum::<usize>().max(1);
        let scratch = wav.with_file_name(SCRATCH_NAME);
        let mut writer = WavWriter::create(wav)?;
        let mut before = 0;
        for (number, segment) in plan.iter().enumerate() {
            check_cancelled(cancel)?;
            let weight = weights[number];
            self.synthesizer.synthesize_into(
                &segment.text,
                &scratch,
                &mut |samples| writer.write(samples),
                &mut |done, phonemes| {
                    if is_cancelled(cancel) {
                        return Err(kempelen::Error::interrupted(Stop));
                    }
                    progress(NarrationProgress::Speaking {
                        done: before + weight * done / phonemes.max(1),
                        of,
                    });
                    Ok(())
                },
            )?;
            check_cancelled(cancel)?;
            writer.silence(silence_samples(segment.padding))?;
            before += weight;
            progress(NarrationProgress::SegmentFinished {
                segment: number + 1,
                segments: plan.len(),
            });
        }
        writer.finish()?;
        Ok(true)
    }
}

/// Load the synthesizer on the GPU when a runtime is given, else on the CPU.
fn load_synthesizer(
    data: &SpeechData,
    voice: Voice,
    speed: f32,
    cuda_runtime: Option<&Path>,
) -> Result<Synthesizer> {
    if let Some(runtime) = cuda_runtime {
        match Synthesizer::with_cuda(data, voice, speed, runtime) {
            Ok(synthesizer) => return Ok(synthesizer),
            Err(error) => log::warn!("GPU narration unavailable, using the CPU: {error}"),
        }
    }
    Ok(Synthesizer::new(data, voice, speed)?)
}

fn is_cancelled(cancel: Option<&AtomicBool>) -> bool {
    cancel.is_some_and(|flag| flag.load(Ordering::Relaxed))
}

pub(crate) fn check_cancelled(cancel: Option<&AtomicBool>) -> Result<()> {
    if is_cancelled(cancel) {
        return Err(Error::Cancelled);
    }
    Ok(())
}
