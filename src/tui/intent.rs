//! Pre-state interpretation of a key event.
//!
//! `Intent` says *what the user meant* given the current focus and mode —
//! before any workspace lookup. `keymap` is a pure function from
//! `(App, KeyEvent)` to `Option<Intent>`; it never mutates and never reads
//! domain data.
//!
//! Splitting "what was meant" (Intent) from "what gets done" (Effect) means
//! `Submit` (commit a draft) and `Confirm(Yes)` (approve a pending action)
//! are distinct intents, each routed independently. No implicit dispatch.

use crossterm::event::{KeyCode, KeyEvent};

use super::app::{App, ProjectFocus, ScreenMode};


#[derive(Debug)]
pub enum Intent {
    Nav(NavIntent),
    Edit(EditIntent),
    Confirm(ConfirmIntent),
    App(AppIntent),
}

#[derive(Debug)]
pub enum NavIntent {
    Up,
    Down,
    /// `l` — drill into / expand.
    Right,
    /// `h` — exit / collapse / jump up.
    Left,
    /// `Tab`.
    NextFocus,
}

#[derive(Debug)]
pub enum EditIntent {
    /// Begin an insert; the resolver fills in target details using cursor.
    Start(InsertKind),
    Cancel,
    Submit,
    Char(char),
    Backspace,
}

/// What the keymap saw. The resolver turns this into a concrete `InsertTarget`
/// using the current cursor — keymap stays pure (no workspace lookups).
#[derive(Debug, Clone, Copy)]
pub enum InsertKind {
    /// `i` — sibling at cursor level.
    Sibling,
    /// `I` — child of cursor (only valid on a task).
    Child,
    /// Sidebar `i` — new project.
    Project,
}

#[derive(Debug)]
pub enum ConfirmIntent {
    Yes,
    No,
}

#[derive(Debug)]
pub enum AppIntent {
    Quit,
    /// Triggered by `dd`.
    RequestDelete,
    /// `space` — toggle status / completion at cursor.
    ToggleStatus,
}

/// Translate a key into an intent.
///
/// Pure: no mutation, no workspace read. Uses `App` only for current focus,
/// mode, overlay, and `last_key` (for chords).
pub fn keymap(app: &App, key: &KeyEvent) -> Option<Intent> {
    // Overlay intercepts first — its only intents are confirm yes/no.
    if app.overlay.is_some() {
        return Some(match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => Intent::Confirm(ConfirmIntent::Yes),
            _ => Intent::Confirm(ConfirmIntent::No),
        });
    }

    // Insert mode: text input + escape/enter routing.
    if let ScreenMode::Insert(_) = app.screen.mode {
        return match key.code {
            KeyCode::Esc => Some(Intent::Edit(EditIntent::Cancel)),
            KeyCode::Enter => Some(Intent::Edit(EditIntent::Submit)),
            KeyCode::Backspace => Some(Intent::Edit(EditIntent::Backspace)),
            KeyCode::Char(c) => Some(Intent::Edit(EditIntent::Char(c))),
            _ => None,
        };
    }

    // Navigate mode — keys depend on focus.
    match (app.screen.focus, key.code) {
        (_, KeyCode::Char('q')) => Some(Intent::App(AppIntent::Quit)),
        (_, KeyCode::Tab) => Some(Intent::Nav(NavIntent::NextFocus)),
        (_, KeyCode::Char('j')) | (_, KeyCode::Down) => Some(Intent::Nav(NavIntent::Down)),
        (_, KeyCode::Char('k')) | (_, KeyCode::Up) => Some(Intent::Nav(NavIntent::Up)),
        (_, KeyCode::Char('l')) | (_, KeyCode::Right) => Some(Intent::Nav(NavIntent::Right)),
        (_, KeyCode::Char('h')) | (_, KeyCode::Left) => Some(Intent::Nav(NavIntent::Left)),

        (ProjectFocus::Sidebar, KeyCode::Char('i')) => {
            Some(Intent::Edit(EditIntent::Start(InsertKind::Project)))
        }
        (ProjectFocus::Sidebar, KeyCode::Enter) => Some(Intent::Nav(NavIntent::Right)),

        (ProjectFocus::Tree, KeyCode::Char('i')) => {
            Some(Intent::Edit(EditIntent::Start(InsertKind::Sibling)))
        }
        (ProjectFocus::Tree, KeyCode::Char('I')) => {
            Some(Intent::Edit(EditIntent::Start(InsertKind::Child)))
        }
        (ProjectFocus::Tree, KeyCode::Char(' ')) => Some(Intent::App(AppIntent::ToggleStatus)),
        (ProjectFocus::Tree, KeyCode::Char('d')) => {
            // `dd` chord.
            if app.last_key == Some(KeyCode::Char('d')) {
                Some(Intent::App(AppIntent::RequestDelete))
            } else {
                None
            }
        }

        _ => None,
    }
}
