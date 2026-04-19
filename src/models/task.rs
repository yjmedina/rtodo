use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

pub use crate::models::priority::Priority;
pub use crate::models::status::Status;

pub const CREATED_AT_FORMAT: &str = "%Y-%m-%d";
/// Implements [`fmt::Debug`] via derive, which allows `println!("{:#?}", task)`.
#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    /// Unique identifier within the parent project.
    pub id: u32,
    /// ID of the parent task, or `None` if this is a top-level task.
    pub parent_id: Option<u32>,
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
    pub fn new(
        id: u32,
        description: String,
        priority: Priority,
        status: Status,
        parent_id: Option<u32>,
    ) -> Self {
        Task {
            id,
            parent_id,
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
