//! Translate an `Intent` into a sequence of `Effect`s.
//!
//! Pure function of `(&App, &Intent) -> Vec<Effect>`. Reads runtime state
//! and the workspace, produces the verb list. Never mutates.

use super::app::{App, ProjectFocus, ScreenMode, TreeRowId};
use super::draft::InsertTarget;
use super::effect::Effect;
use super::intent::{AppIntent, ConfirmIntent, EditIntent, InsertKind, Intent, NavIntent};
use super::overlay::{ConfirmOverlay, Overlay};

pub fn resolve(app: &App, intent: &Intent) -> Vec<Effect> {
    // Overlay confirm pump intercepts before screen logic.
    if app.overlay.is_some() {
        return match intent {
            Intent::Confirm(ConfirmIntent::Yes) => vec![Effect::ConfirmYes, Effect::CloseOverlay],
            _ => vec![Effect::CloseOverlay],
        };
    }

    // Insert mode: text editing + submit/cancel.
    if let ScreenMode::Insert(draft) = &app.screen.mode {
        return match intent {
            Intent::Edit(EditIntent::Cancel) => vec![Effect::CancelInsert],
            Intent::Edit(EditIntent::Backspace) => vec![Effect::DraftPop],
            Intent::Edit(EditIntent::Char(c)) => vec![Effect::DraftPush(*c)],
            Intent::Edit(EditIntent::Submit) => resolve_submit(app, draft),
            _ => Vec::new(),
        };
    }

    // Navigate mode.
    match app.screen.focus {
        ProjectFocus::Sidebar => resolve_sidebar(app, intent),
        ProjectFocus::Tree => resolve_tree(app, intent),
    }
}

// ── Sidebar ──────────────────────────────────────────────────────────────────

fn resolve_sidebar(app: &App, intent: &Intent) -> Vec<Effect> {
    match intent {
        Intent::App(AppIntent::Quit) => vec![Effect::Quit],
        Intent::Nav(NavIntent::NextFocus) => vec![Effect::NextFocus],
        Intent::Nav(NavIntent::Down) => vec![Effect::SelectNextProject],
        Intent::Nav(NavIntent::Up) => vec![Effect::SelectPrevProject],
        // Enter / l in sidebar — jump focus to tree.
        Intent::Nav(NavIntent::Right) => {
            let p_idx = app.screen.sidebar_cursor;
            let valid = p_idx < app.workspace.projects.len();
            if valid {
                vec![Effect::SelectProject { p_idx }, Effect::NextFocus]
            } else {
                Vec::new()
            }
        }
        Intent::Edit(EditIntent::Start(InsertKind::Project)) => {
            vec![Effect::EnterInsert(InsertTarget::Project)]
        }
        _ => Vec::new(),
    }
}

// ── Tree ─────────────────────────────────────────────────────────────────────

fn resolve_tree(app: &App, intent: &Intent) -> Vec<Effect> {
    match intent {
        Intent::App(AppIntent::Quit) => vec![Effect::Quit],
        Intent::Nav(NavIntent::NextFocus) => vec![Effect::NextFocus],
        Intent::Nav(NavIntent::Down) => vec![Effect::TreeMoveDown],
        Intent::Nav(NavIntent::Up) => vec![Effect::TreeMoveUp],
        Intent::Nav(NavIntent::Right) => vec![Effect::TreeExpandOrDescend],
        Intent::Nav(NavIntent::Left) => vec![Effect::TreeCollapseOrAscend],
        Intent::App(AppIntent::ToggleStatus) => match app.screen.tree.cursor {
            Some(id) => vec![Effect::ToggleStatus(id)],
            None => Vec::new(),
        },
        Intent::App(AppIntent::RequestDelete) => resolve_delete(app),
        Intent::Edit(EditIntent::Start(kind)) => resolve_insert_start(app, *kind),
        _ => Vec::new(),
    }
}

// ── Insert ────────────────────────────────────────────────────────────────────

