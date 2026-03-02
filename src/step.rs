use crate::utils::get_tour_step;
use crate::SESSION_PATH;
use crate::TOUR_DIR;
use std::collections::BTreeMap;
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
    let old_files = snapshot_files(&cwd)?;

    // Clear CWD except .tour/
    for entry in fs::read_dir(&cwd)? {
        let entry = entry?;
        if entry.file_name() == ".tour" {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            fs::remove_dir_all(&path)?;
        } else {
            fs::remove_file(&path)?;
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

    let new_files = snapshot_files(&cwd)?;
    print_changes(&old_files, &new_files);

    let message = fs::read_to_string(step_dir.join("message")).unwrap_or_default();
    println!("\n{BOLD}Step {new_step}/{total}:{RESET} {}", message.trim());

    Ok(())
}

fn snapshot_files(root: &Path) -> Result<BTreeMap<PathBuf, String>, io::Error> {
    let mut files = BTreeMap::new();
    collect_files(root, root, &mut files)?;
    Ok(files)
}

fn collect_files(
    root: &Path,
    dir: &Path,
    files: &mut BTreeMap<PathBuf, String>,
) -> Result<(), io::Error> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_name() == ".tour" {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            collect_files(root, &path, files)?;
        } else {
            let relative = path.strip_prefix(root).unwrap_or(&path).to_path_buf();
            let content = fs::read_to_string(&path).unwrap_or_default();
            files.insert(relative, content);
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
