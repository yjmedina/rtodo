use crate::AppError;
use crate::models::{Priority, Project};
use crate::tui::app::{App, Screen, ScreenMode};
use crate::tui::effect::Effect;
use crate::tui::project::{ProjectFocus, ProjectScreen};

fn get_screen<'a>(app: &'a mut App) -> &'a mut ProjectScreen {
    let Screen::Project(s) = app.view_mut() else {
        unreachable!()
    };
    s
}

fn get_project<'a>(app: &'a mut App) -> &'a Project {
    let Screen::Project(s) = app.view() else {
        unreachable!()
    };
    let p_idx = s.p_idx;

    &app.workspace.projects[p_idx]
}

fn get_project_mut<'a>(app: &'a mut App) -> &'a mut Project {
    let Screen::Project(s) = app.view() else {
        unreachable!()
    };
    let p_idx = s.p_idx;

    &mut app.workspace.projects[p_idx]
}

pub fn apply_effect(app: &mut App, effect: Effect) -> Result<(), AppError> {
    match effect {
        Effect::PushScreen => {
            todo!()
        }
        Effect::PopScreen => {
            app.screens.pop();
        }
        // move previous
        Effect::SelectPrev => {
            let project = get_project(app);
            let len = project.task_count();

            let s = get_screen(app);
            match s.focus {
                ProjectFocus::NewTask if len > 0 => {
                    s.focus = ProjectFocus::TaskList;
                    s.list.select(Some(len - 1));
                }
                ProjectFocus::NewTask => {}
                ProjectFocus::TaskList => {
                    s.list.select_previous();
                }
                _ => {}
            }
        }
        // move next
        Effect::SelectNext => {
            let project = get_project(app);
            let len = project.task_count();
            let s = get_screen(app);
            if let ProjectFocus::TaskList = s.focus {
                if s.list.selected() == Some(len.saturating_sub(1)) {
                    s.focus = ProjectFocus::NewTask;
                    s.list.select(None);
                } else {
                    s.list.select_next();
                }
            };
        }
        // Edit
        Effect::EnterInsert => {
            let s = get_screen(app);
            s.mode = ScreenMode::Insert(String::new());
            s.focus = ProjectFocus::NewTask;
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
        Effect::CreateTask { description } => {
            let project = get_project_mut(app);
            project.add_task(description, Priority::Medium, None)?;
            let len = project.task_count();

            let s = get_screen(app);
            s.focus = ProjectFocus::TaskList;
            s.list.select(Some(len - 1));
            app.workspace.save()?;
        }
        Effect::DeleteTask { tid } => {
            let project = get_project_mut(app);
            project.delete_task(tid)?;
            let len = project.task_count();
            let s = get_screen(app);
            if len == 0 {
                s.focus = ProjectFocus::NewTask;
                s.list.select(None);
            } else {
                // select the first project
                s.list.select(Some(len - 1));
                s.focus = ProjectFocus::TaskList;
            }
        }
        Effect::CreateProject { .. } => {}
        Effect::DeleteProject { .. } => {}
        Effect::ToggleTaskStatus => {}
    };

    Ok(())
}
