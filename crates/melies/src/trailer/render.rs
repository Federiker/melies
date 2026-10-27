use crate::{
    Error, Result,
    narration::check_cancelled,
    trailer::{
        OUTPUT_NAME, TrailerInput, TrailerProgress, WORK_DIR,
        mix::{MusicMix, mix_music_with_cancel},
        scene::{Narrate, Scene, render_scene},
        timing::{AUDIO_RATE, BASE_SPEED, FPS, HEIGHT, WIDTH, share_of},
        validate,
    },
};
use kempelen::{assets::SpeechData, tts::Synthesizer};
use kuleshov::{
    ClipFades, Tools, combine_video_files_with_cancel, concat_matching_video_files_with_cancel,
    concat_wav_with_cancel, mux_audio_with_cancel,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Render the trailer with a Kokoro voice from `data` and return the video path.
///
/// The voice is the trailer's, at [`BASE_SPEED`] times the trailer's speed, pronouncing
/// `input.language`; the model runs on the CPU.
pub fn render_trailer(
    tools: &Tools,
    data: &SpeechData,
    input: &TrailerInput<'_>,
    progress: impl FnMut(TrailerProgress),
) -> Result<PathBuf> {
    validate(input.trailer, input.images_dir, input.music)?;
    let speed = BASE_SPEED * input.trailer.speed;
    let mut synthesizer = Synthesizer::new(data, input.trailer.voice, speed)?;
    synthesizer.read_as(input.language)?;
    render_with_narrator(
        tools,
        input,
        |text, wav| synthesizer.synthesize_to_wav(text, wav),
        progress,
    )
}

/// Render the trailer using `narrate(text, wav_path)` as the speech source, and return the video
/// path.
///
/// This is the whole pipeline minus the voice, so it can be exercised without the model. Work
/// files live in `out_dir/trailer-work` and are always removed; an existing `trailer.mp4` is
/// replaced only when the new one is ready.
pub fn render_with_narrator<E>(
    tools: &Tools,
    input: &TrailerInput<'_>,
    mut narrate: impl FnMut(&str, &Path) -> Result<(), E>,
    mut progress: impl FnMut(TrailerProgress),
) -> Result<PathBuf>
where
    E: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    validate(input.trailer, input.images_dir, input.music)?;
    let work = input.out_dir.join(WORK_DIR);
    let _ = fs::remove_dir_all(&work);
    fs::create_dir_all(&work).map_err(Error::file_error("create", &work))?;
    let mut narrate = |text: &str, wav: &Path| narrate(text, wav).map_err(Into::into);
    let result = assemble(tools, input, &mut narrate, &mut progress, &work);
    let _ = fs::remove_dir_all(&work);
    let video = result?;
    progress(TrailerProgress::Finished);
    Ok(video)
}

#[expect(
    clippy::too_many_lines,
    reason = "manuzio's pipeline, stage by stage, as it was copied"
)]
fn assemble(
    tools: &Tools,
    input: &TrailerInput<'_>,
    narrate: &mut Narrate<'_>,
    progress: &mut impl FnMut(TrailerProgress),
    work: &Path,
) -> Result<PathBuf> {
    let trailer = input.trailer;
    let total = trailer.steps.len();
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a validated fade of 0 to 1 second"
    )]
    let fade_ms = (trailer.fade_seconds * 1000.0).round() as u32;
    // Fades between scenes go into each scene's own encode, so the scenes are joined without
    // encoding the video a second time.
    let fade = f64::from(fade_ms) / 1000.0;
    let fades = |index: usize| {
        (fade_ms > 0 && total > 1).then(|| ClipFades {
            fade_in: if index > 0 { fade } else { 0.0 },
            fade_out: if index + 1 < total { fade } else { 0.0 },
        })
    };
    let mut scenes: Vec<Scene> = Vec::with_capacity(total);
    for (index, step) in trailer.steps.iter().enumerate() {
        check_cancelled(input.cancel)?;
        progress(TrailerProgress::SceneStarted { index, total });
        let image = input
            .images_dir
            .join(step.image.as_deref().unwrap_or_default());
        scenes.push(render_scene(
            tools,
            index,
            step,
            &image,
            fades(index),
            narrate,
            work,
            input.cancel,
        )?);
        progress(TrailerProgress::SceneFinished { index, total });
    }

    check_cancelled(input.cancel)?;
    progress(TrailerProgress::MixingAudio);
    let audio_parts: Vec<_> = scenes.iter().map(|s| s.audio.clone()).collect();
    let length = scenes.iter().map(|s| s.seconds).sum::<f64>();
    let narration = if audio_parts.len() == 1 {
        audio_parts[0].clone()
    } else {
        concat_wav_with_cancel(tools, &audio_parts, AUDIO_RATE, work, input.cancel)?
    };
    let soundtrack = match input.music {
        Some(music) => mix_music_with_cancel(
            tools,
            &narration,
            music,
            &MusicMix {
                volume: trailer.music_volume,
                fade_in: share_of(length, trailer.music_fade_in),
                fade_out: share_of(length, trailer.music_fade_out),
            },
            length,
            work,
            input.cancel,
        )?,
        None => narration,
    };

    check_cancelled(input.cancel)?;
    progress(TrailerProgress::Stitching);
    let clips: Vec<_> = scenes.iter().map(|s| s.clip.clone()).collect();
    let silent = if clips.len() == 1 {
        clips[0].clone()
    } else {
        match concat_matching_video_files_with_cancel(
            tools,
            &clips,
            WIDTH,
            HEIGHT,
            FPS,
            work,
            input.cancel,
        )? {
            Some(combined) => combined,
            // The fades are already in the clips: only make them match.
            None => combine_video_files_with_cancel(
                tools,
                &clips,
                WIDTH,
                HEIGHT,
                0,
                FPS,
                work,
                input.cancel,
            )?,
        }
    };
    check_cancelled(input.cancel)?;
    progress(TrailerProgress::Muxing);
    let muxed = mux_audio_with_cancel(
        tools,
        &silent,
        &soundtrack,
        Some(length),
        work,
        input.cancel,
    )?;

    check_cancelled(input.cancel)?;
    let target = input.out_dir.join(OUTPUT_NAME);
    let _ = fs::remove_file(&target);
    fs::rename(&muxed, &target)
        .or_else(|_| fs::copy(&muxed, &target).map(|_| ()))
        .map_err(Error::file_error("write", &target))?;
    Ok(target)
}
