use crate::{Error, Result};
use kuleshov::{Tool, Tools, run_cancellable};
use std::{
    fmt::Write as _,
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
};
use uuid::Uuid;

/// ffmpeg filter graph that mixes input 0 (narration) with input 1 (music).
///
/// The music is scaled by `volume`, cut to `total_secs`, and faded in and out over
/// the given seconds (0 disables a fade). The narration is never attenuated.
pub(crate) fn music_filter(volume: f32, fade_in: f64, fade_out: f64, total_secs: f64) -> String {
    let mut music = format!(
        "[1:a]aresample=44100,aformat=channel_layouts=stereo,volume={volume},atrim=0:{total_secs}"
    );
    if fade_in > 0.0 {
        let _ = write!(music, ",afade=t=in:st=0:d={fade_in}");
    }
    if fade_out > 0.0 {
        let _ = write!(
            music,
            ",afade=t=out:st={}:d={fade_out}",
            (total_secs - fade_out).max(0.0)
        );
    }
    format!(
        "[0:a]aresample=44100,aformat=channel_layouts=stereo[voice];{music}[music];\
[voice][music]amix=inputs=2:duration=first:dropout_transition=0:normalize=0[mix]"
    )
}

/// The music settings of a mix: its level, and its fades in seconds.
pub(crate) struct MusicMix {
    pub volume: f32,
    pub fade_in: f64,
    pub fade_out: f64,
}

/// Mix `narration` with `music`, looped and cut to `total_secs`, into a 32-bit float WAV in
/// `out_dir`: sample aligned, with headroom, and no intermediate lossy encode (the mux encodes it
/// once).
pub(crate) fn mix_music_with_cancel(
    tools: &Tools,
    narration: &Path,
    music: &Path,
    mix: &MusicMix,
    total_secs: f64,
    out_dir: &Path,
    cancel: Option<&AtomicBool>,
) -> Result<PathBuf> {
    let output = out_dir.join(format!("{}.wav", Uuid::new_v4()));
    let mut command = tools.command(Tool::Ffmpeg);
    command
        .args(["-y", "-i"])
        .arg(narration)
        .args(["-stream_loop", "-1", "-i"])
        .arg(music)
        .args(["-filter_complex"])
        .arg(music_filter(
            mix.volume,
            mix.fade_in,
            mix.fade_out,
            total_secs,
        ))
        .args(["-map", "[mix]", "-t"])
        .arg(total_secs.to_string())
        .args(["-c:a", "pcm_f32le"])
        .arg(&output);
    if let Err(error) = run_cancellable(&mut command, cancel) {
        let _ = std::fs::remove_file(&output);
        return Err(if error.is_cancelled() {
            Error::Cancelled
        } else {
            Error::Music {
                music: music.to_owned(),
                source: error,
            }
        });
    }
    Ok(output)
}

#[cfg(test)]
#[path = "../../tests/unit/trailer/mix.rs"]
mod tests;
