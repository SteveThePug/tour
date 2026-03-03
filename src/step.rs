use crate::utils::get_tour_step;
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

pub fn step_n(n: i32) -> Result<(), io::Error> {
    let total = get_tour_step()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

    if n < 0 || n >= total as i32 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Step {} is out of range (0-{})", n, total - 1),
        ));
    }

    step(n - current_step())
}

pub fn next(n: Option<i32>) -> Result<(), io::Error> {
    step(n.unwrap_or(1))
}

pub fn prev(n: Option<i32>) -> Result<(), io::Error> {
    step(-n.unwrap_or(1))
}

/// Returns the current step as a signed integer.
/// Returns -1 when the session has no step yet (reader hasn't started).
fn current_step() -> i32 {
    fs::read_to_string(SESSION_PATH)
        .ok()
        .and_then(|s| {
            s.split("STEP=")
                .nth(1)
                .and_then(|v| v.trim().parse::<i32>().ok())
        })
        .unwrap_or(-1)
}

fn step(delta: i32) -> Result<(), io::Error> {
    let current = current_step();
    let total = get_tour_step()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

    let new_step = current + delta;
    if new_step < 0 || new_step >= total as i32 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Step {} is out of range (0-{})", new_step, total - 1),
        ));
    }
    let new_step = new_step as u32;

    let cwd = std::env::current_dir()?;
    let tracked = get_tracked_files()?;
    let old_files = snapshot_tracked_files(&cwd, &tracked)?;

    // Remove only tracked files from CWD
    for relative in &tracked {
        let full = cwd.join(relative);
        if full.is_file() {
            fs::remove_file(&full)?;
        }
    }

    // Copy step contents into CWD (skipping the message file)
    let step_dir = Path::new(TOUR_DIR).join("steps").join(new_step.to_string());
    for entry in fs::read_dir(&step_dir)? {
        let entry = entry?;
        if entry.file_name() == "message" {
            continue;
        }
        copy_into(&entry.path(), &cwd.join(entry.file_name()))?;
    }

    // Persist the new step
    fs::write(SESSION_PATH, format!("STEP={}", new_step))?;

    let new_files = snapshot_tracked_files(&cwd, &tracked)?;
    print_changes(&old_files, &new_files);

    let message = fs::read_to_string(step_dir.join("message")).unwrap_or_default();
    println!("\n{BOLD}Step {new_step}/{total}:{RESET} {}", message.trim());

    Ok(())
}

fn snapshot_tracked_files(root: &Path, tracked: &BTreeSet<PathBuf>) -> Result<BTreeMap<PathBuf, String>, io::Error> {
    let mut files = BTreeMap::new();
    for relative in tracked {
        let full = root.join(relative);
        if full.is_file() {
            let content = fs::read_to_string(&full).unwrap_or_default();
            files.insert(relative.clone(), content);
        }
    }
    Ok(files)
}

fn get_tracked_files() -> Result<BTreeSet<PathBuf>, io::Error> {
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

fn print_changes(old: &BTreeMap<PathBuf, String>, new: &BTreeMap<PathBuf, String>) {
    for (path, new_content) in new {
        match old.get(path) {
            None => println!("{GREEN}  new:      {}{RESET}", path.display()),
            Some(old_content) if old_content != new_content => {
                println!("{CYAN}  modified: {}{RESET}", path.display());
                print_diff(old_content, new_content);
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
        for v in lo..hi {
            visible[v] = true;
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

fn copy_into(src: &Path, dest: &Path) -> Result<(), io::Error> {
    if src.is_dir() {
        fs::create_dir_all(dest)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            copy_into(&entry.path(), &dest.join(entry.file_name()))?;
        }
    } else {
        fs::copy(src, dest)?;
    }
    Ok(())
}
