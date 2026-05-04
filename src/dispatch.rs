//! Command dispatch layer for `rtodo`.

use crate::cli::{ProjectCommands, TaskCommands, TaskRef};
use crate::error::AppError;
use crate::models::{Project, Status};
use crate::workspace::Workspace;
use crate::{style, ui};

pub fn dispatch_project(
    command: ProjectCommands,
    workspace: &mut Workspace,
) -> Result<(), AppError> {
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
            workspace.clear_active_project();
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

/// Resolve an explicit `TaskRef` or fall back to the active task.
fn tref_or_active(project: &Project, tref: Option<TaskRef>) -> Result<TaskRef, AppError> {
    match tref {
        Some(r) => Ok(r),
        None => Ok(TaskRef {
            task: project.active_task_id.ok_or(AppError::NoActiveTask)?,
            subtask: None,
        }),
    }
}

pub fn dispatch_task(command: TaskCommands, project: &mut Project) -> Result<(), AppError> {
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
        } => match parent {
            Some(tid) => {
                let s = project.add_subtask(tid, desc, priority)?;
                println!("  {} subtask {}", style::action_green("added"), s);
            }
            None => {
                let task = project.add_task(desc, priority);
                println!(
                    "  {} {}",
                    style::action_green("added"),
                    style::fmt_task_action(task)
                );
            }
        },
        TaskCommands::Start { tid } => {
            let task = project.set_active_task(tid)?;
            println!(
                "  {} {}",
                style::action_cyan("started"),
                style::fmt_task_action(task)
            );
        }
        TaskCommands::Complete { tref } => {
            let r = tref_or_active(project, tref)?;
            match r.subtask {
                Some(sid) => {
                    let s = project.complete_subtask(r.task, sid)?;
                    println!("  {} subtask {}", style::action_green("done"), s);
                }
                None => {
                    let task = project.move_task(r.task, Status::Completed)?;
                    println!(
                        "  {} {}",
                        style::action_green("done"),
                        style::fmt_task_action(task)
                    );
                }
            }
        }
        TaskCommands::Move { tref, status } => {
            let r = tref_or_active(project, tref)?;
            match r.subtask {
                Some(sid) => match status {
                    Status::Completed => {
                        let s = project.complete_subtask(r.task, sid)?;
                        println!("  {} subtask {}", style::action_green("moved"), s);
                    }
                    Status::New => {
                        let s = project.uncomplete_subtask(r.task, sid)?;
                        println!("  {} subtask {}", style::action_green("moved"), s);
                    }
                    Status::InProgress => {
                        return Err(AppError::InvalidStatusTransition { id: r.task });
                    }
                },
                None => {
                    let task = project.move_task(r.task, status)?;
                    println!(
                        "  {} {}",
                        style::action_green("moved"),
                        style::fmt_task_action(task)
                    );
                }
            }
        }
        TaskCommands::Delete { tref } => match tref.subtask {
            Some(sid) => {
                let s = project.delete_subtask(tref.task, sid)?;
                println!("  {} subtask {}", style::action_red("removed"), s);
            }
            None => {
                let task = project.delete_task(tref.task)?;
                println!(
                    "  {} {}",
                    style::action_red("removed"),
                    style::fmt_task_action(&task)
                );
            }
        },
        TaskCommands::Edit {
            tref,
            desc,
            priority,
        } => {
            let r = tref_or_active(project, tref)?;
            match r.subtask {
                Some(sid) => {
                    let s = project.edit_subtask(r.task, sid, desc, priority)?;
                    println!("  {} subtask {}", style::action_green("updated"), s);
                }
                None => {
                    let task = project.edit_task(r.task, desc, priority)?;
                    println!(
                        "  {} {}",
                        style::action_green("updated"),
                        style::fmt_task_action(task)
                    );
                }
            }
        }
    }

    Ok(())
}
