use serde::{Deserialize, Serialize};
use std::fmt;

/// Lifecycle state of a task.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, clap::ValueEnum)]
pub enum Status {
    /// Task has been created but work has not started.
    New,
    /// Task is actively being worked on.
    #[value(name = "in-progress")]
    InProgress,
    /// Task has been finished.
    Completed,
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
