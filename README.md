# Nexus Launcher

```text
[INFO] Nexus Launcher Starting...
[INFO] Workspace initialized
[INFO] Target Version: 1.20.1
[INFO] Player Name: AuroBreeze
[INFO] No valid cached Java 17 found. Starting scan...
[INFO] Found matching Java 17: /usr/lib/jvm/java-17-openjdk/bin/java
[INFO] All core components of 1.20.1 are ready!
[INFO] 🚀 Game launched successfully! Process PID: 4092
```

A command-line Minecraft launcher written in **Rust**.

Nexus Launcher is designed to be lightweight and operates entirely from the terminal, avoiding the resource overhead of a graphical user interface. It handles asynchronous game asset downloads and automatically sets up the required Java environment to launch the game.

---

## ✨ Core Features

* **Concurrent Downloading:** Uses `tokio` and `reqwest` to download game assets and libraries asynchronously, with connection limits to avoid server blocks.
* **Java Management:** Automatically detects the required Java version for the game. If it is not found locally, the launcher downloads and extracts the appropriate JRE via the Adoptium API.
* **Configuration Saving:** Stores validated Java paths and launcher settings in a `nexus_config.toml` file to reduce scanning time on subsequent startups.
* **Version Isolation:** Keeps game versions separated. Version-specific files like `saves` and `mods` are stored in their own directories, while common `assets` and `libraries` are shared globally.

## 📂 Workspace Layout

Nexus constructs a clean, logically separated environment upon first boot:

```text
~/.minecraft/
├── nexus_config.toml  -> Persistent launcher state & cache
├── assets/            -> Globally shared game assets
├── libraries/         -> Globally shared dependency jars
├── runtimes/          -> Auto-managed Java JREs
└── versions/
    └── 1.20.1/        -> Isolated sandbox (saves, mods, options)
```

## 🚀 Quick Start

Ensure you have `rustc` and `cargo` installed on your system.

**Launch with default configuration:**
```bash
cargo run
```

**Launch with custom arguments:**
```bash
cargo run -- --username AuroBreeze --game-version 1.8.9 --memory 4096
```

**Force an environment re-evaluation (bypasses TOML cache):**
```bash
cargo run -- --force-java-scan
```

## 📜 Philosophy

* **Zero GUI Overhead:** Maximize system memory and CPU for the game itself, not the launcher.
* **Transparent Execution:** What you see in the terminal log is exactly what is happening under the hood.
* **Idempotent Operations:** Running the launcher multiple times safely verifies existing files without redundant downloads.

