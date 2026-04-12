//! Binary entry point for `rtodo`.
//!
//! Parses CLI arguments via [`CLI`], then delegates to [`exec_project_cmd`]
//! or [`exec_task_cmd`] in [`dispatch`]. Workspace state is loaded before
//! each command and persisted afterward.

use clap::Parser;
use rtodo::cli::{CLI, Commands};
use rtodo::dispatch::{exec_project_cmd, exec_task_cmd};
use rtodo::workspace::Workspace;

/// Discovers and loads the workspace from the nearest `.rtodo/state.json`.
///
/// # Errors
/// Returns `Err` if no workspace is found or the file cannot be parsed.
fn load_workspace() -> Result<Workspace, String> {
    Workspace::load().map_err(|err| format!("Failed to load workspace: {err}"))
}

/// Serializes `workspace` back to `.rtodo/state.json`.
///
/// # Errors
/// Returns `Err` if the file cannot be written.
fn save_workspace(workspace: &Workspace) -> Result<(), String> {
    workspace
        .save()
        .map_err(|err| format!("Failed to save workspace: {err}"))?;
    Ok(())
}

/// Parses arguments and runs the matching command.
///
/// # Errors
/// Returns `Err` with a user-facing message on any failure.
fn run() -> Result<(), String> {
    let cli = CLI::parse();

    match cli.command {
        Commands::Init => {
            Workspace::init().map_err(|err| format!("Failed to initialize workspace: {err}"))?;
        }
        Commands::Project { command } => {
            let mut workspace = load_workspace()?;
            exec_project_cmd(command, &mut workspace)?;
            save_workspace(&workspace)?;
        }
        Commands::Task { command } => {
            let mut workspace = load_workspace()?;
            let project = workspace
                .active_project()
                .ok_or("No active project. Run `rtodo project set <id>` to set one.")?;
            exec_task_cmd(command, project)?;
            // this could be improve to only save the project for performance.
            save_workspace(&workspace)?;
        }
    };
    Ok(())
}

fn main() {
    if let Err(msg) = run() {
        eprintln!("{}", msg);
        std::process::exit(1);
    }
}
