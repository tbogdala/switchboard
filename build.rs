use std::{env, process::Command};

fn main() {
    // First try to get from environment variables (set by Trunk)
    let git_branch = env::var("GIT_BRANCH").unwrap_or_else(|_| {
        // Fallback to running git command directly
        Command::new("git")
            .args(&["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
            .unwrap_or_else(|_| "unknown".to_string())
    });

    let git_hash = env::var("GIT_HASH").unwrap_or_else(|_| {
        // Fallback to running git command directly
        Command::new("git")
            .args(&["rev-parse", "--short", "HEAD"])
            .output()
            .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
            .unwrap_or_else(|_| "unknown".to_string())
    });

    let short_hash = git_hash.trim().chars().take(7).collect::<String>();
    println!("cargo:rustc-env=GIT_HASH={}", short_hash);

    let branch = git_branch.trim();
    println!("cargo:rustc-env=GIT_BRANCH={}", branch);
}
