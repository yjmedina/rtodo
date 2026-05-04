use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::AppError;
pub use crate::models::priority::Priority;
pub use crate::models::status::Status;
pub use crate::models::subtask::Subtask;

pub const CREATED_AT_FORMAT: &str = "%Y-%m-%d";

#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: u32,
    pub description: String,
    pub priority: Priority,
    status: Status,
    pub subtasks: Vec<Subtask>,
    next_subtask_id: u32,
    pub created_at: DateTime<Utc>,
}

impl Task {
    pub fn new(id: u32, description: String, priority: Priority, status: Status) -> Self {
        Task {
            id,
            description,
            priority,
            status,
            subtasks: Vec::new(),
            next_subtask_id: 0,
            created_at: Utc::now(),
        }
    }

    pub fn status(&self) -> Status {
        self.status.clone()
    }

    /// Manual status transition. Allowed iff the move is consistent with the
    /// subtasks' current state (validator below).
    pub fn set_status(&mut self, new_status: Status) -> Result<(), AppError> {
        if !self.is_status_allowed(&new_status) {
            return match new_status {
                Status::Completed => Err(AppError::TaskHasIncompleteSubtasks { id: self.id }),
                _ => Err(AppError::InvalidStatusTransition { id: self.id }),
            };
        }
        self.status = new_status;
        Ok(())
    }

    pub fn completed_subtask_count(&self) -> usize {
        self.subtasks.iter().filter(|s| s.completed).count()
    }

    fn is_status_allowed(&self, new_status: &Status) -> bool {
        let total = self.subtasks.len();
        if total == 0 {
            return true;
        }
        let done = self.completed_subtask_count();

        match new_status {
            Status::Completed => done == total,
            Status::InProgress => true,
            Status::New => done == 0,
        }
    }

    /// Re-derive `status` after a subtask mutation.
    /// Rules (when subtasks non-empty):
    ///   all completed   → Completed
    ///   ≥1 completed    → InProgress
    ///   0  completed    → New (force, overrides manual InProgress)
    /// Empty subtasks list → sticky (status untouched).
    fn recompute_status(&mut self) {
        let total = self.subtasks.len();
        if total == 0 {
            return;
        }
        let done = self.completed_subtask_count();

        self.status = if done == total {
            Status::Completed
        } else if done > 0 {
            Status::InProgress
        } else {
            Status::New
        };
    }

    pub fn add_subtask(&mut self, description: String, priority: Priority) -> &Subtask {
        let id = self.next_subtask_id;
        self.next_subtask_id += 1;
        let s = Subtask::new(id, description, priority);
        self.subtasks.push(s);
        self.recompute_status();
        self.subtasks.last().unwrap()
    }

    pub fn get_subtask(&self, sid: u32) -> Result<usize, AppError> {
        self.subtasks
            .iter()
            .position(|s| s.id == sid)
            .ok_or(AppError::SubtaskNotFound {
                task_id: self.id,
                subtask_id: sid,
            })
    }

    pub fn delete_subtask(&mut self, sid: u32) -> Result<Subtask, AppError> {
        let idx = self.get_subtask(sid)?;
        let removed = self.subtasks.remove(idx);
        self.recompute_status();
        Ok(removed)
    }

    pub fn complete_subtask(&mut self, sid: u32) -> Result<&Subtask, AppError> {
        let idx = self.get_subtask(sid)?;
        self.subtasks[idx].complete();
        self.recompute_status();
        Ok(&self.subtasks[idx])
    }

    pub fn uncomplete_subtask(&mut self, sid: u32) -> Result<&Subtask, AppError> {
        let idx = self.get_subtask(sid)?;
        self.subtasks[idx].uncomplete();
        self.recompute_status();
        Ok(&self.subtasks[idx])
    }

    pub fn edit_subtask(
        &mut self,
        sid: u32,
        description: Option<String>,
        priority: Option<Priority>,
    ) -> Result<&Subtask, AppError> {
        let idx = self.get_subtask(sid)?;
        self.subtasks[idx].edit(description, priority);
        Ok(&self.subtasks[idx])
    }

