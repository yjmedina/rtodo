use crate::tui::app::ScreenMode;
use crate::tui::workspace::{WorkspaceFocus, WorkspaceScreen};
use crate::workspace::Workspace;
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Style},
    widgets::{Block, List, ListItem, Paragraph},
};

pub fn render_frame(frame: &mut Frame, screen: &mut WorkspaceScreen, workspace: &Workspace) {
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

    let items: Vec<ListItem> = workspace
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

    let list = List::new(items)
        .block(Block::bordered().title("Projects"))
        .highlight_style(Style::new().bg(Color::Cyan).fg(Color::Black))
        .highlight_symbol("▶  ");
    frame.render_stateful_widget(list, content_area, &mut screen.list);

    let input_text = match (&screen.focus, &screen.mode) {
        (WorkspaceFocus::NewProject, ScreenMode::Insert(draft)) => {
            format!("Project Name: {}_", draft)
        }
        (WorkspaceFocus::NewProject, ScreenMode::Navigate) => "+ New project...".to_string(),
        (WorkspaceFocus::Content, _) => String::new(),
    };
    let input_border = match screen.focus {
        WorkspaceFocus::NewProject => Block::bordered().style(Style::new().fg(Color::Cyan)),
        WorkspaceFocus::Content => Block::bordered(),
    };
    frame.render_widget(Paragraph::new(input_text).block(input_border), input_area);

    let help_text = match (&screen.focus, &screen.mode) {
        (WorkspaceFocus::Content, ScreenMode::Navigate) => {
            "j/k navigate   ↵/l open   dd delete   q quit"
        }
        (WorkspaceFocus::NewProject, ScreenMode::Navigate) => "↵/l create   k back   q quit",
        (WorkspaceFocus::NewProject, ScreenMode::Insert(_)) => "↵ create   Esc cancel",
        (WorkspaceFocus::Content, ScreenMode::Insert(_)) => "",
    };
    frame.render_widget(
        Paragraph::new(help_text).block(Block::bordered().title("Help")),
        help_area,
    );
}
