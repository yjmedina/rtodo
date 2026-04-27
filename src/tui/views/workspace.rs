use crate::{
    AppError,
    tui::app::{Action, App, Mode, View},
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Style},
    widgets::{Block, List, ListItem, Paragraph},
};

pub fn handle_key_event(app: &App, key: &KeyEvent) -> Option<Action> {
    match app.mode {
        Mode::Navigate => match key.code {
            KeyCode::Char('j') => {
                if app.project_list_state.selected()
                    == Some(app.workspace.projects.len().saturating_sub(1))
                {
                    Some(Action::SetMode(Mode::FocusingInput))
                } else {
                    Some(Action::NextProject)
                }
            }
            KeyCode::Char('k') => Some(Action::PreviousProject),
            KeyCode::Char('l') | KeyCode::Enter => {
                app.project_list_state
                    .selected()
                    .map(|selected| Action::OpenProject {
                        id: app.workspace.projects[selected].id,
                    })
            }
            KeyCode::Char('d') => {
                if app.last_key == Some(KeyCode::Char('d'))
                    && app.project_list_state.selected().is_some()
                {
                    Some(Action::SetMode(Mode::Confirmation))
                } else {
                    None
                }
            }
            _ => None,
        },
        Mode::FocusingInput => match key.code {
            KeyCode::Char('k') => Some(Action::SetMode(Mode::Navigate)),
            KeyCode::Char('l') | KeyCode::Enter => Some(Action::SetMode(Mode::CreateProject)),
            _ => None,
        },
        Mode::CreateProject => match key.code {
            KeyCode::Char(c) => Some(Action::UpdateProjectDraft { c }),
            KeyCode::Backspace => Some(Action::PopProjectDraft),
            KeyCode::Enter => Some(Action::CreateProject),
            KeyCode::Esc => Some(Action::SetMode(Mode::Navigate)),
            _ => None,
        },
        Mode::Confirmation => match key.code {
            KeyCode::Char('y') => {
                app.project_list_state.selected().map(|selected| Action::DeleteProject {
                    id: app.workspace.projects[selected].id,
                })
            }
            KeyCode::Char('n') | KeyCode::Esc => Some(Action::SetMode(Mode::Navigate)),
            _ => None,
        },
    }
}

pub fn handle_action(app: &mut App, action: Action) -> Result<View, AppError> {
    match action {
        Action::NextProject => {
            app.project_list_state.select_next();
        }
        Action::PreviousProject => {
            app.project_list_state.select_previous();
        }
        Action::SetMode(mode) => {
            match &mode {
                Mode::FocusingInput => {
                    app.project_list_state.select(None);
                }
                Mode::Navigate => {
                    if app.project_list_state.selected().is_none() {
                        let last = app.workspace.projects.len().saturating_sub(1);
                        app.project_list_state.select(Some(last));
                    }
                    app.draft = None;
                }
                Mode::CreateProject => {
                    app.draft = Some(String::new());
                }
                Mode::Confirmation => {}
            }
            app.mode = mode;
        }
        Action::UpdateProjectDraft { c } => {
            if let Some(d) = app.draft.as_mut() { d.push(c); }
        }
        Action::PopProjectDraft => {
            if let Some(d) = app.draft.as_mut() { d.pop(); }
        }
        Action::CreateProject => {
            if let Some(name) = app.draft.take() {
                app.workspace.add_project(name);
                app.mode = Mode::Navigate;
            }
        }
        Action::DeleteProject { id } => {
            let selected = app.project_list_state.selected().unwrap_or(0);
            app.workspace.delete_project(id)?;
            let new_len = app.workspace.projects.len();
            if new_len == 0 {
                app.project_list_state.select(None);
            } else {
                app.project_list_state
                    .select(Some(selected.min(new_len - 1)));
            }
            app.mode = Mode::Navigate;
        }
        Action::OpenProject { id } => return Ok(View::Project { pid: id }),
    }
    Ok(View::Workspace)
}

pub fn view(frame: &mut Frame, app: &mut App) {
    let [title_area, content_area, _, input_area, help_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Length(3),
    ])
    .areas(frame.area());

    frame.render_widget(
        Paragraph::new("An awesome todo list").block(Block::bordered().title("rtodo")),
        title_area,
    );

    let project_widgets: Vec<ListItem<'_>> = app
        .workspace
        .projects
        .iter()
        .map(|p| {
            ListItem::new(format!(
                "{:<40} {}/{}",
                p.name,
                p.complete_tasks(),
                p.task_count()
            ))
        })
        .collect();

    let project_widget = List::new(project_widgets)
        .block(Block::bordered().title("Projects"))
        .highlight_style(Style::new().bg(Color::Cyan).fg(Color::Black))
        .highlight_symbol("▶  ");

    frame.render_stateful_widget(project_widget, content_area, &mut app.project_list_state);

    let input_text = match &app.mode {
        Mode::CreateProject => format!("Project Name: {}_", app.draft.as_deref().unwrap_or("")),
        Mode::FocusingInput => "+ New project...".to_string(),
        _ => String::new(),
    };

    let input_border = if matches!(app.mode, Mode::FocusingInput | Mode::CreateProject) {
        Block::bordered().style(Style::new().fg(Color::Cyan))
    } else {
        Block::bordered()
    };

    frame.render_widget(Paragraph::new(input_text).block(input_border), input_area);

    let help_text = match &app.mode {
        Mode::Confirmation => {
            let name = app
                .project_list_state
                .selected()
                .and_then(|i| app.workspace.projects.get(i))
                .map(|p| p.name.as_str())
                .unwrap_or("project");
            format!("⚠  Delete \"{}\"?   y confirm   n cancel", name)
        }
        Mode::FocusingInput => "↵/l create   k back   q quit".to_string(),
        Mode::CreateProject => "↵ create   Esc cancel".to_string(),
        Mode::Navigate => "j/k navigate   ↵/l open   dd delete   q quit".to_string(),
    };

    frame.render_widget(
        Paragraph::new(help_text).block(Block::bordered().title("Help")),
        help_area,
    );
}
