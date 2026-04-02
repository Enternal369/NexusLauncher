// src/java.rs
use regex::Regex;
use std::env;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::process::Command;

#[derive(Debug, Clone)]
pub struct JavaInfo {
    pub path: PathBuf,
    pub major_version: u32,
    pub full_version: String,
}

/// Internal logic of parsing Java version strings
/// "1.8.0_382" -> 8
/// "17.0.8" -> 17
fn parse_major_version(version_str: &str) -> Option<u32> {
    // Split the string by . or _
    let parts: Vec<&str> = version_str.split(|c| c == '.' || c == '_').collect();
    if parts.is_empty() {
        return None;
    }

    if parts[0] == "1" && parts.len() > 1 {
        // Handle Java 8 and below (for example, 1.8 -> take 8)
        parts[1].parse().ok()
    } else {
        // Handle Java 9 and above (for example, 17.0 -> take 17)
        parts[0].parse().ok()
    }
}

/// Test the specified Java path and extract version information
pub async fn check_java_executable(java_path: &Path) -> Option<JavaInfo> {
    // Run java -version silently
    let output = Command::new(java_path)
        .arg("-version")
        .output()
        .await
        .ok()?;

    // Java's version information is always output to stderr
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Match something like: openjdk version "17.0.8" 2023-07-18
    let re = Regex::new(r#"version "([^"]+)""#).unwrap();

    if let Some(caps) = re.captures(&stderr) {
        let full_version = caps[1].to_string();
        if let Some(major_version) = parse_major_version(&full_version) {
            return Some(JavaInfo {
                path: java_path.to_path_buf(),
                major_version,
                full_version,
            });
        }
    }

    None
}

/// Deep scans the specified directory looking for bin/java executables.
async fn scan_jvm_directory(dir: &Path) -> Vec<JavaInfo> {
    let mut results = Vec::new();

    // Return early to avoid unnecessary nesting
    if !dir.is_dir() {
        return results;
    }

    tracing::debug!("Deep scanning directory: {}", dir.display());

    // Handle errors when reading the directory (e.g., permission denied, missing folder)
    let mut entries = match fs::read_dir(dir).await {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("Failed to read Java directory {}: {}", dir.display(), e);
            return results;
        }
    };

    // Use loop + match instead of `while let` for precise error control
    loop {
        match entries.next_entry().await {
            Ok(Some(entry)) => {
                let java_bin = entry.path().join("bin/java");
                // is_file() is safer and more strict than exists()
                if java_bin.is_file() {
                    if let Some(info) = check_java_executable(&java_bin).await {
                        results.push(info);
                    }
                }else {
                    tracing::debug!("Not a valid Java executable: {}", java_bin.display());
                }
            }
            Ok(None) => break, // Successfully read all entries, exit the loop
            Err(e) => {
                // Log the specific I/O error without failing silently or panicking
                tracing::warn!("I/O error while reading entries in {}: {}", dir.display(), e);
                break;
            }
        }
    }

    results
}

pub async fn scan_local_java_environments(custom_scan_path: Option<&Path>) -> Vec<JavaInfo> {
    tracing::info!("Scanning local Java environment...");
    let mut javas: Vec<JavaInfo> = Vec::new();

    // Helper closure: Check for duplicates (determined by absolute path or full version number)
    let mut add_if_new = |new_java: JavaInfo| {
        let is_duplicate = javas.iter().any(|j| {
            // If the path is the same, or the major version and detailed version are exactly the same, it is considered the same
            j.path == new_java.path
                || (j.major_version == new_java.major_version
                    && j.full_version == new_java.full_version)
        });

        if !is_duplicate {
            javas.push(new_java);
        }
    };

    // 1: Check the JAVA_HOME environment variable (the most respected option)
    if let Ok(java_home) = env::var("JAVA_HOME") {
        let java_bin = Path::new(&java_home).join("bin/java");
        if java_bin.exists() {
            tracing::debug!("Detected JAVA_HOME: {}", java_home);
            if let Some(info) = check_java_executable(&java_bin).await {
                add_if_new(info);
            }
        }
    }

    // 2: Check the default java command in the system PATH
    if let Some(info) = check_java_executable(Path::new("java")).await {
        add_if_new(info);
    }

    // 3: Scan a specifically designated location (for example, the runtimes folder that comes with the launcher)
    if let Some(scan_path) = custom_scan_path {
        for info in scan_jvm_directory(scan_path).await {
            add_if_new(info);
        }
    }

    // 4: Arch Linux / General Linux default installation path
    let jvm_dir = Path::new("/usr/lib/jvm");
    for info in scan_jvm_directory(jvm_dir).await {
        add_if_new(info);
    }
   

    tracing::info!("Scan complete, a total of {} unique Java environments were found", javas.len());
    javas
}
