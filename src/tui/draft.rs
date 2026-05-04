//! Insert-mode draft + the target it will become on submit.
//!
//! `InsertTarget` carries the *origin* of the draft: which cursor position
//! started the insert and what kind of item the resolver should produce.
//! Embedding it in the draft (rather than re-deriving from cursor at submit
//! time) makes the resolver a pure function of `(intent, draft, workspace)`
//! with no implicit dependence on UI state that may have shifted.

#[derive(Debug, Clone)]
pub struct Draft {
    pub text: String,
    pub target: InsertTarget,
}

impl Draft {
    pub fn new(target: InsertTarget) -> Self {
        Self {
            text: String::new(),
            target,
        }
    }
}

/// What `Submit` should create.
///
/// Two-level model: tasks live under projects, subtasks live under tasks.
/// There is no "subtask of a subtask"; that combination is structurally
/// unrepresentable here.
#[derive(Debug, Clone, Copy)]
pub enum InsertTarget {
    /// Sidebar `i`.
    Project,
    /// Tree `i` on an empty project.
    TaskRoot,
    /// Tree `i` with cursor on a task — append a sibling at task level.
    TaskSibling { after: u32 },
    /// Tree `I` on a task, or `i` on a subtask (sibling under same parent).
    Subtask { task: u32 },
}
