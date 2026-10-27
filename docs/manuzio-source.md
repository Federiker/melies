# Copied from manuzio

melies is split out of [manuzio](https://github.com/Federiker/manuzio). Its code and tests were copied
from this commit, whose crates `crates/audiobook` and `crates/trailer` (and every crate they use) were
unchanged in the working tree:

| | |
| --- | --- |
| Commit | `0ec8e8ede4be114c3f7d715f196a622e94c1b989` (25 October 2026) |
| Subject | [deps] Slice 12: kuchiki and html5ever 0.25 replaced by dom_query 0.29 (editing) and scraper 0.29 (reading) |
| Copied on | 27 October 2026 |

It is the commit kuleshov and kempelen were copied from too.

| manuzio | Here |
| --- | --- |
| `mnz-audiobook` (`crates/audiobook`): the narration of one chapter (`plan.rs`, `label.rs`, `retail.rs`, the segment loop of `create.rs`) | narrating one text |
| `mnz-trailer` (`crates/trailer`), all of it | trailers |

Before manuzio switches to melies, port whatever changed there since this commit:

```sh
git -C ../manuzio log 0ec8e8ede4be114c3f7d715f196a622e94c1b989..HEAD -- crates/audiobook crates/trailer
```

## What came from where

| melies | manuzio |
| --- | --- |
| `narration::plan`, `Segment`, `SegmentKind`, `notice_segment`, `is_speakable`, `NOTICE_PADDING` | `plan.rs` (`plan_chapter` without the heading it builds) |
| `narration::Narrator::narrate` | `create.rs`: `speak_chapter`, the encode of one chapter, `load_synthesizer`, `load_reader` |
| `narration::NarrationOptions`, `Profile` | `options.rs` (`AudiobookOptions::effective`, `AudiobookProfile`) |
| `narration::NarrationProgress` | `AudiobookProgress::Speaking` and `SegmentFinished`, without the chapter |
| `narration::label` | `label.rs`, with the producer given by the caller |
| `narration::retail` | `retail.rs` without the report: the constants, `combine_options`, `problems` (typed), `ascii_stem` |
| `narration::REVISION` | `resume.rs`'s `PLAN_REVISION` |
| `trailer` | all of `mnz-trailer`, with its own input types |
| the unit and integration tests | `tests/unit/{plan,label,retail}.rs`, `tests/audiobook.rs` (the narration ones), all of `mnz-trailer`'s |

## Switching manuzio

What changes in manuzio when it uses melies (its `SPLITTER-HERE.md`, slice 3):

| manuzio now | With melies |
| --- | --- |
| `plan_chapter(chapter, prefix, options)` | manuzio builds the heading ("Chapter 3", "第3章", the title) and calls `melies::narration::plan(&Text { notice, title, subtitle, body }, &options)`; `chapter_note` stays in manuzio, on that plan (`SegmentKind::Body` for "has text") |
| `speak_chapter` + `combine_audio_files_with_options` + the chapter's tags | `Narrator::new(&speech, voice, speed, language, cuda_runtime)` once per book, then `narrator.narrate(&tools, &text, &options, &work, cancel, progress)` per chapter: `Some(mp3)` to rename into the parts folder, `None` for a chapter with nothing to say |
| `AudiobookOptions { pad_chapter, pad_between_chapters, output_rate, profile, ai_label }` | `NarrationOptions { heading_pause, closing_pause, sample_rate, profile, label }`; voice, speed and language go to the `Narrator` |
| `label::tags(content_id)` | `Label { producer: "manuzio".into(), content_id }`; the notice text (the language pack's `aiNotice`, English when empty) is the first chapter's `Text::notice` |
| `retail::problems(&analysis)` (sentences) | `melies::narration::retail::problems(&analysis)` (`RetailProblem` values, which manuzio words for `retail-check.txt` as before); `retail::ascii_stem` and the constants come from melies |
| the resume fingerprint's `PLAN_REVISION` | `melies::narration::REVISION` (the same value, 1, so earlier runs still resume) |
| `AudiobookProgress::Speaking { done, of }`, `SegmentFinished` | `NarrationProgress`'s same fields; manuzio adds the chapter's index, total and title |
| `mnz_tools::ProcessError::Cancelled` after narration | `melies::Error::is_cancelled()` |
| `mnz_trailer::render_trailer(&input, progress).await` | `melies::trailer::render_trailer(&tools, &speech, &input, progress)`, synchronous |
| `BookTrailer`, `TrailerStep`, `TrailerMovement` | `trailer::Trailer`, `Step`, `Movement`: mapped field for field (`voice_id` → `voice`); `music_path` is resolved by manuzio as now, `add_text` and `text` are not used (they never were) |
| `TrailerError::Cancelled` | `melies::Error::Cancelled` (`is_cancelled()`); the other `TrailerError`s are `melies::Error::Trailer(…)`, with the same messages |
| `is_plain_file_name`, `OUTPUT_NAME`, `MIN_TEXT_CHARS`, `MAX_TEXT_CHARS`, `BASE_SPEED` | the same names in `melies::trailer` |
| `anyhow::Result` | `melies::Result`; `?` turns it into `anyhow` at manuzio's boundaries |

The messages manuzio shows are kept: the trailer's validation messages ("Step 1 has no image"),
"Failed to narrate step N", "Failed to animate the image of step N", "ffmpeg failed to mix the music",
"Failed to write …". manuzio's own context ("Failed to read chapter N aloud") stays manuzio's.
