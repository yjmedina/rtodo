//! Presentation layer for `rtodo`.
//!
//! This module owns all terminal-facing formatted output.  The domain types in
//! [`crate::models`] and [`crate::workspace`] deliberately stay plain — they
//! expose plain-text `Display` impls suitable for debugging and testing.
//! Everything visual lives here.
//!
//! # Design: View structs + `Display`
//!
//! Each "view" is a lightweight struct that borrows the data it needs to render.
//! Implementing [`std::fmt::Display`] instead of returning `String` gives three
//! benefits:
//! 
use std::cmp::Reverse;
use std::fmt;

use owo_colors::{OwoColorize, Stream, Style};

use crate::models::{Project, Status, Task, ALL_STATUSES};
use crate::style;
use crate::workspace::Workspace;

// ── Task list view ────────────────────────────────────────────────────────────

/// A view over a [`Project`]'s task list, ready for colorized terminal display.
///
/// # Example
/// ```ignore
/// println!("{}", TaskSummaryView::new(project, None));
/// println!("{}", TaskSummaryView::new(project, Some(&[Status::New])));
/// ```
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

/// Returns the max task description length in the project, used for column alignment.
/// Subtasks are included so their column aligns with top-level tasks.
fn task_desc_width(project: &Project) -> usize {
    project
        .tasks
        .iter()
        .map(|t| t.description.len())
        .max()
        .unwrap_or(10)
        .max(10)
}

/// Returns top-level tasks that belong in `section_status`, sorted by priority descending.
///
/// When `is_filtered`, a parent is included if it matches *or* any of its subtasks match —
/// so the parent is visible as context even when its own status differs.
fn top_level_for_section<'a>(
    project: &'a Project,
    section_status: &Status,
    is_filtered: bool,
) -> Vec<&'a Task> {
    let mut tasks: Vec<&Task> = project
        .tasks
        .iter()
        .filter(|t| t.parent_id.is_none())
        .filter(|t| {
            if is_filtered {
                t.status == *section_status
                    || project
                        .subtasks_of(t.id)
                        .iter()
                        .any(|s| s.status == *section_status)
            } else {
                t.status == *section_status
            }
        })
        .collect();

    tasks.sort_by_key(|t| Reverse(&t.priority));
    tasks
}

/// Writes one status section: header, task rows (or an "(empty)" notice), and a trailing
/// blank line.
fn write_section(
    f: &mut fmt::Formatter<'_>,
    project: &Project,
    section_status: &Status,
    is_filtered: bool,
    desc_width: usize,
) -> fmt::Result {
    writeln!(f, "{}", style::fmt_status_header(section_status))?;

    let top_level = top_level_for_section(project, section_status, is_filtered);

    if top_level.is_empty() {
        writeln!(f, "  {}", "(empty)".if_supports_color(Stream::Stdout, |v| v.dimmed()))?;
    } else {
        for task in top_level {
            write_task(f, project, task, section_status, is_filtered, desc_width)?;
        }
    }

    writeln!(f) // blank line between sections
}

/// Writes a single task row and, when it has subtasks, the progress badge and subtask rows.
fn write_task(
    f: &mut fmt::Formatter<'_>,
    project: &Project,
    task: &Task,
    section_status: &Status,
    is_filtered: bool,
    desc_width: usize,
) -> fmt::Result {
    let subtasks = project.subtasks_of(task.id);
    let is_active = Some(task.id) == project.active_task_id;
    let line = style::fmt_task_line(task, is_active, desc_width);

    if subtasks.is_empty() {
        return writeln!(f, "{}", line);
    }

    let done = subtasks
        .iter()
        .filter(|s| s.status == Status::Completed)
        .count();
    let total = subtasks.len();
    let badge = format!(
        "{}",
        format!("[{done}/{total}]").if_supports_color(Stream::Stdout, |v| v.cyan())
    );
    writeln!(f, "{}  {}", line, badge)?;

    write_subtasks(f, subtasks, section_status, is_filtered, desc_width)
}

/// Writes the subtask rows for a parent, applying the status filter when active.
fn write_subtasks(
    f: &mut fmt::Formatter<'_>,
    subtasks: Vec<&Task>,
    section_status: &Status,
    is_filtered: bool,
    desc_width: usize,
) -> fmt::Result {
    let visible: Vec<&Task> = if is_filtered {
        subtasks
            .into_iter()
            .filter(|s| s.status == *section_status)
            .collect()
    } else {
        subtasks
    };

    for sub in visible {
        writeln!(f, "{}", style::fmt_sub_line(sub, desc_width))?;
    }

    Ok(())
}

// ── Project list view ─────────────────────────────────────────────────────────

/// A view over a [`Workspace`]'s project list, ready for colorized terminal display.
///
/// # Example
/// ```ignore
/// println!("{}", ProjectListView::new(&workspace));
/// ```
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

        // Align project names in a consistent column.
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
