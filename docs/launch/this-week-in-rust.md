# This Week in Rust

DRAFT – no pull request opened.

## How TWiR takes submissions now (checked 2026-11-18, read only)

Read from github.com/rust-lang/this-week-in-rust (README, `draft/2026-11-18-this-week-in-rust.md`,
issue #8575):

- **Project/Tooling Updates no longer takes pull requests.** Since issue 664 the editors
  pick those links from r/rust only, and the post there has to follow the subreddit's
  guidelines. The draft file says so in a comment under the heading. So for the project
  itself: post on r/rust (`reddit-rust.md`) – that *is* the submission. No PR.
- **LLM-written articles are not accepted** ("We don't accept articles written by LLMs",
  README, section *LLM-written articles*). TWiR has no policy on LLM-written code. The
  blog post may go in only if you wrote it yourself; `blog-post.md` is an AI draft and
  must be rewritten first.
- Other sections still take PRs against the draft of the next issue,
  `draft/<YYYY-MM-DD>-this-week-in-rust.md` (one issue per Wednesday; 2026-11-18 is issue
  678, so the next draft is `2026-11-25-this-week-in-rust.md` once it appears).
- Link text = the title of the page, canonical URL, no `utm_` parameters.

## Option 1: the blog post (only if written by you)

Section **Observations/Thoughts** (articles about Rust; **Rust Walkthroughs** is for
posts with a lot of source code that walk through building something – the post is
closer to Observations). Line to add under `### Observations/Thoughts`:

```markdown
* [Building an EPLAN-style electrical CAE in Rust that runs in the browser]([blog-url])
```

PR title: `Add "Building an EPLAN-style electrical CAE in Rust that runs in the browser"`.
PR body: one sentence, e.g. "A write-up on a 24-crate Rust workspace (egui, native and
wasm): one drawing trait for screen/PDF/DXF, a plugin registry shared by editor, CLI and
MCP server, and shrinking the wasm from 61 to 32 MB." Say in the PR whether AI tools
were used for the text and how, if at all – the editors decide.

## Option 2: Call for Participation – Projects

Takes up to two issues per TWiR issue. Requirements (README, *Call for Participation
guidelines*): an OSI license (yes: MIT / Apache-2.0), a public issue tracker (yes), a new
issue per task with a clear description and the difficulty (easy / medium / hard /
tedious) as a label or in the title, a link to the contribution guidelines in the
issue (there is no `CONTRIBUTING.md` yet; link `AGENTS.md`, which holds the conventions,
or add one).

Format (comment in the draft: `* [project name - title of issue](URL to issue)`), under
`### CFP - Projects`:

```markdown
* [OpenECraft - [easy] Review the Dutch translation of the editor](https://github.com/nilres/OpenECraft/issues/[n])
* [OpenECraft - [medium] Measure and reduce the size of the web editor's WebAssembly further](https://github.com/nilres/OpenECraft/issues/[n])
```

Candidate issues to open first (they are suggestions, none exists yet):

- **[easy] Review the translation of one of the 16 further languages** – the catalogs are
  `crates/openecraft-i18n/src/catalog/<area>/<code>.tsv`; a text without a line is shown
  in English; tests check placeholders.
- **[medium] Wasm size** – 32 MB (8 MB Brotli) today with `opt-level = "s"` and fat LTO;
  try `wasm-opt`, `twiggy` analysis, feature-gating heavy crates (STEP import, DWG) in
  the web build.
- **[medium] A missing symbol set** – GB, GOST and JIS sets are partial
  (`docs/eplan-gap-analysis.md`, section 7).

## Option 3: nothing else

Do not add the project under Project/Tooling Updates by PR – it will be closed.
