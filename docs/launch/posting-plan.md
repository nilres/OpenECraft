# Launch posting plan

DRAFT. Dates are a proposal from 2026-11-18; move them as a block.

## Principles

- One audience per day, so you can stay in each thread for the first 3–4 hours.
- Developers first (they forgive rough edges and find bugs), engineers a week later
  (they judge by whether it works on a real job – fix what the first round found).
- Disclose the AI-written code in every post, up front. Rust venues in particular
  (TWiR, r/rust, users.rust-lang.org) are sceptical of LLM-generated projects, and the
  commit trailers make it visible anyway.
- Write the final texts and every reply yourself. The drafts in this folder were made
  by an AI agent: TWiR does not accept LLM-written articles, and HN asks for
  human-written comments.

## Timeline

| When | What | File |
| --- | --- | --- |
| Week of 23 Nov | Prepare (checklist below); release **0.1.3**, so installers, the web editor and the changelog agree with what the posts say | `docs/releasing.md` |
| 25–27 Nov | Nothing public: US Thanksgiving (Thu 26 Nov) means a quiet HN and Reddit | |
| **Tue 1 Dec**, 14:00–15:00 CET (8–9 am US Eastern) | Publish the blog post; **Show HN** right after, first comment at once | `blog-post.md`, `show-hn.md` |
| Wed 2 Dec, ~15:00 CET | **r/rust** (also the TWiR Project/Tooling submission); egui *Show and tell* the same evening | `reddit-rust.md` |
| Thu 3 Dec | users.rust-lang.org; TWiR PR with the blog post (Observations/Thoughts) against the draft of 9 Dec – only if the post is your own text; optionally the two CFP issues | `reddit-rust.md`, `this-week-in-rust.md` |
| 4–7 Dec | Fix the bugs reported, reply to issues, release 0.1.4 if there were real ones | |
| **Tue 8 Dec**, ~15:00 CET | **r/PLC** (after checking its rules / asking the moderators) | `reddit-plc.md` |
| Tue 8 Dec, 8:00 CET | **LinkedIn** (English and German as two posts, or German first, English a day later) with the video | `linkedin.md` |
| Wed 9 Dec | **sps-forum.de** (E-CAD) | `german-forums.md` |
| Thu 10 Dec | **r/ElectricalEngineering**; mikrocontroller.net *Projekte & Code* | `reddit-plc.md`, `german-forums.md` |

