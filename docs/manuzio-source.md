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