fn resolve_insert_start(app: &App, kind: InsertKind) -> Vec<Effect> {
    // Empty workspace: `i` / `I` from tree → create a project first.
    if app.screen.p_idx.is_none() {
        if matches!(kind, InsertKind::Sibling | InsertKind::Child) {
            return vec![Effect::EnterInsert(InsertTarget::Project)];
        }
        return Vec::new();
    }

    let target = match (kind, app.screen.tree.cursor) {
        (InsertKind::Project, _) => InsertTarget::Project,
        (InsertKind::Sibling, None) | (InsertKind::Child, None) => InsertTarget::TaskRoot,
        (InsertKind::Sibling, Some(TreeRowId::Task(tid))) => {
            InsertTarget::TaskSibling { after: tid }
        }
        (InsertKind::Sibling, Some(TreeRowId::Subtask { task: tid, .. })) => {
            // `i` on subtask = new sibling subtask under same parent.
            InsertTarget::Subtask { task: tid }
        }
        (InsertKind::Child, Some(TreeRowId::Task(tid))) => InsertTarget::Subtask { task: tid },
        (InsertKind::Child, Some(TreeRowId::Subtask { .. })) => {
            // `I` on subtask — can't go deeper.
            return vec![Effect::ShowError(
                "subtasks can't have children — press `i` for a sibling".into(),
            )];
        }
    };

    // Sidebar `i` from tree focus: redirect to sidebar.
    if matches!(target, InsertTarget::Project) {
        return vec![
            Effect::NextFocus, // switch to sidebar
            Effect::EnterInsert(InsertTarget::Project),
        ];
    }

    // For sibling-at-cursor: auto-expand parent task if we're drafting a
    // subtask so the new row is visible.
    let mut effects: Vec<Effect> = Vec::new();
    if let InsertTarget::Subtask { task: tid } = target {
        if !app.screen.tree.expanded.contains(&tid) {
            // We'll insert to expanded in apply after create; EnterInsert is
            // enough for now — apply::CreateSubtask handles the expand.
        }
        let _ = tid; // suppress warning
    }
    effects.push(Effect::EnterInsert(target));
    effects
}

fn resolve_submit(app: &App, draft: &super::draft::Draft) -> Vec<Effect> {
    let text = draft.text.trim().to_string();
    if text.is_empty() {
        return vec![Effect::CancelInsert];
    }

    let p_idx = match app.screen.p_idx {
        Some(i) => i,
        None => {
            // p_idx will be set by CreateProject's apply arm.
            if matches!(draft.target, InsertTarget::Project) {
                return vec![Effect::CreateProject { name: text }];
            }
            return vec![Effect::CancelInsert];
        }
    };

    match draft.target {
        InsertTarget::Project => vec![Effect::CreateProject { name: text }],
        InsertTarget::TaskRoot | InsertTarget::TaskSibling { .. } => {
            vec![Effect::CreateTask {
                p_idx,
                description: text,
            }]
        }
        InsertTarget::Subtask { task: tid } => {
            vec![Effect::CreateSubtask {
                p_idx,
                task_id: tid,
                description: text,
            }]
        }
    }
}

// ── Delete ────────────────────────────────────────────────────────────────────

fn resolve_delete(app: &App) -> Vec<Effect> {
    let p_idx = match app.screen.p_idx {
        Some(i) => i,
        None => return Vec::new(),
    };

    match app.screen.tree.cursor {
        None => Vec::new(),

        Some(TreeRowId::Subtask {
            task: tid,
            sub: sid,
        }) => {
            // Subtask delete is immediate (no confirm).
            vec![Effect::DeleteSubtask {
                p_idx,
                task_id: tid,
                sub_id: sid,
            }]
        }

        Some(TreeRowId::Task(tid)) => {
            let name = app
                .workspace
                .projects
                .get(p_idx)
                .and_then(|p| p.tasks.iter().find(|t| t.id == tid))
                .map(|t| t.description.as_str())
                .unwrap_or("task");
            let prompt = format!("Delete task \"{}\"? [y/n]", name);
            let on_confirm = Box::new(Effect::DeleteTask {
                p_idx,
                task_id: tid,
            });
            vec![Effect::OpenOverlay(Overlay::Confirm(ConfirmOverlay {
                prompt,
                on_confirm,
            }))]
        }
    }
}