If Show HN does not catch on (it often doesn't the first time), don't repost the same
day. The HN FAQ tolerates a small number of reposts of a story that got no significant
attention, and the moderators (hn@ycombinator.com) sometimes put such posts into the
second-chance pool; read the FAQ on the day.

## Prepare before 1 Dec

- [ ] Release 0.1.3 (`docs/releasing.md`), check the GitHub release assets and the
      AppImage update.
- [ ] Check that `https://editor.openecraft.org` serves the 0.1.3 build (e.g. *Help ›
      About OpenECraft* shows the version; the *Report a bug* button exists – the posts
      mention it).
- [ ] Load the web editor on a slow connection and on a phone / tablet; time the first
      load (the posts say 8–12 MB). Open `#wiring` on a tablet.
- [ ] Public servers: the parts server and the website must survive a few thousand
      visitors (static files are fine; check the parts server's rate limits and logs).
      Decide whether the project server sign-up is open; the posts only link the
      editor, the website and GitHub.
- [ ] Record the video and the README GIF (`video-script.md`); put the GIF or MP4 at
      the top of the README, under the first screenshot.
- [ ] README: the text says "fourteen example projects" but the table (and
      `sample::EXAMPLES`) has 16 – fix before launch. Add a short "Status" paragraph
      (what is solid, what is experimental), as the posts say.
- [ ] Add `CONTRIBUTING.md` (how to build, `AGENTS.md` conventions, how to file a bug,
      translations) – TWiR's CFP requires a link to contribution guidelines.
- [ ] Open 3–5 issues labelled `good first issue` (translation review, wasm size, a
      missing symbol set, a docs fix) so newcomers have a place to start.
- [ ] Issue templates for bug reports (the editor's *Report a bug* form fills one in).
- [ ] Rewrite `blog-post.md` in your own words and publish it ([blog-url]; a plain
      page on openecraft.org, dev.to or a personal blog – no paywall, no sign-up).
- [ ] Re-measure the numbers in the posts on the day (commands below).
- [ ] Decide what you say about the public servers' costs and the project's future
      (questions will come).
- [ ] Turn on GitHub notifications for issues; pin an issue "Feedback from the launch"
      if you want one place for general comments.

## Numbers used in the drafts (master ab8ee925, 2026-11-17)

| Fact | Value | Measured with |
| --- | --- | --- |
| First commit | 2026-10-19 | `git log --reverse --format=%ad --date=short \| head -1` |
| Commits | 468 | `git log --oneline \| wc -l` |
| Commits with a Claude co-author trailer | 338 (+ 17 authored by "Claude") | `git log -i --grep='co-authored-by: claude' --format=%H \| wc -l` |
| Releases | 0.1.0 (5 Nov), 0.1.1 (8 Nov), 0.1.2 (15 Nov) | `gh release list` |
| Workspace crates | 24 | `[workspace] members` in `Cargo.toml` |
| Rust | 924 files, ~254,000 lines; tests ~61,000 lines in 371 files; ~2,830 `#[test]` | `git ls-files '*.rs' \| xargs cat \| wc -l`, `grep -c '#\[test\]'` |
| `openecraft-core` | ~120,000 lines | `git ls-files crates/openecraft-core \| grep '\.rs$' \| xargs cat \| wc -l` |
| Commands of the built-in plugins | ~300 (316 `Spec` entries) | `grep 'Spec {'` in `crates/openecraft-api/src/builtin` |
| UI texts | ~8,900 lines in `required.tsv` | `cat crates/openecraft-i18n/src/catalog/*/required.tsv \| wc -l` |
| Languages | 20 (4 required) | `AGENTS.md`, `Lang::ALL` |
| Web editor wasm | 61 MB → 32 MB; 8 MB Brotli, 12 MB gzip; routing 7 % slower | `CHANGELOG.md` 0.1.3, commit 94beb414 (#219) |
| CJK font subsets | 299 KB (SC), 249 KB (JP), 111 KB (KR) | `crates/openecraft-fonts/fonts/` |
| Incremental analysis | 25 ms instead of 3.8 s (building services example) | `docs/eplan-gap-analysis.md` §7 |
| Example projects | 16 | `crates/openecraft-core/src/sample.rs` |
| IEC 60617 symbols | ~330 | `docs/features/symbols.md` |
| Issues | 39 opened, 35 closed, all by outside users; 181 merged PRs | `gh issue list --state all`, `gh pr list --state merged` |

## Handling comments

- Answer fast in the first hours, then at least twice a day for a week.
- Thank people for bug reports and turn each into a GitHub issue (link it in the reply);
  close the loop when it is fixed ("fixed in 0.1.4").
- Criticism of the AI use: don't argue. State what you did, what the tests cover, and
  invite them to find a bug. Never claim more human authorship than there is.
- "Why not contribute to X instead" / "EPLAN does this already": acknowledge, explain the
  niche (open, browser, scriptable), move on.
- Questions about safety / standards: the software comes without warranty, the
  calculations are aids, the engineer stays responsible.
- Feature requests: ask for the concrete workflow ("which report, which format, for which
  machine"), collect them in issues, don't promise dates.
- Don't post the same text to several subreddits on one day; don't ask for upvotes or
  stars anywhere; no sock puppets, no friends voting from a shared link.
- Keep a short log (where, when, link, notable feedback) – it feeds the next release
  notes and a follow-up post in a few months.

## After the launch

- Write a short follow-up issue or post with what you learned and fixed (r/rust likes
  "one month later" posts with numbers).
- Update `CHANGELOG.md` and the README with what changed because of the feedback.
