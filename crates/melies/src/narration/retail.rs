//! The "retail" delivery profile: files that follow the technical rules audiobook stores publish.
//!
//! Such rules usually ask for a constant-bitrate MP3 of at least 192 kbit/s at 44.1 kHz, an overall
//! RMS between -23 and -18 dBFS, peaks below -3 dBFS, a noise floor below -60 dBFS, one to five
//! seconds of room tone before and after the narration, plain file names and files under two
//! hours. Stores differ, and some accept human narration only, so the profile makes the files
//! technically ready and [`problems`] says exactly what is outside the limits; wording it for a
//! report is the caller's.

use kuleshov::{AudioAnalysis, CombineOptions, Mp3Encoding, RoomTone};
use unicode_normalization::UnicodeNormalization;

/// Constant bitrate of retail files, in kbit/s.
pub const BITRATE_KBPS: u32 = 192;
/// Sample rate of retail files, in Hz.
pub const SAMPLE_RATE: u32 = 44_100;
/// Room tone before the first word, in seconds.
pub const LEAD_SECS: f64 = 1.5;
/// Level of the room tone in dBFS RMS, well under the -60 dBFS noise-floor limit.
pub const ROOM_TONE_DBFS: f32 = -70.0;
/// Room tone after the last word must stay between these many seconds.
pub const MIN_PAD_SECS: f64 = 1.0;
pub const MAX_PAD_SECS: f64 = 5.0;
/// The overall level must stay between these, in dBFS RMS.
pub const MIN_RMS_DBFS: f32 = -23.0;
pub const MAX_RMS_DBFS: f32 = -18.0;
/// Peaks must stay at or below this, in dBFS.
pub const MAX_PEAK_DBFS: f32 = -3.0;
/// The noise floor must stay below this, in dBFS.
pub const MAX_NOISE_FLOOR_DBFS: f32 = -60.0;
/// A file must last no longer than this, in seconds (two hours).
pub const MAX_DURATION_SECS: f64 = 120.0 * 60.0;
/// A file shorter than this whose level is too low owes it to the room tone, in seconds.
pub const SHORT_FILE_SECS: f64 = 20.0;

/// How an MP3 is encoded in the retail profile.
pub fn combine_options() -> CombineOptions {
    CombineOptions {
        encoding: Mp3Encoding::Cbr(BITRATE_KBPS),
        room_tone: Some(RoomTone {
            lead_secs: LEAD_SECS,
            level_dbfs: ROOM_TONE_DBFS,
        }),
        tags: Vec::new(),
    }
}

/// Which end of a file its room tone is at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    Start,
    End,
}

/// One way a measured file is outside the retail limits.
#[derive(Debug, Clone, PartialEq)]
pub enum RetailProblem {
    /// The file is not an MP3: its codec as FFprobe names it.
    Format { codec: String },
    /// The bit rate, in bit/s, is below [`BITRATE_KBPS`].
    BitRate { bits_per_second: u32 },
    /// FFprobe reported no bit rate.
    UnknownBitRate,
    /// The sample rate, in Hz, is not [`SAMPLE_RATE`].
    SampleRate { hertz: u32 },
    /// The overall level is outside [`MIN_RMS_DBFS`] to [`MAX_RMS_DBFS`]; `short` when it is too
    /// low in a file under [`SHORT_FILE_SECS`], where the room tone weighs on the average.
    Level { rms_dbfs: f32, short: bool },
    /// The peak is above [`MAX_PEAK_DBFS`].
    Peak { peak_dbfs: f32 },
    /// The noise floor is not below [`MAX_NOISE_FLOOR_DBFS`].
    NoiseFloor { noise_floor_dbfs: f32 },
    /// The room tone at one end lasts outside [`MIN_PAD_SECS`] to [`MAX_PAD_SECS`] (a tenth of a
    /// second either way is allowed).
    RoomTone { edge: Edge, secs: f64 },
    /// The file lasts longer than [`MAX_DURATION_SECS`].
    Duration { secs: f64 },
}

/// A title reduced to plain ASCII letters and digits joined by underscores, for file names.
///
/// Accents are stripped (`Café` becomes `Cafe`); text in other scripts has no ASCII form and
/// gives `None`, so the caller can fall back to a numbered name.
pub fn ascii_stem(title: &str) -> Option<String> {
    let folded: String = title
        .chars()
        .flat_map(|c| match transliterate(c) {
            Some(text) => text.chars().collect::<Vec<_>>(),
            None => c.to_string().nfkd().collect(),
        })
        .filter(char::is_ascii)
        .map(|c| if c.is_ascii_alphanumeric() { c } else { ' ' })
        .collect();
    let stem = folded.split_whitespace().collect::<Vec<_>>().join("_");
    (!stem.is_empty()).then_some(stem)
}

/// Latin letters that do not decompose into a base letter and an accent.
fn transliterate(c: char) -> Option<&'static str> {
    Some(match c {
        'Æ' => "AE",
        'æ' => "ae",
        'Ø' => "O",
        'ø' => "o",
        'Ł' => "L",
        'ł' => "l",
        'Đ' | 'Ð' => "D",
        'đ' | 'ð' => "d",
        'ß' => "ss",
        'Œ' => "OE",
        'œ' => "oe",
        'Þ' => "Th",
        'þ' => "th",
        'ı' => "i",
        'Ħ' => "H",
        'ħ' => "h",
        _ => return None,
    })
}

/// What is outside the retail limits in `analysis`, in this order. Empty means it passes.
pub fn problems(analysis: &AudioAnalysis) -> Vec<RetailProblem> {
    let mut problems = Vec::new();
    if analysis.codec != "mp3" {
        problems.push(RetailProblem::Format {
            codec: analysis.codec.clone(),
        });
    }
    match analysis.bit_rate {
        Some(rate) if rate >= BITRATE_KBPS * 1000 - 1000 => {}
        Some(rate) => problems.push(RetailProblem::BitRate {
            bits_per_second: rate,
        }),
        None => problems.push(RetailProblem::UnknownBitRate),
    }
    if analysis.sample_rate != SAMPLE_RATE {
        problems.push(RetailProblem::SampleRate {
            hertz: analysis.sample_rate,
        });
    }
    if !(MIN_RMS_DBFS..=MAX_RMS_DBFS).contains(&analysis.rms_dbfs) {
        problems.push(RetailProblem::Level {
            rms_dbfs: analysis.rms_dbfs,
            short: analysis.rms_dbfs < MIN_RMS_DBFS && analysis.duration_secs < SHORT_FILE_SECS,
        });
    }
    if analysis.peak_dbfs > MAX_PEAK_DBFS {
        problems.push(RetailProblem::Peak {
            peak_dbfs: analysis.peak_dbfs,
        });
    }
    if analysis.noise_floor_dbfs >= MAX_NOISE_FLOOR_DBFS {
        problems.push(RetailProblem::NoiseFloor {
            noise_floor_dbfs: analysis.noise_floor_dbfs,
        });
    }
    for (edge, secs) in [
        (Edge::Start, analysis.lead_secs),
        (Edge::End, analysis.tail_secs),
    ] {
        if !(MIN_PAD_SECS - 0.1..=MAX_PAD_SECS + 0.1).contains(&secs) {
            problems.push(RetailProblem::RoomTone { edge, secs });
        }
    }
    if analysis.duration_secs > MAX_DURATION_SECS {
        problems.push(RetailProblem::Duration {
            secs: analysis.duration_secs,
        });
    }
    problems
}

#[cfg(test)]
#[path = "../../tests/unit/narration/retail.rs"]
mod tests;
