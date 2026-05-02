//! CLI definitions for `rtodo`.
//!
//! All commands and subcommands are declared here using clap's derive macros.
//! Parsed values flow into [`crate::dispatch`] for execution.

use std::fmt;
use std::str::FromStr;

use crate::models::{Priority, Status};
use clap::{Parser, Subcommand};

/// A reference to either a task (`"0"`) or a subtask (`"0.13"`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaskRef {
    pub task: u32,
    pub subtask: Option<u32>,
}

impl FromStr for TaskRef {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('.');
        let task_str = parts.next().ok_or_else(|| "empty task ref".to_string())?;
        let task: u32 = task_str
            .parse()
            .map_err(|_| format!("invalid task id: '{task_str}'"))?;

        let subtask = match parts.next() {
            None => None,
            Some(sub_str) => {
                let sid: u32 = sub_str
                    .parse()
                    .map_err(|_| format!("invalid subtask id: '{sub_str}'"))?;
                Some(sid)
            }
        };

        if parts.next().is_some() {
            return Err(format!("too many dots in task ref: '{s}'"));
        }

        Ok(TaskRef { task, subtask })
    }
}

impl fmt::Display for TaskRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.subtask {
            Some(sid) => write!(f, "{}.{}", self.task, sid),
            None => write!(f, "{}", self.task),
        }
    }
}

/// Root CLI entry point for `rtodo`.
#[derive(Parser, Debug)]
pub struct CLI {
    #[command(subcommand)]
    pub command: Commands,
}

/// Top-level commands available in `rtodo`.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize a new workspace in the current directory.
    Init,

    /// Manage projects within the workspace.
    Project {
        #[command(subcommand)]
        command: ProjectCommands,
    },

    /// Manage tasks within the active project.
    Task {
        #[command(subcommand)]
        command: TaskCommands,
    },

    /// Start TUI
    Ui,
}

/// Subcommands for `rtodo project`.
#[derive(Subcommand, Debug)]
pub enum ProjectCommands {
    Add { name: String },
    Ls,
    Set { pid: u32 },
    UnSet,
    Delete { pid: u32 },
    Edit { pid: u32, name: String },
}

/// Subcommands for `rtodo task`.
#[derive(Subcommand, Debug)]
pub enum TaskCommands {
    /// Add a new task or subtask.
    ///
    /// Pass `--parent <tid>` to add as a subtask of the given task.
    Add {
        desc: String,
        #[arg(short, long, default_value_t = Priority::Medium)]
        priority: Priority,
        #[arg(short = 'P', long)]
        parent: Option<u32>,
    },

    /// List all tasks in the active project, grouped by status.
    Ls {
        status: Option<Status>,

        #[arg(short, long, conflicts_with = "status")]
        pending: bool,
    },

    /// Set a task as the active task by its ID.
    ///
    /// `start` only accepts task IDs (not subtask refs).
    Start { tid: u32 },

    /// Mark a task or subtask as completed.
    ///
    /// Defaults to active task. Accepts `<tid>` or `<tid>.<sid>`.
    Complete { tref: Option<TaskRef> },

    /// Move a task or subtask to a status.
    ///
    /// Usage: `task move <status> [<tref>]`. Defaults to active task.
    /// Subtasks support only `new` and `completed`.
    Move {
        status: Status,
        tref: Option<TaskRef>,
    },

    /// Delete a task or subtask. Accepts `<tid>` or `<tid>.<sid>`.
    Delete { tref: TaskRef },

    /// Edit a task's or subtask's description and/or priority.
    Edit {
        #[arg(short, long)]
        tref: Option<TaskRef>,
        #[arg(short, long)]
        desc: Option<String>,
        #[arg(short, long)]
        priority: Option<Priority>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn taskref_parses_task_only() {
        let r: TaskRef = "0".parse().unwrap();
        assert_eq!(r.task, 0);
        assert_eq!(r.subtask, None);
    }

    #[test]
    fn taskref_parses_dotted() {
        let r: TaskRef = "0.13".parse().unwrap();
        assert_eq!(r.task, 0);
        assert_eq!(r.subtask, Some(13));
    }

    #[test]
    fn taskref_rejects_too_many_dots() {
        let r: Result<TaskRef, _> = "0.13.4".parse();
        assert!(r.is_err());
    }

    #[test]
    fn taskref_rejects_non_numeric() {
        assert!("abc".parse::<TaskRef>().is_err());
        assert!("0.x".parse::<TaskRef>().is_err());
    }

    #[test]
    fn taskref_rejects_trailing_dot() {
        assert!("0.".parse::<TaskRef>().is_err());
    }

    #[test]
    fn taskref_display_round_trips() {
        let r = TaskRef {
            task: 2,
            subtask: Some(7),
        };
        assert_eq!(r.to_string(), "2.7");
        let r2 = TaskRef {
            task: 2,
            subtask: None,
        };
        assert_eq!(r2.to_string(), "2");
    }
}
