use super::{ProjectFocus, ProjectScreen};
use crate::tui::app::ScreenMode;
use crate::tui::effect::Effect;
use crate::tui::intent::{AppIntent, EditIntent, Intent, NavIntent};
use crate::tui::overlay::{ConfirmOverlay, Overlay};
use crate::workspace::Workspace;
use crossterm::event::{KeyCode, KeyEvent};

pub fn get_intent(
    screen: &ProjectScreen,
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
        (ScreenMode::Navigate, ProjectFocus::TaskList, KeyCode::Char('l') | KeyCode::Enter) => {
            Some(Intent::Nav(NavIntent::Enter))
        }
        // Enter Edit mode to create a project from content
        (ScreenMode::Navigate, ProjectFocus::TaskList, KeyCode::Char('i')) => {
            Some(Intent::Edit(EditIntent::Start))
        }
        // DELETE
        (ScreenMode::Navigate, ProjectFocus::TaskList, KeyCode::Char('d')) => {
            if Some(key.code) == last_key {
                Some(Intent::App(AppIntent::RequestDelete))
            } else {
                None
            }
        }

        // INSERT
        // Enter Edit mode to create a project NewProject
        (
            ScreenMode::Navigate,
            ProjectFocus::NewTask,
            KeyCode::Char('l') | KeyCode::Char('i') | KeyCode::Enter,
        ) => Some(Intent::Edit(EditIntent::Start)),

        // Add char to current task
        (
            ScreenMode::Insert(_),
            ProjectFocus::NewTask | ProjectFocus::NewSubtask,
            KeyCode::Char(c),
        ) => Some(Intent::Edit(EditIntent::NewChar(c))),
        // delete char
        (
            ScreenMode::Insert(_),
            ProjectFocus::NewTask | ProjectFocus::NewSubtask,
            KeyCode::Backspace,
        ) => Some(Intent::Edit(EditIntent::DeleteChar)),
        // Submit char
        (
            ScreenMode::Insert(_),
            ProjectFocus::NewTask | ProjectFocus::NewSubtask,
            KeyCode::Enter,
        ) => Some(Intent::Edit(EditIntent::Submit)),
        (ScreenMode::Insert(_), ProjectFocus::NewTask | ProjectFocus::NewSubtask, KeyCode::Esc) => {
            Some(Intent::Edit(EditIntent::Cancel))
        }
        _ => None,
    }
}

pub fn get_effect(screen: &ProjectScreen, intent: &Intent, workspace: &Workspace) -> Vec<Effect> {
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
            ScreenMode::Insert(description) if !description.trim().is_empty() => {
                vec![
                    Effect::CreateTask {
                        description: description.trim().to_string(),
                    },
                    Effect::CancelInsert,
                ]
            }
            _ => vec![Effect::CancelInsert],
        },

        Intent::App(AppIntent::RequestDelete) => {
            if let Some(index) = screen.list.selected() {
                let project = &workspace.projects[screen.p_idx];
                let task = &project.tasks[index];
                let side_effect = Effect::DeleteTask { tid: task.id };
                let prompt = format!(
                    "Want to delete task {}\n This operation will also delete all subtasks",
                    task.description
                );
                let overlay = ConfirmOverlay::new(prompt, side_effect);
                vec![Effect::OpenOverlay(Overlay::Confirm(overlay))]
            } else {
                vec![]
            }
        }
        _ => vec![],
    }
}
