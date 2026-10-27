//! The integration tests, one program: trailers run FFmpeg (every test runs by default), the
//! narration tests run Kokoro too (the model tier, `mise run test:model`).
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_precision_loss,
    clippy::float_cmp,
    reason = "tests: a failure is a panic"
)]

mod narration;
#[path = "../unit/support.rs"]
mod support;
mod trailer;
