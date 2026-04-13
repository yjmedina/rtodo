//! CLI definitions for `rtodo`.
//!
//! All commands and subcommands are declared here using clap's derive macros.
//! Parsed values flow into [`crate::dispatch`] for execution.

use crate::models::{Priority, Status};
use clap::{Parser, Subcommand};

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
    ///
    /// Creates a `.rtodo/state.json` file. Run this once per project root.
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
}

/// Subcommands for `rtodo project`.
#[derive(Subcommand, Debug)]
pub enum ProjectCommands {
    /// Create a new project with the given name.
    Add { name: String },

    /// List all projects in the workspace.
    Ls,

    /// Set a project as the active project by its ID.
    Set { pid: u32 },

    /// Clear the active project selection.
    UnSet,

    /// Delete a project by its ID.
    Delete { pid: u32 },

    /// Edit Project
    Edit { pid: u32, name: String },
}

/// Subcommands for `rtodo task`.
#[derive(Subcommand, Debug)]
pub enum TaskCommands {
    /// Add a new task to the active project.
    ///
    /// `--priority` accepts: `low`, `medium` (default), `high`.
    /// `--parent` adds this as a subtask of the given task ID (max depth: 2).
    Add {
        desc: String,
        #[arg(short, long, default_value_t = Priority::Medium)]
        priority: Priority,
        #[arg(short = 'P', long)]
        parent: Option<u32>,
    },

    /// List all tasks in the active project, grouped by status.
    ///
    /// Pass a status positionally to filter: `new`, `in-progress`, `completed`.
    /// Use `--pending` / `-p` to show only incomplete tasks (new + in-progress).
    Ls {
        /// Filter by status: new, in-progress, completed
        status: Option<Status>,

        /// Show only incomplete tasks (new + in-progress)
        #[arg(short, long, conflicts_with = "status")]
        pending: bool,
    },

    /// Set a task as the active task by its ID.
    ///
    /// Also transitions the task status to `in_progress`.
    Start { tid: u32 },

    /// Mark the task as completed. Defaults to active task
    ///
    /// --tid defaults to active task
    Complete { tid: Option<u32> },

    /// Move a task to a specific status by its ID.
    ///
    /// --tid defaults to active task
    Move { tid: Option<u32>, status: Status },

    /// Delete a task by its ID.
    Delete { tid: u32 },

    /// Edit a task's description and/or priority.
    ///
    /// --tid defaults to active task
    /// `--priority` accepts: `low`, `medium`, `high`.
    Edit {
        #[arg(short, long)]
        tid: Option<u32>,
        #[arg(short, long)]
        desc: Option<String>,
        #[arg(short, long)]
        priority: Option<Priority>,
    },
}
