use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, List, ListItem, Paragraph},
};

use crate::AppError;
use super::app::{App, Mode, Screen, TasksState};

pub(crate) fn handle(app: &mut App, key: KeyCode) -> Result<bool, AppError> {
    let Screen::Projects(ref mut state) = app.screen else {
        return Ok(false);
    };

    match &mut app.mode {
        Mode::Normal => match key {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Char('j') => {
                let Screen::Projects(ref mut state) = app.screen else { return Ok(false) };
                if state.input_focused {
                    // already at bottom
                } else if state.list_state.selected() == Some(app.workspace.projects.len().saturating_sub(1)) {
                    state.list_state.select(None);
                    state.input_focused = true;
                } else {
                    state.list_state.select_next();
                }
            }
            KeyCode::Char('k') => {
                let Screen::Projects(ref mut state) = app.screen else { return Ok(false) };
                if state.input_focused {
                    state.input_focused = false;
                    let last = app.workspace.projects.len().saturating_sub(1);
                    state.list_state.select(Some(last));
                } else {
                    state.list_state.select_previous();
                }
            }
            KeyCode::Char('l') | KeyCode::Enter => {
                let Screen::Projects(ref mut state) = app.screen else { return Ok(false) };
                if state.input_focused {
                    app.mode = Mode::Creating { input: String::new() };
                } else if let Some(idx) = state.list_state.selected() {
                    app.screen = Screen::Tasks(TasksState::new(idx));
                }
            }
            KeyCode::Char('d') => {
                let Screen::Projects(ref state) = app.screen else { return Ok(false) };
                if state.last_key == Some(KeyCode::Char('d')) {
                    if let Some(selected) = state.list_state.selected() {
                        let idx = app.workspace.get_project(selected as u32)?;
                        let name = app.workspace.projects[idx].name.clone();
                        app.mode = Mode::Confirming { target: name };
                    }
                }
            }
            _ => {}
        },
        Mode::Creating { input } => match key {
            KeyCode::Char(c) => input.push(c),
            KeyCode::Backspace => { input.pop(); }
            KeyCode::Enter => {
                let name = input.clone();
                app.workspace.add_project(name);
                app.mode = Mode::Normal;
                let Screen::Projects(ref mut state) = app.screen else { return Ok(false) };
                state.input_focused = false;
            }
            KeyCode::Esc => {
                app.mode = Mode::Normal;
                let Screen::Projects(ref mut state) = app.screen else { return Ok(false) };
                state.input_focused = false;
            }
            _ => {}
        },
        Mode::Confirming { .. } => {
            if key == KeyCode::Char('y') {
                let Screen::Projects(ref mut state) = app.screen else { return Ok(false) };
                if let Some(selected) = state.list_state.selected() {
                    app.workspace.delete_project(selected as u32)?;
                    let new_len = app.workspace.projects.len();
                    let Screen::Projects(ref mut state) = app.screen else { return Ok(false) };
                    if new_len == 0 {
                        state.list_state.select(None);
                    } else {
                        state.list_state.select(Some(selected.min(new_len - 1)));
                    }
                }
            }
            app.mode = Mode::Normal;
        }
    }

    let Screen::Projects(ref mut state) = app.screen else { return Ok(false) };
    state.last_key = Some(key);

    Ok(false)
}

pub(crate) fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    let [title_area, content_area, _, input_area, help_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Length(3),
    ])
    .areas(area);

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

    let Screen::Projects(ref mut state) = app.screen else { return };
    frame.render_stateful_widget(project_widget, content_area, &mut state.list_state);

    let input_text = match &app.mode {
        Mode::Creating { input } => format!("Project Name: {}_", input),
        _ if state.input_focused => "+ New project...".to_string(),
        _ => String::new(),
    };

    let input_border = if state.input_focused {
        Block::bordered().style(Style::new().fg(Color::Cyan))
    } else {
        Block::bordered()
    };

    frame.render_widget(
        Paragraph::new(input_text).block(input_border),
        input_area,
    );

    let help_text = match &app.mode {
        Mode::Confirming { target } => format!("⚠  Delete \"{}\"?   y confirm   n cancel", target),
        Mode::Creating { .. } => "↵ create   Esc cancel".to_string(),
        _ => "j/k navigate   ↵/l open   dd delete   q quit".to_string(),
    };

    frame.render_widget(
        Paragraph::new(help_text).block(Block::bordered().title("Help")),
        help_area,
    );
}
