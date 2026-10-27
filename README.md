# melies

A Rust library that narrates a text into an MP3 and renders vertical trailers, dedicated to the public
domain. It is named after Georges Méliès (1861–1938), the stage magician who became the pioneer of
film effects.

melies was split out of [manuzio](https://github.com/Federiker/manuzio), whose audiobook chapters and
book trailers it makes. It stands on two libraries split out of manuzio with it:
[kuleshov](https://github.com/Federiker/kuleshov) (FFmpeg) and
[kempelen](https://github.com/Federiker/kempelen) (Kokoro speech, with
[jespersen](https://github.com/Federiker/jespersen)'s pronunciation). Its output is the same as
manuzio's own crates produced, argument for argument and bit for bit; a parity test checks it
([docs/parity.md](docs/parity.md)).

- **Narration.** A `Narrator` loads a Kokoro voice once (on an NVIDIA GPU when a CUDA runtime is
  given, else, or if that fails, on the CPU) and narrates any number of texts. A text is an optional
  notice that the voice is synthetic, an optional title and subtitle, and a body, each followed by its
  pause. Paragraphs without a letter or digit (`* * *`, `⁂`) are ornaments: never read, and heard as
  a pause. The speech is written into one WAV as it comes, so memory does not grow with the text, then
  encoded once: a variable-bitrate MP3, or the **retail** profile's constant 192 kbit/s at 44.1 kHz with
  peaks at -4 dBFS and room tone, with `retail::problems` to measure a file against the limits stores
  usually publish. A labelled narration carries `comment` and `AIGC` tags. Progress is reported per
  piece and before every model chunk, and cancellation stops before the next chunk or stops FFmpeg.
- **Trailers.** Still images, narration and optional music become a 1080×1920, 30 FPS `trailer.mp4`:
  each step (40 to 200 characters) is narrated, placed in a silent slot of a whole number of frames,
  and its image animated for exactly that slot (zooms and pans, optional dips to black encoded in
  each scene's own encode); the scenes are joined without encoding again, the music is mixed sample
  aligned with no intermediate lossy encode, and the result is muxed. The previous video is replaced
  only on success; cancellation stops FFmpeg and removes the work folder.
  `render_with_narrator` takes any narrator, so the pipeline runs without the model.

```rust
use kempelen::{Language, Voice, assets::SpeechData};
use kuleshov::Tools;
use melies::narration::{NarrationOptions, Narrator, Profile, Text};

let tools = Tools::find_in(["/opt/myapp/bin"]); // ffmpeg and ffprobe
let speech = SpeechData::new("/opt/myapp/speech"); // the Kokoro model and voices
let mut narrator = Narrator::new(&speech, Voice::BfEmma, 0.9, Language::English, None)?;
let text = Text {
    title: Some("Chapter 1".into()),
    subtitle: Some("The Cyclone".into()),
    body: "Dorothy lived in the midst of the great Kansas prairies.\n* * *\nThe end.\n".into(),
    ..Text::default()
};
let options = NarrationOptions { profile: Profile::Retail, ..NarrationOptions::default() };
// Some(the MP3, in out_dir/audio), or None for a text with nothing to say.
let mp3 = narrator.narrate(&tools, &text, &options, &out_dir, None, |progress| {})?;
```

```rust
use melies::trailer::{Movement, Step, Trailer, TrailerInput, render_trailer};

let trailer = Trailer {
    steps: vec![Step {
        image: Some("cover.jpg".into()),
        text: "A girl from Kansas is swept away to a land of wonders.".into(),
        movement: Movement::ZoomIn,
    }],
    voice: Voice::AfHeart,
    fade_seconds: 0.5,
    ..Trailer::default()
};
let input = TrailerInput {
    trailer: &trailer,
    language: Language::English,
    images_dir: &images, // the steps' images, by plain file name
    music: Some(&music),
    out_dir: &out_dir, // receives trailer.mp4
    cancel: None,
};
let video = render_trailer(&tools, &speech, &input, |progress| {})?;
```

Every function that runs FFmpeg takes kuleshov's `Tools` first; the speech files are the caller's
kempelen `SpeechData`. melies reads no environment variables and downloads nothing. Errors are one
enum, `melies::Error` ([docs/errors.md](docs/errors.md)). The whole API:
[crates/melies/src/lib.rs](crates/melies/src/lib.rs) and its modules.

### Using it

melies is not on crates.io. Pin the repository by a release tag (see Releasing below), with kuleshov
and kempelen at the tags its own `Cargo.toml` pins:

```toml
melies = { git = "https://github.com/Federiker/melies", tag = "v<version>" }
```

FFmpeg and the Kokoro files are the program's to supply: kuleshov's `Tools::find_or_download` and
kempelen's `assets::stage` install the pinned ones when asked (their `download` and
`speech-download` features). It is tested on Windows and Linux, with kuleshov's pinned FFmpeg builds.

## Licence

melies is dedicated to the public domain under [CC0 1.0](LICENSE). kuleshov is CC0 too; kempelen is
Apache-2.0, with its NOTICE, which a program that ships melies ships for kempelen. FFmpeg, the Kokoro
model and voices are not part of melies and keep their own licences.

## Setup

On a new machine, `scripts/install.sh` (macOS, Linux, Git Bash) or `scripts\install.bat` (Windows)
installs mise and the pinned tools ([mise.toml](mise.toml)), the Rust toolchain
([rust-toolchain.toml](rust-toolchain.toml)) and the Git hooks. Run it again at any time.

The tests run FFmpeg, and the model tier Kokoro. `mise run stage` downloads once the FFmpeg build
kuleshov pins for this platform and the model and voices kempelen pins (about 349 MB), checking every
file's SHA-256, into the gitignored `tools/` and `tools/speech/`; the tests find them through
`MELIES_TOOLS_DIR` and `MELIES_SPEECH_DIR`. To use copies from elsewhere, such as a sibling manuzio
checkout's, set them in a gitignored `mise.local.toml`:

```toml
[env]
MELIES_TOOLS_DIR = "{{config_root}}/../manuzio/assets/tools"
MELIES_SPEECH_DIR = "{{config_root}}/../manuzio/assets/tools/tts"
```

`mise tasks` lists the commands; the main ones:

| Command | Does |
| --- | --- |
| `mise run stage` | Stage the pinned FFmpeg and the Kokoro files in `tools/` |
| `mise run test` | The tests that need FFmpeg only, the parity test's among them |
| `mise run test:model` | The tests that run Kokoro too (`#[ignore]`d otherwise) |
| `mise run coverage` | Both tiers with coverage, failing under 90 % of lines or regions |
| `mise run clippy` | Clippy, warnings denied |
| `mise run bench` | The narration and trailer benchmark ([docs/performance.md](docs/performance.md)) |
| `mise run audit` | Dependency advisories, licences and sources |
| `mise run ci` | Everything CI's light tier runs |
| `mise run tag` | Sets the version, commits it, tags `v<version>` and pushes |

## Releasing

A release is the tag `v<version>` on `main`, with the version set once in the
root `Cargo.toml` (`[workspace.package]`). There is no GitHub release or
crates.io upload (the crates are `publish = false`), so plain Git is enough:
no token, only the credentials you already push with. Programs still pin the
commit a tag names (`git rev-parse v<version>^{commit}`), which no later
change to the tag can move.

Run `mise run tag` on `main` with everything committed and pushed or
pulled. It asks the new version (Enter for the next patch, or the current
one when it was never tagged), writes it to `Cargo.toml` and `Cargo.lock`
and commits those alone (`[release] melies <version>` by default, with
the hooks), tags `v<version>` (annotated), then asks before
`git push --atomic origin main v<version>`. Nothing changes before the
commit; if you stop after it, it says what it did, how to push later and
how to undo it here. The tool is the `melies-tag` crate, copied from
jespersen's.

## Local scratch data

The gitignored `tmp/` folder holds the harness that records the parity references from manuzio,
benchmark outputs, experiments and disposable scripts. Nothing in it is a build input.

## Documentation

[docs/](docs/README.md): the decisions, the parity references and how they were recorded, the errors,
the performance against manuzio's, and the manuzio commit the code was copied from.

## Git conventions

[.gitattributes](.gitattributes) normalises text to LF, with CRLF for Windows batch scripts.

### Hooks

[Lefthook](lefthook.yml) runs three hooks, installed by the install scripts:

- **before a commit**, rustfmt formats the staged Rust files, and the results are staged;
- **on the commit message**, the first line must start with a prefix below;
- **before a push**, Clippy (`mise run clippy`).

`git commit --no-verify` skips them; CI runs the same checks anyway.

### Commit messages

The first line is `[prefix] What changed`, in the imperative and UK English, for example
`[fix] A cancelled narration leaves no WAV`. One commit, one prefix: when a change is two kinds of
work, it is usually two commits. Git's own `Merge`, `Revert` and `fixup!` messages are accepted as
they are.

| Prefix | For |
| --- | --- |
| `[add]` | Something new: a function, an option, a test tier |
| `[fix]` | A bug fixed |
| `[remove]` | A feature, file or dead code taken out |
| `[refactor]` | Code reorganised with the same behaviour (the parity test agrees) |
| `[rename]` | Files, crates or identifiers renamed or moved, nothing else |
| `[perf]` | Faster or smaller, same output; its measurements belong in the message and [docs/performance.md](docs/performance.md) |
| `[data]` | References re-recorded or fixtures changed; their source (the manuzio commit, the FFmpeg build, the machine) belongs in the message |
| `[docs]` | Documentation only |
| `[test]` | Tests only |
| `[fmt]` | Formatting only (large ones go in `.git-blame-ignore-revs`) |
| `[build]` | Tooling, scripts and configuration |
| `[ci]` | The CI workflows |
| `[deps]` | Dependency updates, the kuleshov and kempelen pins included |
| `[security]` | A security fix or hardening |
| `[release]` | A version and its notes |
| `[revert]` | A change undone by hand |
| `[init]` | A repository's first commit |
