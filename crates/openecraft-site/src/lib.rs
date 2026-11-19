//! The pages of the OpenECraft website that are written by its build instead of by
//! hand (`docs/website.md`): the symbol library ([`symbols`]), the example projects
//! with their sheets ([`examples`]), the documentation ([`docs`]), the release notes
//! with their feed ([`news`]) and the pages written as content files of `website/content`
//! ([`content`]: questions and answers, how-tos, tools, comparisons). Every page gets
//! the frame of the site ([`frame`]) and is written in the languages it has.
//!
//! No UI and nothing of the browser: the pages come from the library crates (symbols,
//! examples and exports of `openecraft-core` and `openecraft-api`) and from the files
//! of the repository. `examples/site.rs` writes them into the page trunk builds.

pub mod build;
pub mod content;
pub mod docs;
pub mod examples;
pub mod frame;
pub mod markdown;
pub mod news;
pub mod symbols;
pub mod texts;
