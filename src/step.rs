use crate::error::{IoResultExt, TourError};
use crate::style::{bold, cyan, green, red, reset};
use crate::utils::{get_current_step, get_tour_step, require_tour};
use crate::SESSION_PATH;
use crate::TOUR_DIR;
use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Jump to step `n` (1-based, as shown to the user).
pub fn step_n(n: u32) -> Result<(), TourError> {
    require_tour()?;
    let total = get_tour_step()?;

    if total == 0 {
        return Err(TourError::NoStepsToNavigate);
    }
    if n < 1 || n > total {
        return Err(TourError::StepOutOfRange { step: n, total });
    }

    go_to_step(n - 1, total)
}

pub fn next(n: Option<u32>) -> Result<(), TourError> {
    require_tour()?;
    let total = get_tour_step()?;
    if total == 0 {
        return Err(TourError::NoStepsToNavigate);
    }
    let delta = n.unwrap_or(1);
    let last = total - 1;
    match get_current_step() {
        Some(current) if current >= last => {
            println!("Already at the last step ({total}/{total}).");
            Ok(())
        }
        Some(current) => go_to_step(current.saturating_add(delta).min(last), total),
        None => go_to_step(delta.saturating_sub(1).min(last), total),
    }
}

pub fn prev(n: Option<u32>) -> Result<(), TourError> {
    require_tour()?;
    let total = get_tour_step()?;
    if total == 0 {
        return Err(TourError::NoStepsToNavigate);
    }
    let delta = n.unwrap_or(1);
    let current = get_current_step().ok_or(TourError::NotStarted)?;
    if current == 0 {
        println!("Already at the first step (1/{total}).");
        return Ok(());
    }
    go_to_step(current.saturating_sub(delta), total)
}

enum FileChange {
    Added,
    Modified { old: Option<String>, new: Option<String> },
    Removed,
}

fn go_to_step(target: u32, total: u32) -> Result<(), TourError> {
    let cwd = std::env::current_dir()?;
    let steps_dir = Path::new(TOUR_DIR).join("steps");
    let target_dir = steps_dir.join(target.to_string());

    let mut target_files = BTreeSet::new();
    collect_step_files(&target_dir, &target_dir, &mut target_files)?;

    // Files the tour owns in the working directory: the current step's set if a
    // session exists, otherwise every file any step tracks (conservative).
    let current_files = match get_current_step() {
        Some(current) if steps_dir.join(current.to_string()).is_dir() => {
            let dir = steps_dir.join(current.to_string());
            let mut set = BTreeSet::new();
            collect_step_files(&dir, &dir, &mut set)?;
            set
        }
        _ => get_tracked_files()?,
    };

    // Work out the full change set before touching the working directory
    let mut changes: Vec<(PathBuf, FileChange)> = Vec::new();
    let mut to_install: Vec<PathBuf> = Vec::new();
    for relative in &target_files {
        let wd_file = cwd.join(relative);
        let step_file = target_dir.join(relative);
        if !wd_file.is_file() {
            changes.push((relative.clone(), FileChange::Added));
            to_install.push(relative.clone());
        } else if !files_equal(&wd_file, &step_file)? {
            changes.push((
                relative.clone(),
                FileChange::Modified {
                    old: fs::read_to_string(&wd_file).ok(),
                    new: fs::read_to_string(&step_file).ok(),
                },
            ));
            to_install.push(relative.clone());
        }
    }
    let to_delete: BTreeSet<PathBuf> = current_files
        .difference(&target_files)
        .cloned()
        .collect();
    for relative in &to_delete {
        if cwd.join(relative).is_file() {
            changes.push((relative.clone(), FileChange::Removed));
        }
    }

    // Install only the files that differ. Always copy — never link into the
    // snapshot store — so user edits can't write through to step data.
    for relative in &to_install {
        let dest = cwd.join(relative);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        if dest.exists() {
            fs::remove_file(&dest)?;
        }
        fs::copy(target_dir.join(relative), &dest)
            .context(format!("failed to install {}", relative.display()))?;
    }
    remove_tracked_files(&cwd, &to_delete)?;

    fs::write(SESSION_PATH, format!("STEP={}", target))
        .context("failed to save session")?;

    print_changes(&changes);

    let message = fs::read_to_string(target_dir.join("message"))
        .unwrap_or_else(|_| "(no message)".into());
    println!(
        "\n{}Step {}/{total}:{} {}",
        bold(),
        target + 1,
        reset(),
        message.trim()
    );

    Ok(())
}

/// Fast equality check: same inode, then size, then contents.
fn files_equal(a: &Path, b: &Path) -> Result<bool, io::Error> {
    let meta_a = fs::metadata(a)?;
    let meta_b = fs::metadata(b)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if meta_a.dev() == meta_b.dev() && meta_a.ino() == meta_b.ino() {
            return Ok(true);
        }
    }
    if meta_a.len() != meta_b.len() {
        return Ok(false);
    }
    Ok(fs::read(a)? == fs::read(b)?)
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

fn print_changes(changes: &[(PathBuf, FileChange)]) {
    for (path, change) in changes {
        match change {
            FileChange::Added => {
                println!("{}  new:      {}{}", green(), path.display(), reset())
            }
            FileChange::Modified { old, new } => {
                println!("{}  modified: {}{}", cyan(), path.display(), reset());
                match (old, new) {
                    (Some(o), Some(n)) => print_diff(o, n),
                    _ => println!("    (binary file)"),
                }
            }
            FileChange::Removed => {
                println!("{}  deleted:  {}{}", red(), path.display(), reset())
            }
        }
    }
}

fn print_diff(old: &str, new: &str) {
    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();

    // Trim the common prefix and suffix so the LCS table only covers the
    // changed middle of the file.
    let mut start = 0;
    while start < old_lines.len()
        && start < new_lines.len()
        && old_lines[start] == new_lines[start]
    {
        start += 1;
    }
    let (mut old_end, mut new_end) = (old_lines.len(), new_lines.len());
    while old_end > start && new_end > start && old_lines[old_end - 1] == new_lines[new_end - 1] {
        old_end -= 1;
        new_end -= 1;
    }

    let old_mid = &old_lines[start..old_end];
    let new_mid = &new_lines[start..new_end];
    let m = old_mid.len();
    let n = new_mid.len();

    if m > 1000 || n > 1000 {
        println!("    (file changed)");
        return;
    }

    // LCS table over the changed middle
    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for i in 1..=m {
        for j in 1..=n {
            if old_mid[i - 1] == new_mid[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    // Backtrack
    let mut mid_ops: Vec<(char, &str)> = Vec::new();
    let (mut i, mut j) = (m, n);
    while i > 0 || j > 0 {
        if i > 0 && j > 0 && old_mid[i - 1] == new_mid[j - 1] {
            mid_ops.push((' ', old_mid[i - 1]));
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || dp[i][j - 1] >= dp[i - 1][j]) {
            mid_ops.push(('+', new_mid[j - 1]));
            j -= 1;
        } else {
            mid_ops.push(('-', old_mid[i - 1]));
            i -= 1;
        }
    }
    mid_ops.reverse();

    let mut ops: Vec<(char, &str)> = old_lines[..start].iter().map(|l| (' ', *l)).collect();
    ops.extend(mid_ops);
    ops.extend(old_lines[old_end..].iter().map(|l| (' ', *l)));

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
            '+' => println!("{}  + {line}{}", green(), reset()),
            '-' => println!("{}  - {line}{}", red(), reset()),
            _ => println!("    {line}"),
        }
    }
}
