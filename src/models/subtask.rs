use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::models::priority::Priority;
use crate::models::task::CREATED_AT_FORMAT;

#[derive(Debug, Serialize, Deserialize)]
pub struct Subtask {
    pub id: u32,
    pub description: String,
    pub priority: Priority,
    pub completed: bool,
    pub created_at: DateTime<Utc>,
}

impl Subtask {
    pub fn new(id: u32, description: String, priority: Priority) -> Self {
        Subtask {
            id,
            description,
            priority,
            completed: false,
            created_at: Utc::now(),
        }
    }

    pub fn complete(&mut self) {
        self.completed = true;
    }

    pub fn uncomplete(&mut self) {
        self.completed = false;
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

impl fmt::Display for Subtask {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let marker = if self.completed { "x" } else { " " };
        write!(
            f,
            "[{}] [{}] {} ({}) -- ({})",
            self.id,
            marker,
            self.description,
            self.priority,
            self.created_at.format(CREATED_AT_FORMAT)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_defaults_to_incomplete() {
        let s = Subtask::new(0, String::from("write docs"), Priority::Low);
        assert!(!s.completed);
    }

    #[test]
    fn complete_sets_flag() {
        let mut s = Subtask::new(0, String::from("write docs"), Priority::Low);
        s.complete();
        assert!(s.completed);
    }

    #[test]
    fn uncomplete_clears_flag() {
        let mut s = Subtask::new(0, String::from("write docs"), Priority::Low);
        s.complete();
        s.uncomplete();
        assert!(!s.completed);
    }

    #[test]
    fn edit_mutates_fields() {
        let mut s = Subtask::new(0, String::from("a"), Priority::Low);
        s.edit(Some(String::from("b")), Some(Priority::High));
        assert_eq!(s.description, "b");
        assert_eq!(s.priority, Priority::High);
    }

    #[test]
    fn edit_partial_keeps_others() {
        let mut s = Subtask::new(0, String::from("a"), Priority::Low);
        s.edit(Some(String::from("b")), None);
        assert_eq!(s.description, "b");
        assert_eq!(s.priority, Priority::Low);
    }
}
