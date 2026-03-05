use std::fs;
use std::path::Path;
use std::process::Command;

fn tour_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tour"))
}

fn setup_dir() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    dir
}

fn init_tour(dir: &Path) {
    let status = tour_cmd()
        .arg("init")
        .current_dir(dir)
        .stdin(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            if let Some(ref mut stdin) = child.stdin {
                writeln!(stdin, "test-author")?;
                writeln!(stdin, "test-desc")?;
                writeln!(stdin, "rust")?;
            }
            child.wait()
        })
        .expect("failed to run init");
    assert!(status.success(), "tour init failed");
}

fn create_test_file(dir: &Path, name: &str, content: &str) {
    let path = dir.join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&path, content).unwrap();
}

// -- init tests --

#[test]
fn test_init_creates_tour_dir() {
    let dir = setup_dir();
    init_tour(dir.path());

    assert!(dir.path().join(".tour").exists());
    assert!(dir.path().join(".tour/steps").exists());
    assert!(dir.path().join(".tour/session").exists());
    assert!(dir.path().join(".tour/info").exists());
}

#[test]
fn test_commands_fail_without_init() {
    let dir = setup_dir();

    let output = tour_cmd()
        .args(["commit", "foo.rs", "-m", "msg"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No tour found") || stderr.contains("tour init") || stderr.contains("NoTour"),
        "Expected helpful error, got: {}", stderr);
}

// -- add / commit tests --

#[test]
fn test_add_and_commit() {
    let dir = setup_dir();
    init_tour(dir.path());
    create_test_file(dir.path(), "hello.txt", "hello world");

    // Add
    let output = tour_cmd()
        .args(["add", "hello.txt"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(output.status.success(), "add failed: {}", String::from_utf8_lossy(&output.stderr));

    // Commit from staged
    let output = tour_cmd()
        .args(["commit", "-m", "first step"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(output.status.success(), "commit failed: {}", String::from_utf8_lossy(&output.stderr));

    // Step dir should exist with the file
    let step_dir = dir.path().join(".tour/steps/0");
    assert!(step_dir.exists());
    assert!(step_dir.join("hello.txt").exists());
    assert!(step_dir.join("message").exists());

    // Staged should be cleared
    let staged = fs::read_to_string(dir.path().join(".tour/staged")).unwrap();
    assert!(staged.trim().is_empty());
}

#[test]
fn test_add_deduplicates() {
    let dir = setup_dir();
    init_tour(dir.path());
    create_test_file(dir.path(), "a.txt", "content");

    // Add twice
    tour_cmd().args(["add", "a.txt"]).current_dir(dir.path()).output().unwrap();
    tour_cmd().args(["add", "a.txt"]).current_dir(dir.path()).output().unwrap();

    let staged = fs::read_to_string(dir.path().join(".tour/staged")).unwrap();
    let lines: Vec<&str> = staged.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(lines.len(), 1, "expected 1 staged file, got: {:?}", lines);
}

#[test]
fn test_commit_with_inline_files() {
    let dir = setup_dir();
    init_tour(dir.path());
    create_test_file(dir.path(), "main.rs", "fn main() {}");

    let output = tour_cmd()
        .args(["commit", "main.rs", "-m", "add main"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(dir.path().join(".tour/steps/0/main.rs").exists());
}

// -- end tests --

#[test]
fn test_end_tour() {
    let dir = setup_dir();
    init_tour(dir.path());
    create_test_file(dir.path(), "f.txt", "data");
    tour_cmd().args(["commit", "f.txt", "-m", "step"]).current_dir(dir.path()).output().unwrap();

    let output = tour_cmd()
        .args(["end", "-m", "done"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(output.status.success(), "end failed: {}", String::from_utf8_lossy(&output.stderr));
    assert!(dir.path().join(".tour/ended").exists());
}

#[test]
fn test_end_prevents_further_commits() {
    let dir = setup_dir();
    init_tour(dir.path());
    create_test_file(dir.path(), "f.txt", "data");
    tour_cmd().args(["commit", "f.txt", "-m", "step"]).current_dir(dir.path()).output().unwrap();
    tour_cmd().args(["end", "-m", "done"]).current_dir(dir.path()).output().unwrap();

    create_test_file(dir.path(), "g.txt", "more");
    let output = tour_cmd()
        .args(["commit", "g.txt", "-m", "nope"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
}

#[test]
fn test_end_without_steps_fails() {
    let dir = setup_dir();
    init_tour(dir.path());

    let output = tour_cmd()
        .args(["end", "-m", "empty"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
}

// -- step navigation tests --

#[test]
fn test_start_and_next_prev() {
    let dir = setup_dir();
    init_tour(dir.path());

    create_test_file(dir.path(), "f.txt", "step0");
    tour_cmd().args(["commit", "f.txt", "-m", "first"]).current_dir(dir.path()).output().unwrap();

    create_test_file(dir.path(), "f.txt", "step1");
    tour_cmd().args(["commit", "f.txt", "-m", "second"]).current_dir(dir.path()).output().unwrap();

    // Start
    let output = tour_cmd()
        .arg("start")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(output.status.success(), "start failed: {}", String::from_utf8_lossy(&output.stderr));

    let content = fs::read_to_string(dir.path().join("f.txt")).unwrap();
    assert_eq!(content, "step0");

    // Next
    let output = tour_cmd()
        .arg("next")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(output.status.success(), "next failed: {}", String::from_utf8_lossy(&output.stderr));

    let content = fs::read_to_string(dir.path().join("f.txt")).unwrap();
    assert_eq!(content, "step1");

    // Prev
    let output = tour_cmd()
        .arg("prev")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(output.status.success(), "prev failed: {}", String::from_utf8_lossy(&output.stderr));

    let content = fs::read_to_string(dir.path().join("f.txt")).unwrap();
    assert_eq!(content, "step0");
}

#[test]
fn test_step_n_jumps_to_step() {
    let dir = setup_dir();
    init_tour(dir.path());

    create_test_file(dir.path(), "f.txt", "v0");
    tour_cmd().args(["commit", "f.txt", "-m", "s0"]).current_dir(dir.path()).output().unwrap();
    create_test_file(dir.path(), "f.txt", "v1");
    tour_cmd().args(["commit", "f.txt", "-m", "s1"]).current_dir(dir.path()).output().unwrap();
    create_test_file(dir.path(), "f.txt", "v2");
    tour_cmd().args(["commit", "f.txt", "-m", "s2"]).current_dir(dir.path()).output().unwrap();

    // Start at step 1, then jump to step 3 (1-based)
    tour_cmd().arg("start").current_dir(dir.path()).output().unwrap();
    let output = tour_cmd()
        .args(["step", "3"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(output.status.success(), "step 3 failed: {}", String::from_utf8_lossy(&output.stderr));

    let content = fs::read_to_string(dir.path().join("f.txt")).unwrap();
    assert_eq!(content, "v2");
}

#[test]
fn test_step_out_of_range() {
    let dir = setup_dir();
    init_tour(dir.path());

    create_test_file(dir.path(), "f.txt", "v0");
    tour_cmd().args(["commit", "f.txt", "-m", "s0"]).current_dir(dir.path()).output().unwrap();
    tour_cmd().arg("start").current_dir(dir.path()).output().unwrap();

    // Step 0 is out of range (1-based, only 1 step)
    let output = tour_cmd()
        .args(["step", "0"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(!output.status.success());

    // Step 2 is out of range (only 1 step)
    let output = tour_cmd()
        .args(["step", "2"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
}

// -- unstage tests --

#[test]
fn test_unstage() {
    let dir = setup_dir();
    init_tour(dir.path());
    create_test_file(dir.path(), "a.txt", "aaa");
    create_test_file(dir.path(), "b.txt", "bbb");

    tour_cmd().args(["add", "a.txt", "b.txt"]).current_dir(dir.path()).output().unwrap();

    let output = tour_cmd()
        .args(["unstage", "a.txt"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(output.status.success(), "unstage failed: {}", String::from_utf8_lossy(&output.stderr));

    let staged = fs::read_to_string(dir.path().join(".tour/staged")).unwrap();
    assert!(!staged.contains("a.txt"), "a.txt should be unstaged");
    assert!(staged.contains("b.txt"), "b.txt should still be staged");
}

// -- list tests --

#[test]
fn test_list_steps() {
    let dir = setup_dir();
    init_tour(dir.path());

    create_test_file(dir.path(), "f.txt", "v0");
    tour_cmd().args(["commit", "f.txt", "-m", "first step"]).current_dir(dir.path()).output().unwrap();
    create_test_file(dir.path(), "f.txt", "v1");
    tour_cmd().args(["commit", "f.txt", "-m", "second step"]).current_dir(dir.path()).output().unwrap();

    let output = tour_cmd()
        .arg("list")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("first step"), "expected 'first step' in: {}", stdout);
    assert!(stdout.contains("second step"), "expected 'second step' in: {}", stdout);
}

// -- carry-forward tests --

#[test]
fn test_files_carry_forward() {
    let dir = setup_dir();
    init_tour(dir.path());

    create_test_file(dir.path(), "a.txt", "aaa");
    create_test_file(dir.path(), "b.txt", "bbb");
    tour_cmd().args(["commit", "a.txt", "b.txt", "-m", "step 1"]).current_dir(dir.path()).output().unwrap();

    // Second step only modifies a.txt
    create_test_file(dir.path(), "a.txt", "aaa modified");
    tour_cmd().args(["commit", "a.txt", "-m", "step 2"]).current_dir(dir.path()).output().unwrap();

    // b.txt should be carried forward to step 2
    assert!(dir.path().join(".tour/steps/1/b.txt").exists(), "b.txt should be carried forward");

    // Navigate to step 2 and verify both files exist
    tour_cmd().arg("start").current_dir(dir.path()).output().unwrap();
    tour_cmd().arg("next").current_dir(dir.path()).output().unwrap();

    let a = fs::read_to_string(dir.path().join("a.txt")).unwrap();
    assert_eq!(a, "aaa modified");
    let b = fs::read_to_string(dir.path().join("b.txt")).unwrap();
    assert_eq!(b, "bbb");
}

// -- init guard tests --

#[test]
fn test_init_twice_fails() {
    let dir = setup_dir();
    init_tour(dir.path());

    let output = tour_cmd()
        .arg("init")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(!output.status.success(), "second init should fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("already exists"), "expected 'already exists', got: {}", stderr);
}

// -- gitignore tests --

#[test]
fn test_init_creates_gitignore() {
    let dir = setup_dir();
    init_tour(dir.path());

    let gitignore = fs::read_to_string(dir.path().join(".gitignore")).unwrap();
    assert!(gitignore.contains(".tour/session"), "gitignore should contain .tour/session");
    assert!(gitignore.contains(".tour/staged"), "gitignore should contain .tour/staged");
}

// -- add validation tests --

#[test]
fn test_add_nonexistent_file() {
    let dir = setup_dir();
    init_tour(dir.path());

    let output = tour_cmd()
        .args(["add", "nonexistent.txt"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not found") || stderr.contains("No such file") || stderr.contains("FileNotFound"),
        "Expected file not found error, got: {}", stderr);
}

// -- subdirectory tests --

#[test]
fn test_subdirectory_files() {
    let dir = setup_dir();
    init_tour(dir.path());

    create_test_file(dir.path(), "src/main.rs", "fn main() {}");
    create_test_file(dir.path(), "src/lib.rs", "pub fn hello() {}");

    let output = tour_cmd()
        .args(["commit", "src/main.rs", "src/lib.rs", "-m", "add source"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(output.status.success(), "commit failed: {}", String::from_utf8_lossy(&output.stderr));

    // Verify step directory has subdirectory structure
    assert!(dir.path().join(".tour/steps/0/src/main.rs").exists());
    assert!(dir.path().join(".tour/steps/0/src/lib.rs").exists());

    // Modify one file and commit step 2
    create_test_file(dir.path(), "src/main.rs", "fn main() { println!(\"hi\"); }");
    let output = tour_cmd()
        .args(["commit", "src/main.rs", "-m", "modify main"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(output.status.success());

    // lib.rs should be carried forward
    assert!(dir.path().join(".tour/steps/1/src/lib.rs").exists());

    // Start at step 1 and verify
    tour_cmd().arg("start").current_dir(dir.path()).output().unwrap();
    let content = fs::read_to_string(dir.path().join("src/main.rs")).unwrap();
    assert_eq!(content, "fn main() {}");
    let content = fs::read_to_string(dir.path().join("src/lib.rs")).unwrap();
    assert_eq!(content, "pub fn hello() {}");

    // Go to step 2 and verify modified file + carried-forward file
    tour_cmd().arg("next").current_dir(dir.path()).output().unwrap();
    let content = fs::read_to_string(dir.path().join("src/main.rs")).unwrap();
    assert_eq!(content, "fn main() { println!(\"hi\"); }");
    let content = fs::read_to_string(dir.path().join("src/lib.rs")).unwrap();
    assert_eq!(content, "pub fn hello() {}");

    // Go back to step 1 and verify cleanup
    tour_cmd().arg("prev").current_dir(dir.path()).output().unwrap();
    let content = fs::read_to_string(dir.path().join("src/main.rs")).unwrap();
    assert_eq!(content, "fn main() {}");
}

// -- status tests --

#[test]
fn test_status_shows_info() {
    let dir = setup_dir();
    init_tour(dir.path());

    create_test_file(dir.path(), "f.txt", "data");
    tour_cmd().args(["commit", "f.txt", "-m", "step"]).current_dir(dir.path()).output().unwrap();

    let output = tour_cmd()
        .arg("status")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("1 steps"), "expected step count in: {}", stdout);
    assert!(stdout.contains("not started"), "expected 'not started' in: {}", stdout);
}
