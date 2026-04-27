//! Binary entry point for `rtodo`.
//!
//! Parses CLI arguments via [`CLI`], then delegates to [`dispatch_project`]
//! or [`dispatch_task`] in [`dispatch`]. Workspace state is loaded before
//! each command and persisted afterward.

use std::io::IsTerminal;

use clap::Parser;
use rtodo::cli::{CLI, Commands};
use rtodo::dispatch::{dispatch_project, dispatch_task};
use rtodo::error::AppError;
use rtodo::tui;
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
            dispatch_project(command, &mut workspace)?;
            workspace.save()?;
        }
        Commands::Task { command } => {
            let mut workspace = Workspace::load()?;
            let project = workspace.active_project()?;
            dispatch_task(command, project)?;
            // this could be improved to only save the project for performance.
            workspace.save()?;
        }
        Commands::Ui => {
            let mut workspace = Workspace::load()?;
            tui::main(&mut workspace)?;
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
