use std::process::Command;

fn main() {
    // Get git version information
    let version = if let Ok(output) = Command::new("git")
        .args(["describe", "--tags", "--always", "--dirty"])
        .output()
    {
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    } else {
        // Fallback to cargo version if git not available
        env!("CARGO_PKG_VERSION").to_string()
    };

    println!("cargo:rustc-env=GIT_VERSION={}", version);
    println!("cargo:rerun-if-changed=.git/HEAD");
}
