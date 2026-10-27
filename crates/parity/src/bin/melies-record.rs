//! The recording shim: kuleshov's ([`kuleshov_parity::Shim`]), which also records each WAV among
//! the arguments, or named in a concat list, by size and SHA-256. The narration's WAV is gone once
//! the MP3 is encoded, and must be the same bit for bit.

use std::process::ExitCode;

fn main() -> ExitCode {
    kuleshov_parity::Shim {
        name: "melies-record",
        wavs: true,
    }
    .run()
}
