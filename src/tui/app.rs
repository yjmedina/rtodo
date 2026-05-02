//! Runtime state for the TUI.
//!
//! `App` owns *runtime* state (cursor positions, mode, overlay, error) and
//! borrows the `Workspace` for *domain* state (the only thing persisted to
//! disk).
//!
//! Types here are intentionally dumb — no behavior. Key handling is in
//! [`super::intent`], the resolver in [`super::resolve`], mutations in
//! [`super::apply`], rendering in [`super::render`].

use std::collections::HashSet;

use crossterm::event::KeyCode;

use crate::workspace::Workspace;

use super::draft::Draft;
use super::overlay::Overlay;

pub struct App<'a> {
    pub workspace: &'a mut Workspace,
    pub screen: ProjectScreen,
    pub overlay: Option<Overlay>,
    /// Ephemeral message shown in the help row. Cleared on next keypress.
    pub error: Option<String>,
    /// Last key for two-key chords (`dd`).
    pub last_key: Option<KeyCode>,
    pub should_quit: bool,
}

impl<'a> App<'a> {
    pub fn new(workspace: &'a mut Workspace) -> Self {
        let screen = ProjectScreen::new(workspace);
        App {
            workspace,
            screen,
            overlay: None,
            error: None,
            last_key: None,
            should_quit: false,
        }
    }
}

/// The single TUI screen. Contains the sidebar cursor (project list) plus
/// the tree state for the currently-open project.
pub struct ProjectScreen {
    /// Index into `workspace.projects`, or `None` when the workspace is empty.
    pub p_idx: Option<usize>,
    /// Cursor within the sidebar's project list (own state — does not have to
    /// match `p_idx` while the user is browsing).
    pub sidebar_cursor: usize,
    pub tree: TreeState,
    pub focus: ProjectFocus,
    pub mode: ScreenMode,
}

impl ProjectScreen {
    pub fn new(workspace: &Workspace) -> Self {
        // Boot to the workspace's active project if one is set and still
        // exists; otherwise fall back to the first project (or `None` when
        // empty). Keeps the user in their last working project across runs.
        let p_idx = workspace
            .active_project_id
            .and_then(|id| workspace.projects.iter().position(|p| p.id == id))
            .or_else(|| (!workspace.projects.is_empty()).then_some(0));

        // Seed cursor on first task of the active project so the user never
        // has to press `j` just to see a selection highlight.
        let initial_cursor = p_idx.and_then(|i| {
            workspace.projects[i].tasks.first().map(|t| TreeRowId::Task(t.id))
        });
        Self {
            p_idx,
            sidebar_cursor: p_idx.unwrap_or(0),
            tree: TreeState {
                expanded: std::collections::HashSet::new(),
                cursor: initial_cursor,
            },
            focus: ProjectFocus::Tree,
            mode: ScreenMode::Navigate,
        }
    }
}

/// Which pane the keystrokes apply to.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ProjectFocus {
    Sidebar,
    Tree,
}

impl ProjectFocus {
    /// `Tab` cycle.
    pub fn next(self) -> Self {
        match self {
            ProjectFocus::Sidebar => ProjectFocus::Tree,
            ProjectFocus::Tree => ProjectFocus::Sidebar,
        }
    }
}

/// How keys are interpreted on the focused pane.
///
/// `Insert` carries the draft as data: the type system rejects "Insert mode
/// with no draft" and "stale draft outside Insert".
pub enum ScreenMode {
    Navigate,
    Insert(Draft),
}

/// Cursor within the task tree.
#[derive(Default)]
pub struct TreeState {
    /// Task ids whose subtasks are currently shown.
    pub expanded: HashSet<u32>,
    /// Where the cursor sits.
    pub cursor: Option<TreeRowId>,
}

/// Stable identity of a row in the tree.
///
/// Rows are addressed by domain id, not by row index, so the cursor survives
/// expand/collapse and create/delete without jumping.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TreeRowId {
    Task(u32),
    Subtask { task: u32, sub: u32 },
}
