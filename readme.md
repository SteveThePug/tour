# tour

A CLI tool for creating and navigating code tutorials as a series of file snapshots.

## Install

```sh
cargo install --path .
```

## Quick Start

### Creating a tour

```sh
# Initialize a new tour in the current directory
tour init

# Stage and commit files as steps
tour add src/main.rs Cargo.toml
tour commit -m "After running cargo init we get our template"

# Or commit with inline file arguments
tour commit src/lib.rs -m "In lib.rs we add some functions"

tour commit src/main.rs -m "We import our newly made function from lib.rs"

# Finalize the tour
tour end -m "Now your tour is complete and you can use rust modules!"
```

### Taking a tour

```sh
tour start
#   new:      Cargo.toml
#   new:      src/main.rs
#
# Step 1/3: After running cargo init we get our template

tour next
#   new:      src/lib.rs
#
# Step 2/3: In lib.rs we add some functions

tour prev
#   deleted:  src/lib.rs
#
# Step 1/3: After running cargo init we get our template

tour next 2
#   new:      src/lib.rs
#   modified: src/main.rs
#   + use lib::my_function;
#
# Step 3/3: We import our newly made function from lib.rs

tour step 1    # jump to any step by number
```

## Commands

### Author workflow

| Command | Description |
|---------|-------------|
| `tour init` | Set up a new tour in the current directory |
| `tour add <files...>` | Stage files for the next commit |
| `tour unstage <files...>` | Remove files from staging |
| `tour commit -m <msg>` | Commit staged files as a new step |
| `tour commit <files...> -m <msg>` | Stage and commit files in one step |
| `tour end -m <msg>` | Finalize the tour |

### Reader workflow

| Command | Description |
|---------|-------------|
| `tour start` | Load the first step |
| `tour next [n]` | Advance n steps (default 1) |
| `tour prev [n]` | Go back n steps (default 1) |
| `tour step <n>` | Jump to step n |

### Other

| Command | Description |
|---------|-------------|
| `tour info` | Show tour metadata |
| `tour status` | Show current step and staged files |
| `tour list` | List all steps with messages |
| `tour help` | Show help message |

## How it works

Each step is stored as a complete file snapshot in `.tour/steps/<N>/`. When navigating between steps, `tour` replaces tracked files in your working directory and shows a diff of what changed — new files, modified files (with line-level diffs), and deleted files.

The `.tour/` directory contains:
- `steps/` — numbered directories, each holding the full file state for that step plus a `message` file
- `session` — tracks the reader's current position
- `info` — tour metadata (author, description, language, dates)
- `staged` — files staged for the next commit
- `ended` — marker indicating the tour is finalized
