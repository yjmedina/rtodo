//! Core domain types for `rtodo`.
//!
//! This module defines the data model:
//! - [`Project`] — a named container that holds [`Task`]s.
//! - [`Task`] — a unit of work with a [`Status`] and [`Priority`].
//! - [`Status`] — lifecycle state of a task (`new` → `in_progress` → `completed`).
//! - [`Priority`] — importance level of a task (`low`, `medium`, `high`).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::fmt;
use std::fmt::Write;
const CREATED_AT_FORMAT: &str = "%Y-%m-%d";
const ALL_STATUSES: &[Status] = &[Status::InProgress, Status::New, Status::Completed];

/// A named project that contains a list of tasks.
#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    /// Unique identifier, assigned sequentially within the workspace.
    pub id: u32,
    /// Human-readable project name.
    pub name: String,
    /// All tasks belonging to this project.
    pub tasks: Vec<Task>,
    /// ID of the currently active task, if any.
    pub active_task_id: Option<u32>,
    /// Timestamp when this project was created.
    pub created_at: DateTime<Utc>,
}

impl Project {
    /// Create a new empty project with the given `id` and `name`.
    pub fn new(id: u32, name: String) -> Self {
        Project {
            id,
            name,
            tasks: Vec::new(),
            active_task_id: None,
            created_at: Utc::now(),
        }
    }

    /// Append a new task and return a reference to it.
    pub fn add_task(&mut self, description: String, priority: Priority) -> &Task {
        let idx = self.tasks.len();
        let task = Task::new(idx as u32, description, priority, Status::New);
        self.tasks.push(task);
        &self.tasks[idx]
    }

    /// Remove and return the task with `id`.
    ///
    /// # Errors
    /// Returns `Err` if no task with `id` exists in this project.
    pub fn delete_task(&mut self, id: u32) -> Result<Task, String> {
        let pos = self
            .find_task(id)
            .ok_or_else(|| format!("Task {id} not found."))?;
        if self.active_task_id == Some(id) {
            self.active_task_id = None;
        }
        Ok(self.tasks.swap_remove(pos))
    }

    /// Total number of tasks in this project.
    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }

    /// Return the active task, or `None` if no task is currently active.
    pub fn active_task(&self) -> Option<&Task> {
        self.find_task(self.active_task_id?)
            .map(|idx| &self.tasks[idx])
    }

    /// Set task `id` as the active task and transition it to `InProgress`.
    ///
    /// # Errors
    /// Returns `Err` if no task with `id` exists in this project.
    pub fn set_active_task(&mut self, id: u32) -> Result<&mut Task, String> {
        let idx = self
            .find_task(id)
            .ok_or_else(|| format!("Task {id} not found."))?;
        self.active_task_id = Some(id);
        let task = &mut self.tasks[idx];
        task.status = Status::InProgress;
        Ok(task)
    }

    /// Move task `id` to the given `status`.
    ///
    /// # Errors
    /// Returns `Err` if no task with `id` exists in this project.
    pub fn move_task(&mut self, id: u32, status: Status) -> Result<&Task, String> {
        let idx = self
            .find_task(id)
            .ok_or_else(|| format!("Task {id} not found."))?;
        self.tasks[idx].status = status;
        Ok(&self.tasks[idx])
    }

    /// Mark the active task as `Completed`.
    ///
    /// # Errors
    /// Returns `Err` if there is no active task.
    pub fn active_task_completed(&mut self) -> Result<&Task, String> {
        let id = self.active_task_id.ok_or(
            "No active task. Use `rtodo task set <id>` or `rtodo task move <id> <status>` instead.",
        )?;
        self.move_task(id, Status::Completed)
    }

    /// Return the index of task `id` within `self.tasks`, or `None` if not found.
    pub fn find_task(&self, id: u32) -> Option<usize> {
        self.tasks.iter().position(|t| t.id == id)
    }

    /// Edit description and/or priority of task `id`.
    ///
    /// # Errors
    /// Returns `Err` if no task with `id` exists in this project.
    pub fn edit_task(
        &mut self,
        id: u32,
        description: Option<String>,
        priority: Option<Priority>,
    ) -> Result<&Task, String> {
        let idx = self
            .find_task(id)
            .ok_or_else(|| format!("Task {id} not found."))?;
        if let Some(desc) = description {
            self.tasks[idx].description = desc;
        }
        if let Some(p) = priority {
            self.tasks[idx].priority = p;
        }
        Ok(&self.tasks[idx])
    }

    /// Return all tasks matching `status`, sorted by priority descending.
    ///
    /// Lifetimes are explicit here for practice — the returned references
    /// borrow from `self`.
    pub fn tasks_by_status(&self, status: &Status) -> Vec<&Task> {
        let mut filtered_tasks: Vec<&Task> =
            self.tasks.iter().filter(|&t| t.status == *status).collect();
        filtered_tasks.sort_by_key(|&t| Reverse(&t.priority));
        filtered_tasks
    }

    /// Build a formatted string listing all tasks grouped by status.
    pub fn task_summary(&self, status: Option<Status>) -> String {
        let mut out = String::new();

        let statuses: &[Status] = match &status {
            Some(s) => std::slice::from_ref(s),
            None => ALL_STATUSES,
        };

        for status in statuses {
            writeln!(out, "{status}").unwrap();

            let ftasks = self.tasks_by_status(status);

            if ftasks.is_empty() {
                writeln!(out, "  (none)").unwrap();
                continue;
            }

            for t in ftasks {
                writeln!(out, "  {t}").unwrap();
            }
        }

        out
    }
}

