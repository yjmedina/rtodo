//! Command dispatch layer for `rtodo`.
//!
//! Bridges parsed CLI arguments (from [`crate::cli`]) to operations on the
//! workspace and project models. Each function handles one family of subcommands.

use crate::cli::{ProjectCommands, TaskCommands};
use crate::models::{Project, Status};
use crate::workspace::Workspace;
use crate::{style, ui};

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
            let p = workspace.add_project(name);
            println!("  {} {}", style::action_green("added"), p);
        }
        ProjectCommands::Ls => {
            print!("{}", ui::ProjectListView::new(workspace));
        }
        ProjectCommands::Set { pid } => {
            let p = workspace.set_active_project(pid)?;
            println!("  {} {}", style::action_cyan("active"), p);
        }
        ProjectCommands::UnSet => {
            workspace.unset_active_project();
            println!("  {} no active project", style::action_red("unset"));
        }
        ProjectCommands::Delete { pid } => {
            let p = workspace.delete_project(pid)?;
            println!("  {} {}", style::action_red("removed"), p);
        }
        ProjectCommands::Edit { pid, name } => {
            let p = workspace.edit_project(pid, name)?;
            println!("  {} {}", style::action_green("updated"), p);
        }
    };
    Ok(())
}

/// Resolve an explicit task ID or fall back to the active task.
///
/// # Errors
/// Returns `Err` if both `tid` and the active task are `None`.
pub fn tid_or_active(project: &Project, tid: Option<u32>) -> Result<u32, String> {
    tid.or(project.active_task_id).ok_or(
        "No task selected. Provide a task ID or set an active task with `rtodo task start <id>`."
            .into(),
    )
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
        TaskCommands::Ls { status, pending } => {
            let statuses: Option<Vec<Status>> = if pending {
                Some(vec![Status::New, Status::InProgress])
            } else {
                status.map(|s| vec![s])
            };
            print!("{}", ui::TaskSummaryView::new(project, statuses.as_deref()));
        }
        TaskCommands::Add {
            desc,
            priority,
            parent,
        } => {
            let task = project.add_task(desc, priority, parent)?;
            println!(
                "  {} {}",
                style::action_green("added"),
                style::fmt_task_action(task)
            );
        }
        TaskCommands::Start { tid } => {
            let task = project.set_active_task(tid)?;
            println!(
                "  {} {}",
                style::action_cyan("started"),
                style::fmt_task_action(task)
            );
        }
        TaskCommands::Complete { tid } => {
            let tid = tid_or_active(project, tid)?;
            if project.has_incomplete_subtasks(tid) {
                return Err("Task has incomplete subtasks. Complete them first.".into());
            }
            let task = project.move_task(tid, Status::Completed)?;
            println!(
                "  {} {}",
                style::action_green("done"),
                style::fmt_task_action(task)
            );
        }
        TaskCommands::Move { tid, status } => {
            let tid = tid_or_active(project, tid)?;
            if status == Status::Completed && project.has_incomplete_subtasks(tid) {
                return Err("Task has incomplete subtasks. Complete them first.".into());
            }
            let task = project.move_task(tid, status)?;
            println!(
                "  {} {}",
                style::action_green("moved"),
                style::fmt_task_action(task)
            );
        }
        TaskCommands::Delete { tid } => {
            let task = project.delete_task(tid)?;
            println!(
                "  {} {}",
                style::action_red("removed"),
                style::fmt_task_action(&task)
            );
        }
        TaskCommands::Edit {
            tid,
            desc,
            priority,
        } => {
            let tid = tid_or_active(project, tid)?;
            let task = project.edit_task(tid, desc, priority)?;
            println!(
                "  {} {}",
                style::action_green("updated"),
                style::fmt_task_action(task)
            );
        }
    }

    Ok(())
}
