use std::{env, process::Command, path::Path, fs};

fn main() {
    // Create a file to track the current git state
    let out_dir = env::var("OUT_DIR").unwrap();
    let git_info_path = Path::new(&out_dir).join(".git_info");

    // Get current git info
    let git_branch = get_git_branch();
    let git_hash = get_git_hash();
    let short_hash = git_hash.trim().chars().take(7).collect::<String>();

    // Only update if git info has changed
    if needs_update(&git_info_path, &git_branch, &short_hash) {
        println!("cargo:rustc-env=GIT_HASH={}", short_hash);
        println!("cargo:rustc-env=GIT_BRANCH={}", git_branch.trim());

        // Write current git info to file for future comparison
        fs::write(git_info_path, format!("{}\n{}", git_branch, short_hash)).unwrap();
    }

    // Tell Cargo to rerun if the git info changes
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/heads/");
}

fn get_git_branch() -> String {
    env::var("GIT_BRANCH").unwrap_or_else(|_| {
        Command::new("git")
            .args(&["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
            .unwrap_or_else(|_| "unknown".to_string())
    })
}

fn get_git_hash() -> String {
    env::var("GIT_HASH").unwrap_or_else(|_| {
        Command::new("git")
            .args(&["rev-parse", "--short", "HEAD"])
            .output()
            .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
            .unwrap_or_else(|_| "unknown".to_string())
    })
}

fn needs_update(git_info_path: &Path, current_branch: &str, current_hash: &str) -> bool {
    if !git_info_path.exists() {
        return true;
    }

    if let Ok(contents) = fs::read_to_string(git_info_path) {
        let parts: Vec<&str> = contents.trim().split('\n').collect();
        if parts.len() == 2 {
            return parts[0] != current_branch || parts[1] != current_hash;
        }
    }

    true
}