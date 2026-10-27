# Performance

melies runs manuzio's narration and trailer pipeline unchanged, on kuleshov's and kempelen's copies of
manuzio's media crates, so it should be exactly as fast. This page keeps the numbers that show it, and
manuzio's own measurements from before the split.

## The benchmark

`mise run bench -- audiobook` and `mise run bench -- trailer`
(`cargo run --release -p melies --example bench -- …`) are manuzio's `mnz-bench audiobook` and
`trailer`:

- `audiobook`: the Introduction and Chapters I and II of manuzio's sample book (*The Wonderful Wizard
  of Oz*, public domain), the texts manuzio read for the parity references, read by `bf_emma` at 0.9
  in English, standard profile, on the CPU: three texts, 8 segments, three MP3 encodes.
- `trailer`: three 1080×1920 scenes (zoom in, pan, zoom out) with `bf_emma`, music and 0.5 s fades,
  from manuzio's `01.jpg` and `02.jpg` and `fiction_hopeful.opus` when given
  (`--images ../manuzio/assets/samples --music ../manuzio/assets/audios/fiction_hopeful.opus`, read
  only), else from the generated parity inputs.

Each prints the time of every stage, the peak memory (peak working set, Windows only) and the SHA-256
of every file and of its decoded audio (and frames). The manuzio side is the same program through
manuzio's crates, in the harness (`melies-harness bench`, [parity.md](parity.md#recording-them-again)),
printed the same way. The audiobook's manuzio side also writes the ZIP (its "archive" stage), which is
manuzio's, not melies'.

## melies against manuzio, 27 October 2026

Alternately, release builds, the same staged FFmpeg (gyan.dev 9.0.2) and Kokoro files, on the CPU.
Machine: AMD Ryzen 7 5800H (8 cores, 16 threads), 31 GB RAM, Windows 11, shared with other sessions,
which took 12–20 % of the CPU in bursts.

**Audiobook** (1,089.9 s of audio):

| | manuzio 1 | melies 1 | manuzio 2 | melies 2 | manuzio 3 | melies 3 |
| --- | --- | --- | --- | --- | --- | --- |
| model load | (in speech) | 1.17 s | (in speech) | 3.72 s | (in speech) | 1.34 s |
| speech and padding | 303.71 s | 458.01 s | 553.47 s | 342.11 s | 285.91 s | 280.59 s |
| chapter encodes | 6.40 s | 8.34 s | 13.27 s | 6.45 s | 5.25 s | 5.03 s |
| archive (manuzio's) | 0.31 s | — | 1.08 s | — | 0.33 s | — |
| **total** | 310.48 s | 467.53 s | 567.84 s | 352.27 s | **291.50 s** | **286.95 s** |
| real time | 3.51× | 2.33× | 1.92× | 3.09× | 3.74× | 3.80× |
| peak memory | 1,090 MB | 1,073 MB | 1,081 MB | 1,073 MB | 1,081 MB | 1,072 MB |

Runs 1 and 2 swung by up to 80 % with the other sessions' load, both ways for both programs; the third
pair ran on a quieter machine and agrees within 2 %, as manuzio's own final measurement (289.3 s,
3.77×, 1,109 MB) does. The model is over 98 % of the time, and it is the same ONNX Runtime build,
session and thread count. Every run of both gave the same three MP3s, byte for byte
(`c13b831c…`, `e98fd9ab…`, `e7ac6270…`) and so the same decoded audio.

**Trailer** (manuzio's `01.jpg`, `02.jpg` and `fiction_hopeful.opus`):

| | manuzio 1 | melies 1 | manuzio 2 | melies 2 |
| --- | --- | --- | --- | --- |
| model load | 1.41 s | 1.07 s | 1.11 s | 1.40 s |
| scenes (narration and motion) | 32.84 s | 29.42 s | 28.70 s | 31.19 s |
| of which narration | 4.37 s | 4.20 s | 3.89 s | 4.24 s |
| narration and music mix | 0.22 s | 0.19 s | 0.19 s | 0.22 s |
| stitching (fades) | 0.34 s | 0.28 s | 0.29 s | 0.33 s |
| muxing | 0.49 s | 0.47 s | 0.44 s | 0.47 s |
| **total** | 35.30 s | 31.43 s | 30.72 s | 33.61 s |
| peak memory | 488 MB | 487 MB | 489 MB | 487 MB |

The same video in every run, byte for byte (`a201dcbb…`), so the same frames and sound.

## manuzio's earlier measurements (its docs/optimisation-notes.md)

Measured on the same laptop, 23 October 2026, before kuleshov and kempelen (with eSpeak NG then:
1,089 s of audio for the same chapters):

| | Time |
| --- | --- |
| baseline audiobook, CPU, ONNX Runtime's default threads (8) | 314.5 / 330.0 s (3.46× / 3.30×), peak 1,220 MB |
| final audiobook after streaming the WAV and the other slices | 289.3 s (3.77×), peak 1,109 MB |
| the same on CUDA (RTX 3060 Laptop at a 30 W power cap) | 155.9 s, peak 1,383 MB |
| cancelling inside a chapter body (before the next model chunk) | 0.6 s (104–188 s before) |
| baseline trailer, CPU | 40.5 / 53.3 s |
| final trailer, with the fades in each scene's own encode | 32.5 s (stitching 0.3 s instead of 9.2 s) |

CUDA was not measured here: it needs the 1.6 GB runtime, and on this laptop the GPU is capped at 30 W.
