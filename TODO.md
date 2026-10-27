# TODO

The shared roadmap (see [AGENTS.md](AGENTS.md#shared-roadmap)). melies takes over the narration of
one text from manuzio's `mnz-audiobook` and all of `mnz-trailer`, with the same output
([docs/parity.md](docs/parity.md)).

## Foundation

- [ ] Repository (the owner creates `Federiker/melies` and pushes by hand):
  - [ ] Checks green on Linux (a push) and Windows (a manual run), once it is on GitHub. Locally both
    tiers pass on Windows, and the parity test (both tiers) on Linux in WSL.

## manuzio

- [ ] manuzio switches to melies (manuzio's `SPLITTER-HERE.md`, slice 3; what changes there:
  [docs/manuzio-source.md](docs/manuzio-source.md#switching-manuzio)):
  - [ ] Port what changed in manuzio's `crates/audiobook` and `crates/trailer` since the copied
    commit.
  - [ ] The parity test passes against manuzio's staged FFmpeg and Kokoro files at the pinned commit.
  - [ ] manuzio's adapters checked with `melies-parity`: its reader and heading builder give the
    recorded texts of each case's book (`write_book`), its `BookTrailer` mapping the trailer cases.
  - [ ] `TRAILER_TEXT_MIN` and `TRAILER_TEXT_MAX` (`packages/models/src/trailer.ts`) still agree
    with `MIN_TEXT_CHARS` and `MAX_TEXT_CHARS` (40 and 200).

## Changes to suggest upstream (the owner decides)

- [ ] kuleshov: `kuleshov_parity` could offer its recording shim as a library function, recording
  the WAVs handed to FFmpeg (as arguments or named in concat lists) by size and SHA-256. melies'
  `melies-record` would then be a two-line binary instead of a copy of `kuleshov-record` with that
  addition ([decision 19](docs/decisions.md#while-porting-27-october-2026)).

## From manuzio's TODO: P1, Audiobooks (copied 27 October 2026)

The open items about narration, as manuzio's TODO has them; each says where its work would land.

- [ ] Retail profile:
  - [ ] Spoken opening and closing credits (title, author, narrator, "The End"), localised, saying the
        voice is synthesised (the words are manuzio's; melies narrates them as texts)
  - [ ] Very short chapters can miss the −23 dBFS average because of the room tone (the report flags
        them): consider raising the level of short files (melies' encode: a change of output, so ask
        first and bump `narration::REVISION`)
  - [ ] Opt-in ZIP naming for stores that want one file per section plus credits files; a retail sample
        (≤ 5 minutes) export (manuzio)
  - [ ] Decide whether retail-ready becomes the default (standard: VBR q2 MP3, chapter-title names)
        (manuzio's default; melies' `Profile::default()` follows the decision)
  - [ ] Before a real store submission: the noise floor and silence against the store's own checker
        (ACX's is the reference), each store's policy on synthetic voices, and ACX's limits again
        (melies' `retail` limits)
- [ ] A first main-matter chapter with no text makes a preview of just its heading (manuzio's previews)
- [ ] Abbreviations inside a word are not found (German "Goethestr."): consider suffix entries
      (kempelen's reading rules)
- [ ] Later, with a native speaker: ordinal forms for numbers written with a full stop (German
      "am 3. Mai" → "dritten") (kempelen)

And from manuzio's P1, Chinese market:

- [ ] Read the national standard behind China's labelling Measures and align the `AIGC` tag fields
      (`narration::label`); consider repeating the spoken notice at the start of every file (manuzio
      decides which texts carry the notice)

## Verification

- [ ] For the owner, after manuzio switches: listen to a chapter and watch a trailer made by the
  installed app.
