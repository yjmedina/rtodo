//! Pure tree flattening for the Project pane.
//!
//! Given a `Project` and the current expanded set, produce a `Vec<TreeRow>`
//! ready for the renderer to walk linearly. No mutation, no I/O — trivially
//! testable.

use std::collections::HashSet;

use crate::models::Project;

use super::app::TreeRowId;

#[derive(Debug, Clone)]
pub struct TreeRow {
    pub id: TreeRowId,
    pub depth: u8,
    /// Last sibling among its peers — drives `└─` vs `├─` glyph choice.
    pub last_in_group: bool,
    /// Visual hint state for parents.
    pub kind: RowKind,
}

#[derive(Debug, Clone)]
pub enum RowKind {
    /// Task with no subtasks.
    Leaf,
    /// Task with subtasks; bool tells whether they are currently visible.
    Parent { expanded: bool },
    /// A subtask row.
    Subtask,
    /// In-tree draft placeholder (the inline insert text).
    Drafting { text: String },
}

pub fn flatten(project: &Project, expanded: &HashSet<u32>) -> Vec<TreeRow> {
    let mut out = Vec::new();
    let n = project.tasks.len();
    for (i, task) in project.tasks.iter().enumerate() {
        let last_in_group = i + 1 == n;
        let has_subs = !task.subtasks.is_empty();
        let is_expanded = expanded.contains(&task.id);

        let kind = if has_subs {
            RowKind::Parent {
                expanded: is_expanded,
            }
        } else {
            RowKind::Leaf
        };
        out.push(TreeRow {
            id: TreeRowId::Task(task.id),
            depth: 0,
            last_in_group,
            kind,
        });

        if has_subs && is_expanded {
            let m = task.subtasks.len();
            for (j, sub) in task.subtasks.iter().enumerate() {
                out.push(TreeRow {
                    id: TreeRowId::Subtask {
                        task: task.id,
                        sub: sub.id,
                    },
                    depth: 1,
                    last_in_group: j + 1 == m,
                    kind: RowKind::Subtask,
                });
            }
        }
    }
    out
}

/// Find the linear index of a cursor inside a flattened tree.
pub fn cursor_index(rows: &[TreeRow], cursor: TreeRowId) -> Option<usize> {
    rows.iter().position(|r| r.id == cursor)
}
