//! Core domain types for `rtodo`.
//!

pub mod priority;
pub mod status;
pub mod subtask;
pub mod task;

use crate::error::AppError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::fmt;

pub use crate::models::subtask::Subtask;
pub use crate::models::task::{CREATED_AT_FORMAT, Priority, Status, Task};

/// A named project that contains a list of tasks.
#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    pub id: u32,
    pub name: String,
    pub tasks: Vec<Task>,
    pub active_task_id: Option<u32>,
    pub created_at: DateTime<Utc>,
}

impl Project {
    pub fn new(id: u32, name: String) -> Self {
        Project {
            id,
            name,
            tasks: Vec::new(),
            active_task_id: None,
            created_at: Utc::now(),
        }
    }

    pub fn add_task(&mut self, description: String, priority: Priority) -> &Task {
        let id = self
            .tasks
            .iter()
            .map(|t| t.id)
            .max()
            .map(|m| m + 1)
            .unwrap_or(0);
        let task = Task::new(id, description, priority, Status::New);
        self.tasks.push(task);
        self.tasks.last().unwrap()
    }

    pub fn delete_task(&mut self, id: u32) -> Result<Task, AppError> {
        let pos = self.get_task(id)?;
        if self.active_task_id == Some(id) {
            self.active_task_id = None;
        }
        Ok(self.tasks.swap_remove(pos))
    }

    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }

    pub fn active_task(&self) -> Result<&Task, AppError> {
        let id = self.active_task_id.ok_or(AppError::NoActiveTask)?;
        let idx = self.get_task(id).map_err(|_| AppError::NoActiveTask)?;
        Ok(&self.tasks[idx])
    }

    pub fn set_active_task(&mut self, id: u32) -> Result<&mut Task, AppError> {
        let idx = self.get_task(id)?;
        self.active_task_id = Some(id);
        let task = &mut self.tasks[idx];
        // `set_status` enforces validity (no-op when subtasks already drove it).
        let _ = task.set_status(Status::InProgress);
        Ok(task)
    }

    pub fn move_task(&mut self, id: u32, status: Status) -> Result<&Task, AppError> {
        let idx = self.get_task(id)?;
        self.tasks[idx].set_status(status)?;
        Ok(&self.tasks[idx])
    }

    pub fn complete_active_task(&mut self) -> Result<&Task, AppError> {
        let id = self.active_task_id.ok_or(AppError::NoActiveTask)?;
        self.move_task(id, Status::Completed)
    }

    pub fn get_task(&self, id: u32) -> Result<usize, AppError> {
        self.tasks
            .iter()
            .position(|t| t.id == id)
            .ok_or(AppError::TaskNotFound { id })
    }

    pub fn get_task_mut(&mut self, id: u32) -> Result<&mut Task, AppError> {
        let idx = self.get_task(id)?;
        Ok(&mut self.tasks[idx])
    }

    pub fn edit_task(
        &mut self,
        id: u32,
        description: Option<String>,
        priority: Option<Priority>,
    ) -> Result<&Task, AppError> {
        let idx = self.get_task(id)?;
        self.tasks[idx].edit(description, priority);
        Ok(&self.tasks[idx])
    }

    pub fn tasks_with_status(&self, status: &Status) -> Vec<&Task> {
        let mut filtered: Vec<&Task> = self
            .tasks
            .iter()
            .filter(|t| t.status() == *status)
            .collect();
        filtered.sort_by_key(|t| Reverse(&t.priority));
        filtered
    }

    pub fn complete_tasks(&self) -> usize {
        self.tasks
            .iter()
            .filter(|t| t.status() == Status::Completed)
            .count()
    }

    // ---------- subtask pass-throughs ----------

    pub fn add_subtask(
        &mut self,
        tid: u32,
        description: String,
        priority: Priority,
    ) -> Result<&Subtask, AppError> {
        Ok(self.get_task_mut(tid)?.add_subtask(description, priority))
    }

    pub fn delete_subtask(&mut self, tid: u32, sid: u32) -> Result<Subtask, AppError> {
        self.get_task_mut(tid)?.delete_subtask(sid)
    }

    pub fn complete_subtask(&mut self, tid: u32, sid: u32) -> Result<&Subtask, AppError> {
        self.get_task_mut(tid)?.complete_subtask(sid)
    }

    pub fn uncomplete_subtask(&mut self, tid: u32, sid: u32) -> Result<&Subtask, AppError> {
        self.get_task_mut(tid)?.uncomplete_subtask(sid)
    }

    pub fn edit_subtask(
        &mut self,
        tid: u32,
        sid: u32,
        description: Option<String>,
        priority: Option<Priority>,
    ) -> Result<&Subtask, AppError> {
        self.get_task_mut(tid)?
            .edit_subtask(sid, description, priority)
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
        let first = project.add_task(String::from("a"), Priority::Low);
        assert_eq!(first.id, 0);
        let second = project.add_task(String::from("b"), Priority::Low);
        assert_eq!(second.id, 1);
    }

    #[test]
    fn delete_task_owns_subtasks() {
        let mut project = get_project();
        project.add_task(String::from("parent"), Priority::Low);
        project
            .add_subtask(0, String::from("a"), Priority::Low)
            .unwrap();
        project
            .add_subtask(0, String::from("b"), Priority::Low)
            .unwrap();
        let removed = project.delete_task(0).unwrap();
        assert_eq!(removed.subtasks.len(), 2);
        assert_eq!(project.task_count(), 0);
    }

    #[test]
    fn delete_missing_task() {
        let mut project = get_project();
        let err = project.delete_task(99);
        assert!(err.is_err());
    }

    #[test]
    fn find_active_task() {
        let mut project = get_project();
        project.add_task(String::from("a"), Priority::Low);
        project.add_task(String::from("b"), Priority::Low);
        project.set_active_task(1).unwrap();
        let task = project.active_task().unwrap();
        assert_eq!(task.id, 1);
    }

    #[test]
    fn find_active_task_is_none() {
        let project = get_project();
        assert!(project.active_task().is_err());
    }

    #[test]
    fn complete_subtask_propagates_to_task_status() {
        let mut project = get_project();
        project.add_task(String::from("parent"), Priority::Low);
        project
            .add_subtask(0, String::from("a"), Priority::Low)
            .unwrap();
        project.complete_subtask(0, 0).unwrap();
        let task = &project.tasks[0];
        assert_eq!(task.status(), Status::Completed);
    }
}
