use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn setup_conflict_repo() -> std::path::PathBuf {
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let temp_dir = std::env::temp_dir().join(format!("git_gathuk_test_{}_{}", std::process::id(), id));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    let run_git = |args: &[&str]| {
        let status = Command::new("git")
            .current_dir(&temp_dir)
            .args(args)
            .status()
            .expect("Failed to run git");
        assert!(status.success(), "Git command failed: {:?}", args);
    };

    run_git(&["init", "-b", "main"]);
    run_git(&["config", "user.email", "test@example.com"]);
    run_git(&["config", "user.name", "Tester"]);

    fs::write(
        temp_dir.join("session.rs"),
        "pub struct Session {\n    id: u64,\n}\n",
    )
    .unwrap();
    run_git(&["add", "session.rs"]);
    run_git(&["commit", "-m", "Initial commit"]);

    run_git(&["checkout", "-b", "feature"]);
    fs::write(
        temp_dir.join("session.rs"),
        "pub struct Session {\n    id: u64,\n    remote_flag: bool,\n}\n",
    )
    .unwrap();
    run_git(&["commit", "-am", "Remote feature"]);

    run_git(&["checkout", "main"]);
    fs::write(
        temp_dir.join("session.rs"),
        "pub struct Session {\n    id: u64,\n    local_flag: bool,\n}\n",
    )
    .unwrap();
    run_git(&["commit", "-am", "Local main"]);

    // Attempt merge -> will fail with conflict
    let _ = Command::new("git")
        .current_dir(&temp_dir)
        .args(["merge", "feature"])
        .status();

    temp_dir
}

#[test]
fn test_git_conflict_resolution_flow() {
    let repo_dir = setup_conflict_repo();

    // Verify git diff filter U shows session.rs
    let output = Command::new("git")
        .current_dir(&repo_dir)
        .args(["diff", "--name-only", "--diff-filter=U"])
        .output()
        .unwrap();
    let conflicted = String::from_utf8_lossy(&output.stdout);
    assert!(conflicted.contains("session.rs"));

    // Check git show :2: and :3:
    let local_content = Command::new("git")
        .current_dir(&repo_dir)
        .args(["show", ":2:session.rs"])
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&local_content.stdout).contains("local_flag"));

    let remote_content = Command::new("git")
        .current_dir(&repo_dir)
        .args(["show", ":3:session.rs"])
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&remote_content.stdout).contains("remote_flag"));

    // Clean up
    let _ = fs::remove_dir_all(&repo_dir);
}
