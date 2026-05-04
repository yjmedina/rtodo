//! Sidebar — left pane showing all workspace projects with progress.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
};

use super::app::{App, ProjectFocus, ScreenMode};
use super::draft::InsertTarget;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let focused = app.screen.focus == ProjectFocus::Sidebar;
    let border_color = if focused { Color::Cyan } else { Color::Gray };

    let mut items: Vec<ListItem> = app
        .workspace
        .projects
        .iter()
        .map(|p| {
            let done = p.complete_tasks();
            let total = p.task_count();
            let line = Line::from(vec![
                Span::raw(format!(" {}", p.name)),
                Span::styled(
                    format!("  {done}/{total}"),
                    Style::default().fg(Color::Gray),
                ),
            ]);
            ListItem::new(line)
        })
        .collect();

    // Inline draft row when creating a new project.
    let mut draft_idx: Option<usize> = None;
    if let ScreenMode::Insert(d) = &app.screen.mode
        && matches!(d.target, InsertTarget::Project)
    {
        let row = ListItem::new(Line::from(vec![
            Span::raw(" "),
            Span::styled(format!("{}▌", d.text), Style::default().fg(Color::Yellow)),
        ]));
        draft_idx = Some(items.len());
        items.push(row);
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" WORKSPACE ")
        .border_style(Style::default().fg(border_color));

    let mut state = ListState::default();
    let selected = draft_idx.or_else(|| {
        if app.workspace.projects.is_empty() {
            None
        } else {
            Some(
                app.screen
                    .sidebar_cursor
                    .min(app.workspace.projects.len() - 1),
            )
        }
    });
    state.select(selected);

    frame.render_stateful_widget(
        List::new(items)
            .block(block)
            .highlight_style(Style::default().bg(Color::DarkGray)),
        area,
        &mut state,
    );
}
