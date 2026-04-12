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
        ProjectCommands::Delete { pid } => {
            let p = workspace.delete_project(pid)?;
            println!("{p} have been deleted")
        }
        ProjectCommands::Edit { pid, name } => {
            let p = workspace.edit_project(pid, name)?;
            println!("{p}")
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
        TaskCommands::Ls { status } => {
            let status = status.map(|s| Status::try_from(s.as_str())).transpose()?;
            println!("{}", project.task_summary(status));
        }
        TaskCommands::Add {
            desc,
            priority,
            parent,
        } => {
            let priority = Priority::try_from(priority.as_str())?;
            let task = project.add_task(desc, priority, parent)?;
            println!("Task added successfully\n{task}");
        }
        TaskCommands::Set { tid } => {
            let task = project.set_active_task(tid)?;
            println!("Active task: {task}");
        }
        TaskCommands::Completed => {
            let id = project.active_task_id.ok_or(
                "No active task. Use `rtodo task set <id>` or `rtodo task move <id> <status>` instead.",
            )?;
            if project.has_incomplete_subtasks(id) {
                return Err("Task has incomplete subtasks. Complete them first.".into());
            }
            let task = project.active_task_completed()?;
            println!("Completed!: {task}");
        }
        TaskCommands::Move { tid, status } => {
            let status = Status::try_from(status.as_str())?;
            if status == Status::Completed && project.has_incomplete_subtasks(tid) {
                return Err("Task has incomplete subtasks. Complete them first.".into());
            }
            let task = project.move_task(tid, status)?;
            println!("Task moved to {}: {}", task.status, task);
        }
        TaskCommands::Delete { tid } => {
            let task = project.delete_task(tid)?;
            println!("Deleted task: {task}");
        }
        TaskCommands::Edit {
            tid,
            desc,
            priority,
        } => {
            let priority = priority
                .map(|p| Priority::try_from(p.as_str()))
                .transpose()?;
            let task = project.edit_task(tid, desc, priority)?;
            println!("Updated task: {task}");
        }
    }

    Ok(())
}
