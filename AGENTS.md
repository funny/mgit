# MGIT - Agent Development Guidelines

This document provides guidelines for agentic coding agents working on the MGIT multi-repository Git management tool.

## Project Overview

MGIT is a Rust workspace project with three main crates:
- `core`: Core library containing git operations and shared functionality
- `cli`: Command-line interface application  
- `gui`: Graphical user interface using egui framework

## Development Commands

### Building
```bash
# Build entire workspace
cargo build [--release]

# Build specific packages
cargo build -p mgit-core
cargo build -p mgit-cli
cargo build -p mgit-gui

# Quick compilation check
cargo check
```

### Testing
```bash
# Run all tests
cargo test

# Run tests for specific package
cargo test -p mgit-core
cargo test -p mgit-cli
cargo test -p mgit-gui

# Run tests with features (Docker Gitea integration)
cargo test --features=use_gitea

# Run specific test
cargo test test_name
cargo test -p mgit-core -- test_name
```

### Linting and Formatting
```bash
# Run clippy lints
cargo clippy

# Format code
cargo fmt

# Check for unused dependencies
cargo machete
```

## Code Style Guidelines

### Import Organization
Organize imports in this order:
1. Standard library: `use std::...`
2. External crates: `use anyhow::Context`, `use clap::Parser`
3. Internal modules: `use crate::core::git`, `use super::...`

Example:
```rust
use std::path::{Path, PathBuf};
use anyhow::{anyhow, Context};
use clap::Parser;
use crate::core::git::RemoteRef;
use crate::utils::progress::Progress;
```

### Naming Conventions
- **Structs**: `PascalCase` - `TomlRepo`, `SyncOptions`, `StyleMessage`
- **Functions**: `snake_case` - `sync_repo`, `get_current_commit`
- **Constants**: `SCREAMING_SNAKE_CASE` - `DEFAULT_WIDTH`, `GIT_VERSION`
- **Enums**: `PascalCase` with `snake_case` serde attributes - `#[serde(rename_all = "kebab-case")]`
- **Modules**: `snake_case` - `utils`, `commands`, `toml_settings`

### Error Handling Patterns

Use thiserror for custom error types and anyhow for error context:

```rust
// Custom error types
#[derive(Debug, Error)]
pub enum MgitError {
    #[error("Load config file failed!")]
    LoadConfigFailed,
    #[error("{0}")]
    DirNotFound(StyleMessage),
}

// Use anyhow for context
fn process_repo(path: &Path) -> MgitResult<()> {
    let config = load_config(path).context("Failed to load configuration")?;
    sync_repo(&config).context("Failed to sync repository")?;
    Ok(())
}
```

### Type Definitions

Use custom result types and be explicit with Option types:

```rust
// Custom result type
pub type MgitResult<T = StyleMessage, E = anyhow::Error> = Result<T, E>;

// Struct with serde support
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct TomlRepo {
    pub local: Option<String>,
    pub remote: Option<String>,
    pub branch: Option<String>,
    pub tag: Option<String>,
    pub commit: Option<String>,
    pub sparse: Option<Vec<String>>,
}
```

### CLI Command Pattern

Use clap derive macros for CLI commands:

```rust
#[derive(Debug, Hash, PartialEq, Eq, Clone, Default, Args)]
pub struct SyncCommand {
    pub path: Option<PathBuf>,
    #[arg(long, action = ArgAction::SetTrue)]
    stash: bool,
    #[arg(long, action = ArgAction::SetTrue)]
    hard: bool,
    #[arg(long, default_value = "4")]
    thread: usize,
}

impl CliCommand for SyncCommand {
    fn exec(self) -> MgitResult {
        // Implementation
    }
}
```

### Git Operations

Execute git commands externally rather than using libgit2:

```rust
use std::process::Command;

fn get_current_branch(path: &Path) -> MgitResult<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(path)
        .output()
        .context("Failed to execute git command")?;
    
    let branch = String::from_utf8(output.stdout)?.trim().to_string();
    Ok(branch)
}
```

### Progress Reporting

Implement the Progress trait for user feedback:

```rust
pub trait Progress {
    fn repos_start(&self, total: usize);
    fn repo_start(&self, repo_info: &RepoInfo, message: StyleMessage);
    fn repo_end(&self, repo_info: &RepoInfo, message: StyleMessage);
}
```

### GUI Development (egui)

Use immediate mode GUI patterns with egui:

```rust
impl eframe::App for Editor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("MGIT Repository Manager");
            // UI components
        });
    }
}
```

## Project-Specific Patterns

### Configuration Management
- Use TOML format for configuration files
- Implement `load()` and `serialize()` methods for config structs
- Handle configuration in dedicated `toml_settings` module

### Parallel Processing
- Use Rayon for parallel repository operations
- Configure thread count via CLI arguments
- Implement progress reporting for parallel operations

### Testing Strategy
- Write unit tests for core functionality
- Use Docker Gitea server for integration tests (`use_gitea` feature)
- Test cross-platform functionality

## Dependencies Management

All dependencies are managed in workspace root `Cargo.toml`. Key dependencies:
- `clap 4.0.8` - CLI parsing
- `egui 0.19.0` - GUI framework
- `anyhow 1` - Error handling
- `rayon 1.5` - Parallel processing
- `thiserror 1.0.4` - Custom errors

## Workspace Organization

```
mgit/
├── mgit-core/     # Core library (lib name: mgit)
├── mgit-shell/    # Shell abstraction layer
├── mgit-cli/      # CLI application (binary name: mgit)
├── mgit-gui/      # GUI application (binary name: mgit-gui)
└── tests/gitea-env/ # Docker test environment
```

## Release Process

Releases are automated via GitHub Actions with cross-compilation:
- CLI: Tags like `1.5.1` trigger release
- GUI: Tags like `gui-1.5.1` trigger release
- Supports Windows, Linux (x86_64/aarch64), macOS (x86_64/aarch64)

## Important Notes

- Project documentation is primarily in Chinese
- Git operations are executed externally, not via libgit2
- GUI uses immediate mode programming with egui
- All operations are synchronous (no tokio/async)
- Error context is crucial for debugging git operations
- Configuration files use kebab-case for serde serialization
