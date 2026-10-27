# Project guidance

Shared by every coding assistant (Claude reads it through [CLAUDE.md](CLAUDE.md)) and by people.
Read [README.md](README.md) for setup and the API, [docs/parity.md](docs/parity.md) before changing
anything that reaches FFmpeg or Kokoro, and [docs/manuzio-source.md](docs/manuzio-source.md) for the
manuzio commit the code was copied from.

## The same output as manuzio (read first)

melies narrates a text and renders trailers for manuzio, which pins it by commit. It was split out of
manuzio's `mnz-audiobook` (the narration of one chapter) and `mnz-trailer`, and must produce exactly
what they produced:

- The committed references in `crates/parity/references/` (the texts and plans manuzio made of its
  sample book, every FFmpeg and FFprobe call with the WAVs handed to them, and the outputs' hashes)
  were **recorded from manuzio's crates**, with the harness in the gitignored `tmp/harness`. They are
  only ever re-recorded from manuzio, the same way; never from melies itself, and never edited by hand
  to make the parity test pass.
- The references belong to kuleshov's pinned FFmpeg build per platform and to kempelen's pinned model
  and voices; the Kokoro cases to one kind of machine too ([docs/parity.md](docs/parity.md)). A new
  kuleshov or kempelen pin that changes the output means new references: ask the owner first.
- A change that alters the plan, the arguments or the output is a decision for the owner, not a
  refactor: ask before making it, and bump `narration::REVISION` when it changes what a narration
  sounds like.
- Cancellation stops FFmpeg and the narration before the next model chunk, and leaves no WAV, MP3 or
  work folder; the previous `trailer.mp4` is replaced only on success; errors name the step or the
  file. Each has its tests: keep them.

## Shared roadmap

- Read the root [TODO.md](TODO.md) at the start of each task to understand the remaining work and
  avoid duplicating another session's progress.
- Update relevant checklist items before finishing: add newly discovered work, split large tasks,
  note blockers and tick completed slices only after implementation and verification. A stub does
  not complete a feature. Once an item is done and described in the docs, remove it.
- Keep roadmap changes focused on the current task; preserve other contributors' entries and
  unresolved decisions.

## Layout

| Path | What |
| --- | --- |
| `crates/melies` | The library: `narration` (plan, narrator, label, retail) and `trailer` |
| `crates/melies/tests` | Unit tests (`tests/unit`, included with `#[path]`) and the integration tests (`tests/it`, one program) |
| `crates/melies/examples/bench.rs` | The benchmark, manuzio's `mnz-bench audiobook` and `trailer` ([docs/performance.md](docs/performance.md)) |
| `crates/parity` | `melies-parity` (unpublished): the cases, the recording shim, the references and the parity test, which manuzio can run too |
| `crates/stage` | `melies-stage` (unpublished): stages the pinned FFmpeg and Kokoro files with kuleshov's and kempelen's downloads |
| `scripts` | The install scripts and the commit-message hook |
| `docs` | The documentation (index below) |
| `tmp` | Gitignored: the reference harness (`tmp/harness`, path dependencies on `../manuzio`), benchmark outputs, experiments |

## Tests

- **Light tier** (`mise run test`): FFmpeg, no model. Every test that needs only FFmpeg runs by
  default, the parity test's plans and model-free trailers among them; one that needs FFmpeg fails
  clearly, naming the folder, when it is missing. CI runs it on Linux and Windows.
- **Model tier** (`mise run test:model`): the `#[ignore]`d tests that run Kokoro on the CPU, reading
  the staged files in `MELIES_SPEECH_DIR`. The parity test's Kokoro cases run only on the references'
  machine and say why elsewhere.
- Keep coverage at 90 % or more of lines and regions over both tiers (`mise run coverage`, which CI's
  model tier runs: the light tier alone cannot narrate); new code comes with its tests.

## Conventions

- UK English in identifiers, documentation and messages. Names kept from manuzio's API for its callers
  and FFmpeg's own names (`zoompan`, `normalize=0`) stay as they are.
- Rust: dependencies as `{ workspace = true }`; Clippy `all` and `pedantic` warn, and CI denies
  warnings. **No inline test modules**: unit tests live in `crates/<crate>/tests/unit/…` and are
  included with `#[cfg(test)] #[path = "…"] mod tests;`; integration tests in `crates/<crate>/tests/`.
  `lib.rs` and `mod.rs` stay declarative: modules and exports only.
- Typed errors: `melies::Error` ([docs/errors.md](docs/errors.md)); `anyhow` never in the library.
  Report errors explicitly; do not substitute empty or success-shaped values.
- The library takes inputs: no environment variables, no app folders, no app names. The caller
  says where FFmpeg is (kuleshov's `Tools`) and where the Kokoro files are (kempelen's
  `SpeechData`), and gives its own name for the AI label. Only tests, examples and `melies-stage`
  read `MELIES_TOOLS_DIR` and `MELIES_SPEECH_DIR`.
- **Never commit FFmpeg, the model, the voices or any media** they make: the tests generate their
  inputs with FFmpeg, and the references are hashes, argument lists and texts.
- kuleshov and kempelen are pinned by commit in the root `Cargo.toml` (with their parity crates at the
  same commits). If one of them needs a change, write it down in [TODO.md](TODO.md) for the owner, or
  ask whether to make it there; do not copy code around it.
- Ask before decisions that materially change the public API, the output or the licence (CC0 1.0).
- Not on crates.io: manuzio pins the repository by commit. Until the first release, APIs need not stay
  backwards compatible.
- The version is set once, in the root `Cargo.toml` (`[workspace.package]`).
- Commit messages start with a `[prefix]`, as described in the README's "Git conventions" (the
  commit-msg hook checks it), for example `[fix] A cancelled narration leaves no WAV`.

## Assistant conduct

- **Local notes:** if `tmp/docs/README.md` exists, read it at the start of each task and follow it;
  it holds local-only instructions that are deliberately not committed (`tmp/` is gitignored).
- Use the gitignored root `tmp/` folder for downloads, the reference harness, benchmark outputs and
  experiments. Do not stage it or treat its contents as build inputs. Keep durable findings in
  tracked docs.
- manuzio, kuleshov, kempelen, jespersen and the other sibling repositories are read only from here.
  Do not run manuzio's app or its dev servers.
- Ask before downloading anything large (the Kokoro files are about 349 MB, an FFmpeg build about
  100 MB).
- Stop helper processes when finished.
- Ask about material uncertainties instead of guessing: anything that affects the output, the
  public API or the licence. When a decision is the owner's, ask with a small set of concrete
  choices and a recommended option (the question tool when there is one, numbered choices
  otherwise), allowing a custom answer.

Be courteous and constructive. If a request includes insults or abusive language directed at the
assistant, politely decline to continue that request.

## Docs index

- [docs/manuzio-source.md](docs/manuzio-source.md): the manuzio commit melies was copied from, what came from where, and what changes in manuzio when it switches.
- [docs/parity.md](docs/parity.md): the references recorded from manuzio, the harness and the parity test.
- [docs/decisions.md](docs/decisions.md): the owner's decisions and the choices made while porting.
- [docs/errors.md](docs/errors.md): the error type and its messages.
- [docs/performance.md](docs/performance.md): speed and memory, measured against manuzio's.
