//! Post-state verbs.
//!
//! An `Effect` carries the verb plus everything `apply` needs to perform it.
//! Every state change in the TUI flows through `App::apply(Effect)`. That
//! single mutation surface is what lets persistence (`workspace.save()`) be
//! a single line at the bottom of `apply` — impossible to forget.

use super::app::TreeRowId;
use super::draft::InsertTarget;
use super::overlay::Overlay;

#[derive(Debug)]
pub enum Effect {
    // ── runtime ──
    Quit,

    // ── focus / cursor ──
    NextFocus,
    SelectNextProject,
    SelectPrevProject,
    SelectProject { p_idx: usize },
    TreeMoveDown,
    TreeMoveUp,
    /// `l`: expand on collapsed parent / drill on expanded parent / no-op on leaf.
    TreeExpandOrDescend,
    /// `h`: collapse on expanded parent / move-to-parent on subtask / no-op on leaf.
    TreeCollapseOrAscend,

    // ── mode / draft ──
    EnterInsert(InsertTarget),
    CancelInsert,
    DraftPush(char),
    DraftPop,

    // ── overlay ──
    OpenOverlay(Overlay),
    CloseOverlay,

    // ── error display ──
    /// Set the help-line error. Apply auto-clears the error on every other
    /// effect, so an explicit `ClearError` is unnecessary.
    ShowError(String),

    // ── domain mutations ──
    CreateProject {
        name: String,
    },
    CreateTask {
        p_idx: usize,
        description: String,
    },
    DeleteTask {
        p_idx: usize,
        task_id: u32,
    },
    CreateSubtask {
        p_idx: usize,
        task_id: u32,
        description: String,
    },
    DeleteSubtask {
        p_idx: usize,
        task_id: u32,
        sub_id: u32,
    },
    /// Cycle status / toggle completion. Effect knows exactly *what* to
    /// toggle (resolver already classified the cursor row).
    ToggleStatus(TreeRowId),

    /// Pumped by apply: take overlay's `on_confirm` and execute it. Avoids
    /// needing `Effect: Clone` just to pass the boxed effect back through
    /// resolve (which only holds `&App`).
    ConfirmYes,
}
