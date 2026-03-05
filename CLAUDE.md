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

**Author workflow:** `tour init` → `tour add <files>` → `tour commit -m <msg>` (repeat) → `tour end -m <msg>`

**Reader workflow:** `tour start` → `tour next [n]` / `tour prev [n]` / `tour step <n>` / `tour reset`

## Architecture

Entry point is `main.rs`, which uses clap's derive macro to parse subcommands and dispatch to per-command modules.

**On-disk format** (stored in `.tour/` in the project being toured):
- `.tour/steps/<N>/` — one numbered directory per step (each step is a complete file snapshot)
- `.tour/steps/<N>/message` — the commit message for that step
- `.tour/session` — tracks current reader position as `STEP=<n>`
- `.tour/staged` — list of files staged for the next commit
- `.tour/info` — tour metadata (author, description, language, dates)
- `.tour/removed` — list of files marked for removal in the next commit
- `.tour/ended` — marker file indicating the tour is finalized

**Module layout:**
- `init.rs` — creates `.tour/` structure, collects tour metadata, updates `.gitignore`
- `add.rs` — stages files for the next commit; `get_staged()` reads the staged file list
- `unstage.rs` — removes files from staging
- `commit.rs` — commits staged files as a new step with carry-forward from previous step; only clears staging when staging was used
- `rm.rs` — marks files for removal in the next commit (skipped during carry-forward)
- `end.rs` — finalizes the tour (writes `.tour/ended` marker)
- `step.rs` — navigation: `next`, `prev`, `step_n`; handles file replacement, diff display, binary detection
- `reset.rs` — resets tour session and removes tracked files from working directory
- `status.rs` — shows current step position and staged files
- `list.rs` — lists all steps with their messages
- `info.rs` — tour metadata (author, description, language, dates)
- `utils.rs` — shared helpers: `require_tour`, `get_current_step`, `get_tour_step`, `copy_tree`, path validation
- `error.rs` — unified `TourError` enum with `Display` impl for user-facing messages; includes `CorruptedTour` for integrity checks

**Key design decisions:**
- Each step is a **complete snapshot** — `commit.rs` carries forward files from the previous step before overlaying new ones
- `TourError` is the unified error type across all modules, with `From<io::Error>` for automatic conversion
- `main()` catches errors and prints them with `Display` format (not `Debug`) for user-friendly messages
- Constants `TOUR_DIR` and `SESSION_PATH` are defined in `main.rs` and imported via `crate::`

**Testing:** Integration tests in `tests/integration.rs` use `tempfile` crate to create isolated tour directories and test via `Command`-based process spawning.
