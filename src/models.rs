//! Core domain types for `rtodo`.
//!

pub mod priority;
pub mod status;
pub mod task;

use crate::error::AppError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::fmt;

pub use crate::models::task::{CREATED_AT_FORMAT, Priority, Status, Task};

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
    ///
    /// # Errors
    /// Returns `Err` if `parent_id` does not exist or is itself a subtask (max depth: 2).
    pub fn add_task(
        &mut self,
        description: String,
        priority: Priority,
        parent_id: Option<u32>,
    ) -> Result<&Task, AppError> {
        if let Some(pid) = parent_id {
            let parent = self
                .tasks
                .iter()
                .find(|t| t.id == pid)
                .ok_or_else(|| AppError::TaskNotFound { id: pid })?;
            if parent.parent_id.is_some() {
                return Err(AppError::SubtaskDepthExceeded);
            }
        }
        let id = self
            .tasks
            .iter()
            .map(|t| t.id)
            .max()
            .map(|m| m + 1)
            .unwrap_or(0);
        let task = Task::new(id, description, priority, Status::New, parent_id);
        self.tasks.push(task);
        Ok(self.tasks.last().unwrap())
    }

    /// Return all direct subtasks of `task_id`, in insertion order.
    pub fn subtasks_of(&self, task_id: u32) -> Vec<&Task> {
        self.tasks
            .iter()
            .filter(|t| t.parent_id == Some(task_id))
            .collect()
    }

    /// Return `true` if `task_id` has at least one non-`Completed` subtask.
    pub fn has_incomplete_subtasks(&self, task_id: u32) -> bool {
        self.subtasks_of(task_id)
            .iter()
            .any(|t| t.status != Status::Completed)
    }

    /// Remove and return the task with `id`, cascading to any subtasks.
    ///
    /// # Errors
    /// Returns `Err` if no task with `id` exists in this project.
    pub fn delete_task(&mut self, id: u32) -> Result<Task, AppError> {
        self.find_task(id)
            .ok_or_else(|| AppError::TaskNotFound { id })?;
        // Clear active_task if it's the deleted task or one of its subtasks
        if self.active_task_id == Some(id)
            || self
                .active_task_id
                .map(|aid| {
                    self.tasks
                        .iter()
                        .any(|t| t.id == aid && t.parent_id == Some(id))
                })
                .unwrap_or(false)
        {
            self.active_task_id = None;
        }
        // Cascade: remove subtasks
        self.tasks.retain(|t| t.parent_id != Some(id));
        // Re-find after retain (indices may have shifted)
        let pos = self.find_task(id).unwrap();
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
    pub fn set_active_task(&mut self, id: u32) -> Result<&mut Task, AppError> {
        let idx = self
            .find_task(id)
            .ok_or_else(|| AppError::TaskNotFound { id })?;
        self.active_task_id = Some(id);
        let task = &mut self.tasks[idx];
        task.status = Status::InProgress;
        Ok(task)
    }

    /// Move task `id` to the given `status`.
    ///
    /// # Errors
    /// Returns `Err` if no task with `id` exists in this project.
    pub fn move_task(&mut self, id: u32, status: Status) -> Result<&Task, AppError> {
        let idx = self
            .find_task(id)
            .ok_or_else(|| AppError::TaskNotFound { id })?;
        self.tasks[idx].status = status;
        Ok(&self.tasks[idx])
    }

    /// Mark the active task as `Completed`.
    ///
    /// # Errors
    /// Returns `Err` if there is no active task.
    pub fn active_task_completed(&mut self) -> Result<&Task, AppError> {
        let id = self.active_task_id.ok_or(AppError::NoActiveTask)?;
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
    ) -> Result<&Task, AppError> {
        let idx = self
            .find_task(id)
            .ok_or_else(|| AppError::TaskNotFound { id })?;

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

#[cfg(test)]
mod tests {
    use super::*;

    fn get_project() -> Project {
        Project::new(0, String::from("A testing project"))
    }

    #[test]
    fn add_task_increments_id() {
        let mut project = get_project();
        let first_task = project
            .add_task(String::from("My first task"), Priority::Low, None)
            .unwrap();
        assert_eq!(first_task.id, 0);
        let second_task = project
            .add_task(String::from("My second task"), Priority::Low, None)
            .unwrap();
        assert_eq!(second_task.id, 1);
    }

    #[test]
    fn delete_task() {
        let mut project = get_project();
        project
            .add_task(String::from("My first task"), Priority::Low, None)
            .unwrap();
        let deleted_task = project.delete_task(0).expect("Task 0 should exists");
        assert_eq!(deleted_task.id, 0);
        assert_eq!(&deleted_task.description, "My first task");
        assert_eq!(project.task_count(), 0);
    }

    #[test]
    fn delete_missing_task() {
        let mut project = get_project();
        project
            .add_task(String::from("My first task"), Priority::Low, None)
            .unwrap();
        let deleted_task = project.delete_task(99);
        assert!(deleted_task.is_err(), "the task with id 99 must not exist");
        assert_eq!(project.task_count(), 1);
    }

    #[test]
    fn delete_task_cascades_subtasks() {
        let mut project = get_project();
        project
            .add_task(String::from("Parent"), Priority::Low, None)
            .unwrap();
        project
            .add_task(String::from("Sub A"), Priority::Low, Some(0))
            .unwrap();
        project
            .add_task(String::from("Sub B"), Priority::Low, Some(0))
            .unwrap();
        assert_eq!(project.task_count(), 3);
        project.delete_task(0).unwrap();
        assert_eq!(project.task_count(), 0);
    }

    #[test]
    fn add_subtask_depth_limit() {
        let mut project = get_project();
        project
            .add_task(String::from("Parent"), Priority::Low, None)
            .unwrap();
        project
            .add_task(String::from("Child"), Priority::Low, Some(0))
            .unwrap();
        let result = project.add_task(String::from("Grandchild"), Priority::Low, Some(1));
        assert!(result.is_err());
    }

    #[test]
    fn find_active_task() {
        let mut project = get_project();
        project
            .add_task(String::from("My first task"), Priority::Low, None)
            .unwrap();
        project
            .add_task(String::from("My Second task"), Priority::Low, None)
            .unwrap();
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
        project
            .add_task(String::from("My first task"), Priority::Low, None)
            .unwrap();
        project
            .add_task(String::from("My Second task"), Priority::Low, None)
            .unwrap();
        let task = project.active_task();
        assert!(task.is_none());
    }

    #[test]
    fn find_task() {
        let mut project = get_project();
        project
            .add_task(String::from("My first task"), Priority::Low, None)
            .unwrap();
        project
            .add_task(String::from("My Second task"), Priority::Low, None)
            .unwrap();
        let idx = project.find_task(0).expect("Task 0 must exists");
        assert_eq!(idx, 0);
        assert_eq!(&project.tasks[idx].description, "My first task");
    }

    #[test]
    fn find_missing_task() {
        let mut project = get_project();
        project
            .add_task(String::from("My first task"), Priority::Low, None)
            .unwrap();
        project
            .add_task(String::from("My Second task"), Priority::Low, None)
            .unwrap();
        let task = project.find_task(99);
        assert!(task.is_none());
    }
}
