use crate::AppError;
use crate::tui::app::{App, Screen, ScreenMode};
use crate::tui::effect::Effect;
use crate::tui::workspace::{WorkspaceFocus, WorkspaceScreen};

fn get_screen<'a>(app: &'a mut App) -> &'a mut WorkspaceScreen {
    let Screen::Workspace(s) = app.view_mut() else {
        unreachable!()
    };
    s
}

pub fn apply_effect(app: &mut App, effect: Effect) -> Result<(), AppError> {
    let project_count = app.workspace.projects.len();
    match effect {
        Effect::PushScreen => {
            todo!()
        }
        Effect::PopScreen => {
            app.should_quit = true;
        }
        // move previous
        Effect::SelectPrev => {
            let s = get_screen(app);
            match s.focus {
                WorkspaceFocus::NewProject if project_count > 0 => {
                    s.focus = WorkspaceFocus::Content;
                    s.list.select(Some(project_count - 1));
                }
                WorkspaceFocus::NewProject => {}
                WorkspaceFocus::Content => {
                    s.list.select_previous();
                }
            }
        }
        // move next
        Effect::SelectNext => {
            let s = get_screen(app);
            match s.focus {
                WorkspaceFocus::Content => {
                    if s.list.selected() == Some(project_count.saturating_sub(1)) {
                        s.focus = WorkspaceFocus::NewProject;
                        s.list.select(None);
                    } else {
                        s.list.select_next();
                    }
                }
                WorkspaceFocus::NewProject => {}
            };
        }
        // Edit
        Effect::EnterInsert => {
            let s = get_screen(app);
            s.mode = ScreenMode::Insert(String::new());
            s.focus = WorkspaceFocus::NewProject;
        }
        Effect::CancelInsert => {
            let s = get_screen(app);
            s.mode = ScreenMode::Navigate;
        }
        Effect::DraftPush(c) => {
            let s = get_screen(app);
            if let ScreenMode::Insert(draft) = &mut s.mode {
                draft.push(c);
            }
        }
        Effect::DraftPop => {
            let s = get_screen(app);
            if let ScreenMode::Insert(draft) = &mut s.mode {
                draft.pop();
            }
        }

        Effect::OpenOverlay(overlay) => app.overlay = Some(overlay),
        Effect::CloseOverlay => app.overlay = None,

        // workspace
        Effect::CreateProject { name } => {
            app.workspace.add_project(name);
            app.workspace.save()?;
            let len = app.workspace.projects.len();
            let s = get_screen(app);
            s.focus = WorkspaceFocus::Content;
            s.list.select(Some(len - 1));
        }
        Effect::DeleteProject { pid } => {
            app.workspace.delete_project(pid)?;
            app.workspace.save()?;
            let len = app.workspace.projects.len();
            let s = get_screen(app);
            if len == 0 {
                s.focus = WorkspaceFocus::NewProject;
                s.list.select(None);
            } else {
                // select the first project
                s.list.select(Some(len - 1));
                s.focus = WorkspaceFocus::Content;
            }
        }
        Effect::CreateTask => {}
        Effect::DeleteTask => {}
        Effect::ToggleTaskStatus => {}
    };

    Ok(())
}
