# Errors

Every fallible function returns [`melies::Error`](../crates/melies/src/error.rs) (`melies::Result`).
The library never uses `anyhow`; manuzio turns the error into its own at its boundaries with `?`.

| Variant | When | Message |
| --- | --- | --- |
| `Cancelled` | The caller's flag was set: FFmpeg was stopped, or the next model chunk or stage never started; what had been written was removed | `cancelled` |
| `Trailer(TrailerError)` | A trailer setting prevents rendering, before any work | the `TrailerError`'s, as manuzio's (below) |
| `Media(kuleshov::Error)` | FFmpeg or FFprobe failed, or could not be started | kuleshov's, which names the file |
| `Speech(kempelen::Error)` | The model or a voice could not be loaded, or the synthesis failed (no speech, a language narration cannot read) | kempelen's |
| `Narration { step, source }` | A trailer narrator failed (the caller's, or Kokoro's) | `Failed to narrate step N` |
| `Animation { step, source }` | FFmpeg could not animate a step's image | `Failed to animate the image of step N` |
| `Music { music, source }` | FFmpeg could not mix the narration with the music | `ffmpeg failed to mix the music` |
| `File { action, path, source }` | The work folder could not be made, or the trailer moved into place | `Failed to create …`, `Failed to write …` |

`error.is_cancelled()` and `error.file()` (the file an error is about: kuleshov's, the music, the
path) answer the common questions. Cancellation is never reported as another error: a cancelled
FFmpeg run (`kuleshov::Error::Cancelled`) and a synthesis stopped before a model chunk (kempelen's
`Error::Interrupted` with melies' own marker) both become `Error::Cancelled`, even inside a step's
animation or the music mix.

`TrailerError`'s messages, unchanged from manuzio's, because manuzio shows them:

| `TrailerError` | Message |
| --- | --- |
| `NoSteps` | The trailer needs at least one step |
| `MissingImage { step }` | Step N has no image |
| `UnsafeImageName { step }` | Step N has an invalid image name |
| `ImageNotFound { step }` | The image of step N was not found |
| `TextTooShort { step, min }` | Step N needs at least 40 characters of text |
| `TextTooLong { step, max }` | Step N has more than 200 characters of text |
| `MusicNotFound` | The selected music file was not found |
| `InvalidSetting(what)` | Invalid trailer setting: speed must be between 0 and 2 (and the others) |

manuzio's `TrailerError::Cancelled` is `Error::Cancelled` here. The retail limits are not errors:
`narration::retail::problems` returns `RetailProblem` values, which the caller words.
