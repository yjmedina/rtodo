//! Presentation layer for `rtodo`.
//!
//! This module owns all terminal-facing formatted output.  The domain types in
//! [`crate::models`] and [`crate::workspace`] deliberately stay plain — they
//! expose plain-text `Display` impls suitable for debugging and testing.
//! Everything visual lives here.

use std::cmp::Reverse;
use std::fmt;

use owo_colors::{OwoColorize, Stream, Style};

use crate::models::{Project, Status, Subtask, Task};
use crate::style;
use crate::workspace::Workspace;

// ── Task list view ────────────────────────────────────────────────────────────

const ALL_STATUSES: &[Status] = &[Status::InProgress, Status::New, Status::Completed];

pub struct TaskSummaryView<'a> {
    project: &'a Project,
    filter: Option<&'a [Status]>,
}

impl<'a> TaskSummaryView<'a> {
    pub fn new(project: &'a Project, filter: Option<&'a [Status]>) -> Self {
        Self { project, filter }
    }
}

impl fmt::Display for TaskSummaryView<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let is_filtered = self.filter.is_some();
        let statuses = self.filter.unwrap_or(ALL_STATUSES);
        let width = task_desc_width(self.project);

        for section_status in statuses {
            write_section(f, self.project, section_status, is_filtered, width)?;
        }

        Ok(())
    }
}

fn task_desc_width(project: &Project) -> usize {
    let task_max = project.tasks.iter().map(|t| t.description.len()).max();
    let sub_max = project
        .tasks
        .iter()
        .flat_map(|t| t.subtasks.iter())
        .map(|s| s.description.len())
        .max();
    task_max
        .into_iter()
        .chain(sub_max)
        .max()
        .unwrap_or(10)
        .max(10)
}

/// `Status::InProgress` for a subtask is mapped from `completed=false` *only*
/// when the parent task is in-progress (i.e. has at least one done sibling).
/// For section filtering we treat subtask state as `Completed` if `completed`,
/// else `New`.
fn subtask_status(sub: &Subtask) -> Status {
    if sub.completed {
        Status::Completed
    } else {
        Status::New
    }
}

fn tasks_for_section<'a>(
    project: &'a Project,
    section_status: &Status,
    is_filtered: bool,
) -> Vec<&'a Task> {
    let mut tasks: Vec<&Task> = project
        .tasks
        .iter()
        .filter(|t| {
            if is_filtered {
                t.status() == *section_status
                    || t.subtasks
                        .iter()
                        .any(|s| subtask_status(s) == *section_status)
            } else {
                t.status() == *section_status
            }
        })
        .collect();

    tasks.sort_by_key(|t| Reverse(&t.priority));
    tasks
}

fn write_section(
    f: &mut fmt::Formatter<'_>,
    project: &Project,
    section_status: &Status,
    is_filtered: bool,
    desc_width: usize,
) -> fmt::Result {
    writeln!(f, "{}", style::fmt_status_header(section_status))?;

    let tasks = tasks_for_section(project, section_status, is_filtered);

    if tasks.is_empty() {
        writeln!(
            f,
            "  {}",
            "(empty)".if_supports_color(Stream::Stdout, |v| v.dimmed())
        )?;
    } else {
        for task in tasks {
            write_task(f, project, task, section_status, is_filtered, desc_width)?;
        }
    }

    writeln!(f) // blank line between sections
}

fn write_task(
    f: &mut fmt::Formatter<'_>,
    project: &Project,
    task: &Task,
    section_status: &Status,
    is_filtered: bool,
    desc_width: usize,
) -> fmt::Result {
    let is_active = Some(task.id) == project.active_task_id;
    let line = style::fmt_task_line(task, is_active, desc_width);

    if task.subtasks.is_empty() {
        return writeln!(f, "{}", line);
    }

    let done = task.subtasks.iter().filter(|s| s.completed).count();
    let total = task.subtasks.len();
    let badge = format!(
        "{}",
        format!("[{done}/{total}]").if_supports_color(Stream::Stdout, |v| v.cyan())
    );
    writeln!(f, "{}  {}", line, badge)?;

    write_subtasks(f, &task.subtasks, section_status, is_filtered, desc_width)
}

fn write_subtasks(
    f: &mut fmt::Formatter<'_>,
    subtasks: &[Subtask],
    section_status: &Status,
    is_filtered: bool,
    desc_width: usize,
) -> fmt::Result {
    for sub in subtasks {
        if is_filtered && subtask_status(sub) != *section_status {
            continue;
        }
        writeln!(f, "{}", style::fmt_sub_line(sub, desc_width))?;
    }
    Ok(())
}

// ── Project list view ─────────────────────────────────────────────────────────

pub struct ProjectListView<'a> {
    workspace: &'a Workspace,
}

impl<'a> ProjectListView<'a> {
    pub fn new(workspace: &'a Workspace) -> Self {
        Self { workspace }
    }
}

impl fmt::Display for ProjectListView<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let workspace = self.workspace;

        if workspace.projects.is_empty() {
            return writeln!(
                f,
                "  {}",
                "No projects yet. Run `rtodo project add <name>` to create one."
                    .if_supports_color(Stream::Stdout, |v| v.dimmed())
            );
        }

        let name_width = workspace
            .projects
            .iter()
            .map(|p| p.name.len())
            .max()
            .unwrap_or(10)
            .max(10);

        for p in &workspace.projects {
            let id = style::fmt_id(p.id);
            let name = format!("{:<width$}", p.name, width = name_width);
            let count = format!("{} tasks", p.task_count());
            let date = style::fmt_date(&p.created_at);
            let active = if workspace.active_project_id == Some(p.id) {
                format!(
                    "  {}",
                    "← active"
                        .if_supports_color(Stream::Stdout, |v| v.style(Style::new().cyan().bold()))
                )
            } else {
                String::new()
            };
            writeln!(f, "  {} {}  {}  {}{}", id, name, count, date, active)?;
        }

        Ok(())
    }
}
