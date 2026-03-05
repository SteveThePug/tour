use crate::error::TourError;
use crate::utils::{copy_tree, get_current_step, get_tour_step, require_tour};
use crate::SESSION_PATH;
use crate::TOUR_DIR;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const CYAN: &str = "\x1b[36m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

/// Jump to step `n` (1-based, as shown to the user).
pub fn step_n(n: u32) -> Result<(), TourError> {
    require_tour()?;
    let total = get_tour_step()?;

    if n < 1 || n > total {
        return Err(TourError::StepOutOfRange { step: n, total });
    }

    go_to_step(n - 1, total)
}

pub fn next(n: Option<u32>) -> Result<(), TourError> {
    require_tour()?;
    let total = get_tour_step()?;
    let delta = n.unwrap_or(1);
    let target = match get_current_step() {
        Some(c) => c.saturating_add(delta),
        None if delta > 0 => delta - 1,
        None => return Err(TourError::StepOutOfRange { step: 0, total }),
    };
    if target >= total {
        return Err(TourError::StepOutOfRange {
            step: target + 1,
            total,
        });
    }
    go_to_step(target, total)
}

pub fn prev(n: Option<u32>) -> Result<(), TourError> {
    require_tour()?;
    let total = get_tour_step()?;
    let delta = n.unwrap_or(1);
    let current = get_current_step().ok_or(TourError::StepOutOfRange { step: 0, total })?;
    let target = current
        .checked_sub(delta)
        .ok_or(TourError::StepOutOfRange { step: 0, total })?;
    go_to_step(target, total)
}

fn go_to_step(target: u32, total: u32) -> Result<(), TourError> {
    let cwd = std::env::current_dir()?;
    let tracked = get_tracked_files()?;
    let old_files = snapshot_tracked_files(&cwd, &tracked)?;

    remove_tracked_files(&cwd, &tracked)?;

    // Copy step contents into CWD (skipping the message file)
    let step_dir = Path::new(TOUR_DIR).join("steps").join(target.to_string());
    for entry in fs::read_dir(&step_dir)? {
        let entry = entry?;
        if entry.file_name() == "message" {
            continue;
        }
        copy_tree(&entry.path(), &cwd.join(entry.file_name()))?;
    }

    // Persist the new step
    fs::write(SESSION_PATH, format!("STEP={}", target))?;

    let new_files = snapshot_tracked_files(&cwd, &tracked)?;
    print_changes(&old_files, &new_files);

    let message = fs::read_to_string(step_dir.join("message")).unwrap_or_default();
    println!(
        "\n{BOLD}Step {}/{total}:{RESET} {}",
        target + 1,
        message.trim()
    );

    Ok(())
}

pub fn get_tracked_files() -> Result<BTreeSet<PathBuf>, io::Error> {
    let steps_dir = Path::new(TOUR_DIR).join("steps");
    let mut tracked = BTreeSet::new();

    if !steps_dir.exists() {
        return Ok(tracked);
    }

    for entry in fs::read_dir(&steps_dir)? {
        let entry = entry?;
        if entry.path().is_dir() {
            collect_step_files(&entry.path(), &entry.path(), &mut tracked)?;
        }
    }
    Ok(tracked)
}

pub fn remove_tracked_files(cwd: &Path, tracked: &BTreeSet<PathBuf>) -> Result<(), io::Error> {
    let mut dirs_to_check = BTreeSet::new();
    for relative in tracked {
        let full = cwd.join(relative);
        if full.is_file() {
            fs::remove_file(&full)?;
            if let Some(parent) = relative.parent()
                && parent != Path::new("")
            {
                dirs_to_check.insert(cwd.join(parent));
            }
        }
    }
    // Remove empty directories (deepest first)
    let mut dirs: Vec<_> = dirs_to_check.into_iter().collect();
    dirs.sort_by_key(|b| std::cmp::Reverse(b.components().count()));
    for dir in dirs {
        if dir.is_dir() && dir.read_dir().map(|mut d| d.next().is_none()).unwrap_or(false) {
            let _ = fs::remove_dir(&dir);
        }
    }
    Ok(())
}

fn collect_step_files(
    step_root: &Path,
    dir: &Path,
    files: &mut BTreeSet<PathBuf>,
) -> Result<(), io::Error> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let relative = path.strip_prefix(step_root).unwrap_or(&path).to_path_buf();
        if relative == Path::new("message") {
            continue;
        }
        if path.is_dir() {
            collect_step_files(step_root, &path, files)?;
        } else {
            files.insert(relative);
        }
    }
    Ok(())
}

/// Snapshots tracked files from root. Returns None for binary files (invalid UTF-8).
fn snapshot_tracked_files(
    root: &Path,
    tracked: &BTreeSet<PathBuf>,
) -> Result<BTreeMap<PathBuf, Option<String>>, io::Error> {
    let mut files = BTreeMap::new();
    for relative in tracked {
        let full = root.join(relative);
        if full.is_file() {
            files.insert(relative.clone(), fs::read_to_string(&full).ok());
        }
    }
    Ok(files)
}

fn print_changes(
    old: &BTreeMap<PathBuf, Option<String>>,
    new: &BTreeMap<PathBuf, Option<String>>,
) {
    for (path, new_content) in new {
        match old.get(path) {
            None => println!("{GREEN}  new:      {}{RESET}", path.display()),
            Some(old_content) if old_content != new_content => {
                println!("{CYAN}  modified: {}{RESET}", path.display());
                match (old_content, new_content) {
                    (Some(o), Some(n)) => print_diff(o, n),
                    _ => println!("    (binary file)"),
                }
            }
            _ => {}
        }
    }
    for path in old.keys() {
        if !new.contains_key(path) {
            println!("{RED}  deleted:  {}{RESET}", path.display());
        }
    }
}

fn print_diff(old: &str, new: &str) {
    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();

    let m = old_lines.len();
    let n = new_lines.len();

    // LCS table
    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for i in 1..=m {
        for j in 1..=n {
            if old_lines[i - 1] == new_lines[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    // Backtrack
    let mut ops: Vec<(char, &str)> = Vec::new();
    let (mut i, mut j) = (m, n);
    while i > 0 || j > 0 {
        if i > 0 && j > 0 && old_lines[i - 1] == new_lines[j - 1] {
            ops.push((' ', old_lines[i - 1]));
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || dp[i][j - 1] >= dp[i - 1][j]) {
            ops.push(('+', new_lines[j - 1]));
            j -= 1;
        } else {
            ops.push(('-', old_lines[i - 1]));
            i -= 1;
        }
    }
    ops.reverse();

    // Print with context: only show changed lines and up to 2 surrounding context lines
    const CTX: usize = 2;
    let changed: Vec<usize> = ops
        .iter()
        .enumerate()
        .filter(|(_, (op, _))| *op != ' ')
        .map(|(i, _)| i)
        .collect();

    if changed.is_empty() {
        return;
    }

    let mut visible = vec![false; ops.len()];
    for &ci in &changed {
        let lo = ci.saturating_sub(CTX);
        let hi = (ci + CTX + 1).min(ops.len());
        for item in visible.iter_mut().take(hi).skip(lo) {
            *item = true;
        }
    }

    let mut prev_visible = false;
    for (idx, (op, line)) in ops.iter().enumerate() {
        if !visible[idx] {
            if prev_visible {
                println!("    ...");
            }
            prev_visible = false;
            continue;
        }
        prev_visible = true;
        match op {
            '+' => println!("{GREEN}  + {line}{RESET}"),
            '-' => println!("{RED}  - {line}{RESET}"),
            _ => println!("    {line}"),
        }
    }
}
