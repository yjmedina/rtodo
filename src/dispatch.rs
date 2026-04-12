//! Command dispatch layer for `rtodo`.
//!
//! Bridges parsed CLI arguments (from [`crate::cli`]) to operations on the
//! workspace and project models. Each function handles one family of subcommands.

use crate::cli::{ProjectCommands, TaskCommands};
use crate::models::{Priority, Project, Status};
use crate::workspace::Workspace;

/// Execute a `project` subcommand against the given workspace.
///
/// Mutates `workspace` in place. The caller is responsible for persisting
/// the workspace afterward.
///
/// # Errors
/// Returns `Err` if the requested project ID does not exist.
pub fn exec_project_cmd(command: ProjectCommands, workspace: &mut Workspace) -> Result<(), String> {
    match command {
        ProjectCommands::Add { name } => {
            workspace.add_project(name);
        }
        ProjectCommands::Ls => {
            println!("{}", workspace);
        }
        ProjectCommands::Set { pid } => {
            let p = workspace.set_active_project(pid)?;
            println!("Active project set to '{}'", p);
        }
        ProjectCommands::UnSet => {
            workspace.unset_active_project();
            println!("Active project unset");
        }
        ProjectCommands::Delete { .. } => {
            println!("This command is not yet implemented.");
        }
    };
    Ok(())
}

/// Execute a `task` subcommand against the active project.
///
/// Mutates `project` in place. The caller is responsible for persisting
/// the workspace afterward.
///
/// # Errors
/// Returns `Err` if the task ID does not exist, the status/priority string is
/// invalid, or there is no active task when one is required.
pub fn exec_task_cmd(command: TaskCommands, project: &mut Project) -> Result<(), String> {
    match command {
        TaskCommands::Ls => {
            println!("{}", project.task_summary());
        }
        TaskCommands::Add { desc, priority } => {
            let priority = Priority::try_from(priority.as_str())?;
            // todo add priority
            let task = project.add_task(desc, priority);
            println!("Task added succesfully\n{task}");
        }
        TaskCommands::Set { tid } => {
            let task = project.set_active_task(tid)?;
            println!("Active task: {task}");
        }
        TaskCommands::Completed => {
            let task = project.active_task_completed()?;
            println!("Completed!: {task}");
        }
        TaskCommands::Move { tid, status } => {
            let status = Status::try_from(status.as_str())?;
            let task = project.move_task(tid, status)?;
            println!("Task moved to {}: {}", task.status, task);
        }
    }

    Ok(())
}
