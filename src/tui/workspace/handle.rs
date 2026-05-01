use super::{WorkspaceFocus, WorkspaceScreen};
use crate::tui::app::ScreenMode;
use crate::tui::effect::Effect;
use crate::tui::intent::{AppIntent, EditIntent, Intent, NavIntent};
use crate::tui::overlay::{ConfirmOverlay, Overlay};
use crate::workspace::Workspace;
use crossterm::event::{KeyCode, KeyEvent};

pub fn get_intent(
    screen: &WorkspaceScreen,
    key: &KeyEvent,
    last_key: Option<KeyCode>,
) -> Option<Intent> {
    match (&screen.mode, &screen.focus, key.code) {
        // Go back
        (ScreenMode::Navigate, _, KeyCode::Char('q') | KeyCode::Esc) => {
            Some(Intent::Nav(NavIntent::Back))
        }
        // move down
        (ScreenMode::Navigate, _, KeyCode::Char('j')) => Some(Intent::Nav(NavIntent::Down)),
        // move up
        (ScreenMode::Navigate, _, KeyCode::Char('k')) => Some(Intent::Nav(NavIntent::Up)),
        // Enter into a project
        (ScreenMode::Navigate, WorkspaceFocus::Content, KeyCode::Char('l') | KeyCode::Enter) => {
            Some(Intent::Nav(NavIntent::Enter))
        }
        // Enter Edit mode to create a project from content
        (ScreenMode::Navigate, WorkspaceFocus::Content, KeyCode::Char('i')) => {
            Some(Intent::Edit(EditIntent::Start))
        }
        // Enter Edit mode to create a project NewProject
        (
            ScreenMode::Navigate,
            WorkspaceFocus::NewProject,
            KeyCode::Char('l') | KeyCode::Char('i') | KeyCode::Enter,
        ) => Some(Intent::Edit(EditIntent::Start)),

        // DELETE
        (ScreenMode::Navigate, WorkspaceFocus::Content, KeyCode::Char('d')) => {
            if Some(key.code) == last_key {
                Some(Intent::App(AppIntent::RequestDelete))
            } else {
                None
            }
        }

        // INSERT
        // Add char to current Project
        (ScreenMode::Insert(_), WorkspaceFocus::NewProject, KeyCode::Char(c)) => {
            Some(Intent::Edit(EditIntent::NewChar(c)))
        }
        // delete char
        (ScreenMode::Insert(_), WorkspaceFocus::NewProject, KeyCode::Backspace) => {
            Some(Intent::Edit(EditIntent::DeleteChar))
        }
        // Submit char
        (ScreenMode::Insert(_), WorkspaceFocus::NewProject, KeyCode::Enter) => {
            Some(Intent::Edit(EditIntent::Submit))
        }
        (ScreenMode::Insert(_), WorkspaceFocus::NewProject, KeyCode::Esc) => {
            Some(Intent::Edit(EditIntent::Cancel))
        }
        _ => None,
    }
}

pub fn get_effect(screen: &WorkspaceScreen, intent: &Intent, workspace: &Workspace) -> Vec<Effect> {
    match intent {
        // natvigation
        Intent::Nav(NavIntent::Up) => vec![Effect::SelectPrev],
        Intent::Nav(NavIntent::Down) => vec![Effect::SelectNext],
        // figure out what kind of Screen
        Intent::Nav(NavIntent::Enter) => vec![Effect::PushScreen],
        Intent::Nav(NavIntent::Back) => vec![Effect::PopScreen],

        // insert
        Intent::Edit(EditIntent::Start) => vec![Effect::EnterInsert],
        Intent::Edit(EditIntent::Cancel) => vec![Effect::CancelInsert],
        Intent::Edit(EditIntent::NewChar(c)) => vec![Effect::DraftPush(*c)],
        Intent::Edit(EditIntent::DeleteChar) => vec![Effect::DraftPop],
        Intent::Edit(EditIntent::Submit) => match &screen.mode {
            ScreenMode::Insert(name) if !name.trim().is_empty() => {
                vec![
                    Effect::CreateProject {
                        name: name.trim().to_string(),
                    },
                    Effect::CancelInsert,
                ]
            }
            _ => vec![Effect::CancelInsert],
        },

        Intent::App(AppIntent::RequestDelete) => {
            if let Some(index) = screen.list.selected() {
                let p = &workspace.projects[index];
                let side_effect = Effect::DeleteProject { pid: p.id };
                let prompt = format!("Want to delete project: {}", p.name);
                let overlay = ConfirmOverlay::new(prompt, side_effect);
                vec![Effect::OpenOverlay(Overlay::Confirm(overlay))]
            } else {
                vec![]
            }
        }
        _ => vec![],
    }
}
