use serde::{Deserialize, Serialize};
use std::fmt;

/// Importance level of a task.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, clap::ValueEnum)]
pub enum Priority {
    /// Low importance — tackle after `Medium` and `High` tasks.
    Low,
    /// Default importance level.
    Medium,
    /// High importance — shown with a `!` marker in listings.
    High,
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