    pub fn edit(&mut self, description: Option<String>, priority: Option<Priority>) {
        if let Some(desc) = description {
            self.description = desc;
        }
        if let Some(p) = priority {
            self.priority = p;
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

    fn task() -> Task {
        Task::new(0, String::from("parent"), Priority::Medium, Status::New)
    }

    #[test]
    fn add_subtask_increments_local_id() {
        let mut t = task();
        let a = t.add_subtask(String::from("a"), Priority::Low).id;
        let b = t.add_subtask(String::from("b"), Priority::Low).id;
        assert_eq!(a, 0);
        assert_eq!(b, 1);
    }

    #[test]
    fn add_subtask_keeps_status_new() {
        let mut t = task();
        t.add_subtask(String::from("a"), Priority::Low);
        assert_eq!(t.status(), Status::New);
    }

    #[test]
    fn add_subtask_to_in_progress_drops_to_new() {
        // Q8: manually started task, add first subtask → drops to New (recompute).
        let mut t = task();
        t.set_status(Status::InProgress).unwrap();
        t.add_subtask(String::from("a"), Priority::Low);
        assert_eq!(t.status(), Status::New);
    }

    #[test]
    fn complete_subtask_flips_task_in_progress() {
        let mut t = task();
        t.add_subtask(String::from("a"), Priority::Low);
        t.add_subtask(String::from("b"), Priority::Low);
        t.complete_subtask(0).unwrap();
        assert_eq!(t.status(), Status::InProgress);
    }

    #[test]
    fn complete_all_subtasks_flips_task_completed() {
        // Q2a: auto-flip Completed.
        let mut t = task();
        t.add_subtask(String::from("a"), Priority::Low);
        t.add_subtask(String::from("b"), Priority::Low);
        t.complete_subtask(0).unwrap();
        t.complete_subtask(1).unwrap();
        assert_eq!(t.status(), Status::Completed);
    }

    #[test]
    fn uncomplete_drops_back_to_in_progress() {
        // Q6a: un-complete one of all-completed → InProgress.
        let mut t = task();
        t.add_subtask(String::from("a"), Priority::Low);
        t.add_subtask(String::from("b"), Priority::Low);
        t.complete_subtask(0).unwrap();
        t.complete_subtask(1).unwrap();
        t.uncomplete_subtask(0).unwrap();
        assert_eq!(t.status(), Status::InProgress);
    }

    #[test]
    fn delete_completed_subtask_drops_to_new() {
        // Q5a: 2 subs, 1 completed, delete completed → all New → New.
        let mut t = task();
        t.add_subtask(String::from("a"), Priority::Low);
        t.add_subtask(String::from("b"), Priority::Low);
        t.complete_subtask(0).unwrap();
        assert_eq!(t.status(), Status::InProgress);
        t.delete_subtask(0).unwrap();
        assert_eq!(t.status(), Status::New);
    }

    #[test]
    fn delete_only_completed_subtask_keeps_completed() {
        // Q4: 1/1 completed → Completed. Delete it → empty list → sticky Completed.
        let mut t = task();
        t.add_subtask(String::from("a"), Priority::Low);
        t.complete_subtask(0).unwrap();
        assert_eq!(t.status(), Status::Completed);
        t.delete_subtask(0).unwrap();
        assert_eq!(t.status(), Status::Completed);
    }

    #[test]
    fn set_status_zero_subs_allows_anything() {
        let mut t = task();
        assert!(t.set_status(Status::InProgress).is_ok());
        assert!(t.set_status(Status::Completed).is_ok());
        assert!(t.set_status(Status::New).is_ok());
    }

    #[test]
    fn set_status_completed_blocked_when_subs_not_done() {
        let mut t = task();
        t.add_subtask(String::from("a"), Priority::Low);
        let err = t.set_status(Status::Completed).unwrap_err();
        assert!(matches!(err, AppError::TaskHasIncompleteSubtasks { .. }));
    }

    #[test]
    fn set_status_in_progress_allowed_when_all_subs_new() {
        // Q19: all New subs, can manually set InProgress.
        let mut t = task();
        t.add_subtask(String::from("a"), Priority::Low);
        assert!(t.set_status(Status::InProgress).is_ok());
        assert_eq!(t.status(), Status::InProgress);
    }

    #[test]
    fn set_status_new_blocked_when_any_sub_completed() {
        // Q19: once a sub is completed, can't manually go back to New.
        let mut t = task();
        t.add_subtask(String::from("a"), Priority::Low);
        t.add_subtask(String::from("b"), Priority::Low);
        t.complete_subtask(0).unwrap();
        let err = t.set_status(Status::New).unwrap_err();
        assert!(matches!(err, AppError::InvalidStatusTransition { .. }));
    }

    #[test]
    fn delete_missing_subtask_errors() {
        let mut t = task();
        let err = t.delete_subtask(99).unwrap_err();
        assert!(matches!(err, AppError::SubtaskNotFound { .. }));
    }
}
