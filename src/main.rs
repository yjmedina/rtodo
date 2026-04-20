//! Binary entry point for `rtodo`.
//!
//! Parses CLI arguments via [`CLI`], then delegates to [`exec_project_cmd`]
//! or [`exec_task_cmd`] in [`dispatch`]. Workspace state is loaded before
//! each command and persisted afterward.

use std::io::IsTerminal;

use clap::Parser;
use rtodo::cli::{CLI, Commands};
use rtodo::dispatch::{exec_project_cmd, exec_task_cmd};
use rtodo::error::AppError;
use rtodo::workspace::Workspace;
/// Parses arguments and runs the matching command.
///
/// # Errors
/// Returns `Err` with a user-facing message on any failure.
fn run() -> Result<(), AppError> {
    // Disable ANSI color codes when stdout is not a TTY (e.g. piped to a file).
    // `IsTerminal` is a stable trait since Rust 1.70.
    if !std::io::stdout().is_terminal() {
        owo_colors::set_override(false);
    }

    let cli = CLI::parse();

    match cli.command {
        Commands::Init => {
            Workspace::init()?;
        }
        Commands::Project { command } => {
            let mut workspace = Workspace::load()?;
            exec_project_cmd(command, &mut workspace)?;
            workspace.save()?;
        }
        Commands::Task { command } => {
            let mut workspace = Workspace::load()?;
            let project = workspace
                .active_project()
                .ok_or(AppError::NoActiveProject)?;
            exec_task_cmd(command, project)?;
            // this could be improved to only save the project for performance.
            workspace.save()?;
        }
    };
    Ok(())
}

fn main() {
    if let Err(msg) = run() {
        eprintln!("  {} {}", rtodo::style::error_prefix(), msg);
        std::process::exit(1);
    }
}