impl fmt::Display for Project {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[{}] {} ({} tasks) -- ({})",
            self.id,
            self.name,
            self.task_count(),
            self.created_at.format(CREATED_AT_FORMAT)
        )
    }
}

/// Lifecycle state of a task.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Status {
    /// Task has been created but work has not started.
    New,
    /// Task is actively being worked on.
    InProgress,
    /// Task has been finished.
    Completed,
}

impl TryFrom<&str> for Status {
    type Error = String;

    /// Parse a status from its string representation.
    ///
    /// # Errors
    /// Returns `Err` if `s` is not one of `new`, `in_progress`, `completed`.
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "completed" => Ok(Status::Completed),
            "in_progress" => Ok(Status::InProgress),
            "new" => Ok(Status::New),
            _ => Err(format!(
                "Unknown status \"{s}\". Valid values: new, in_progress, completed."
            )),
        }
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Status::New => write!(f, "new"),
            Status::InProgress => write!(f, "in_progress"),
            Status::Completed => write!(f, "completed"),
        }
    }
}

/// Importance level of a task.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    /// Low importance — tackle after `Medium` and `High` tasks.
    Low,
    /// Default importance level.
    Medium,
    /// High importance — shown with a `!` marker in listings.
    High,
}

impl TryFrom<&str> for Priority {
    type Error = String;

    /// Parse a priority from its string representation.
    ///
    /// # Errors
    /// Returns `Err` if `s` is not one of `low`, `medium`, `high`.
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "low" => Ok(Priority::Low),
            "medium" => Ok(Priority::Medium),
            "high" => Ok(Priority::High),
            _ => Err(format!(
                "Unknown priority \"{s}\". Valid values: low, medium, high."
            )),
        }
    }
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Priority::Low => write!(f, "low"),
            Priority::Medium => write!(f, "medium"),
            Priority::High => write!(f, "high"),
        }
    }
}

/// A single unit of work within a [`Project`].
///
/// Implements [`fmt::Debug`] via derive, which allows `println!("{:#?}", task)`.
#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    /// Unique identifier within the parent project.
    pub id: u32,
    /// Human-readable description of the work to be done.
    pub description: String,
    /// Relative importance of this task.
    pub priority: Priority,
    /// Current lifecycle state of this task.
    pub status: Status,
    /// Timestamp when this task was created.
    pub created_at: DateTime<Utc>,
}

impl Task {
    /// Create a new task with the given fields and the current UTC timestamp.
    pub fn new(id: u32, description: String, priority: Priority, status: Status) -> Self {
        Task {
            id,
            description,
            priority,
            status,
            created_at: Utc::now(),
        }
    }
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let marker = if self.priority == Priority::High {
            "!"
        } else {
            " "
        };
        write!(
            f,
            "[{}]{} {} ({}) [{}] -- ({})",
            self.id,
            marker,
            self.description,
            self.priority,
            self.status,
            self.created_at.format(CREATED_AT_FORMAT)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_project() -> Project {
        Project::new(0, String::from("A testing project"))
    }

    #[test]
    fn add_task_increments_id() {
        let mut project = get_project();
        let first_task = project.add_task(String::from("My first task"), Priority::Low);
        assert_eq!(first_task.id, 0);
        let second_task = project.add_task(String::from("My second task"), Priority::Low);
        assert_eq!(second_task.id, 1);
    }

    #[test]
    fn delete_task() {
        let mut project = get_project();
        project.add_task(String::from("My first task"), Priority::Low);
        let deleted_task = project.delete_task(0).expect("Task 0 should exists");
        assert_eq!(deleted_task.id, 0);
        assert_eq!(&deleted_task.description, "My first task");
        assert_eq!(project.task_count(), 0);
    }

    #[test]
    fn delete_missing_task() {
        let mut project = get_project();
        project.add_task(String::from("My first task"), Priority::Low);
        let deleted_task = project.delete_task(99);
        assert!(deleted_task.is_err(), "the task with id 99 must not exist");
        assert_eq!(project.task_count(), 1);
    }

    #[test]
    fn find_active_task() {
        let mut project = get_project();
        project.add_task(String::from("My first task"), Priority::Low);
        project.add_task(String::from("My Second task"), Priority::Low);
        project.active_task_id = Some(1);
        let task = project
            .active_task()
            .expect("Active task must be the second task");
        assert_eq!(task.id, 1);
        assert_eq!(task.description, "My Second task");
    }

    #[test]
    fn find_active_task_is_none() {
        let mut project = get_project();
        project.add_task(String::from("My first task"), Priority::Low);
        project.add_task(String::from("My Second task"), Priority::Low);
        let task = project.active_task();
        assert!(task.is_none());
    }

    #[test]
    fn find_task() {
        let mut project = get_project();
        project.add_task(String::from("My first task"), Priority::Low);
        project.add_task(String::from("My Second task"), Priority::Low);
        let idx = project.find_task(0).expect("Task 0 must exists");
        assert_eq!(idx, 0);
        assert_eq!(&project.tasks[idx].description, "My first task");
    }

    #[test]
    fn find_missing_task() {
        let mut project = get_project();
        project.add_task(String::from("My first task"), Priority::Low);
        project.add_task(String::from("My Second task"), Priority::Low);
        let task = project.find_task(99);
        assert!(task.is_none());
    }
}
