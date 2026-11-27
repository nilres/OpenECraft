## What changes

<!-- For the user: what can they do now, what is fixed? Link the issue (Fixes #123). -->

## How it was tested

<!-- New tests, and what you tried in the editor / web editor / command line. -->

## Checklist

- [ ] I confirm the [clean room pledge](../CONTRIBUTING.md#the-clean-room-pledge): this contribution is my own work from public sources only and contains no code, symbols, data or texts copied or derived from a commercial product

- [ ] `cargo test --workspace` and both `cargo clippy` runs of [CONTRIBUTING.md](../CONTRIBUTING.md) pass without warnings
- [ ] New texts are in the catalog in English, German, French and Spanish
- [ ] New behaviour has a test (in `src/<module>/tests.rs`), a fix a regression test
- [ ] User-visible change: entry in `CHANGELOG.md`, documentation in `docs/` updated
