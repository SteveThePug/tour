# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```sh
cargo build          # build the project
cargo run -- <cmd>   # run a subcommand (e.g. cargo run -- init)
cargo test           # run all tests
cargo test <name>    # run a single test by name
cargo clippy         # lint
```

## What This Project Is

`tour` is a CLI tool for creating and navigating code tutorials as a series of snapshots. Authors create tours by committing file snapshots with explanations; readers step through them with `next`/`prev`.

**Author workflow:** `tour init` → `tour commit <files> -m <msg>` (repeat) → `tour end -m <msg>`

**Reader workflow:** `tour start` → `tour next [n]` / `tour prev [n]`

## Architecture

Entry point is `main.rs`, which uses clap's derive macro to parse subcommands and dispatch to per-command modules.

**On-disk format** (stored in `.tour/` in the project being toured):
- `.tour/steps/<N>/` — one numbered directory per step
- `.tour/session` — tracks current reader position as `STEP=<n>`

**Module layout:**
- `init.rs` — creates `.tour/steps/` and `.tour/session`
- `commit.rs` — validates files, then saves them as a new numbered step
- `end.rs` — finalizes the tour
- `next.rs` / `prev.rs` — advance/retreat the session step
- `utils.rs` — shared helpers: `copy_files`, `get_session_step`, `get_tour_step`, path validation
- `error.rs` — custom error types (currently `CommitError`)

Constants `TOUR_DIR` and `SESSION_PATH` are defined in `main.rs` and imported via `crate::`.

**Status:** Early development. `next`, `prev`, and `end` are stubbed. `commit` validates paths but hasn't yet written the step to disk. `utils::get_tour_step` has a dead `Ok(0)` after a `match` expression (unreachable code).
