//! The sound of a scene that is not narrated: silence as long as its slot, in the format of a
//! narrated scene's (16-bit mono PCM at [`AUDIO_RATE`]).

use crate::{Error, Result, trailer::timing::AUDIO_RATE};
use std::{
    fs::File,
    io::{BufWriter, Write as _},
    path::Path,
};

/// The samples of `seconds` at [`AUDIO_RATE`].
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a slot of seconds, never negative"
)]
pub(crate) fn sample_count(seconds: f64) -> u32 {
    (seconds.max(0.0) * f64::from(AUDIO_RATE)).round() as u32
}

/// Write `seconds` of silence to the WAV at `path`.
pub(crate) fn write_silence(path: &Path, seconds: f64) -> Result<()> {
    let data = sample_count(seconds) * 2;
    let write = || -> std::io::Result<()> {
        let mut file = BufWriter::new(File::create(path)?);
        file.write_all(b"RIFF")?;
        file.write_all(&(36 + data).to_le_bytes())?;
        file.write_all(b"WAVEfmt ")?;
        file.write_all(&16_u32.to_le_bytes())?;
        // PCM, one channel.
        file.write_all(&1_u16.to_le_bytes())?;
        file.write_all(&1_u16.to_le_bytes())?;
        file.write_all(&AUDIO_RATE.to_le_bytes())?;
        file.write_all(&(AUDIO_RATE * 2).to_le_bytes())?;
        // Two bytes a sample, 16 bits.
        file.write_all(&2_u16.to_le_bytes())?;
        file.write_all(&16_u16.to_le_bytes())?;
        file.write_all(b"data")?;
        file.write_all(&data.to_le_bytes())?;
        let zeros = [0_u8; 4096];
        let mut left = data as usize;
        while left > 0 {
            let chunk = left.min(zeros.len());
            file.write_all(&zeros[..chunk])?;
            left -= chunk;
        }
        file.flush()
    };
    write().map_err(Error::file_error("write", path))
}

#[cfg(test)]
#[path = "../../tests/unit/trailer/silence.rs"]
mod tests;
