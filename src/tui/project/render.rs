use crate::models::{Priority, Status, Task};
use crate::tui::app::ScreenMode;
use crate::tui::project::{ProjectFocus, ProjectScreen};
use crate::workspace::Workspace;
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Style},
    widgets::{Block, List, ListItem, Paragraph},
};

pub fn render_frame(frame: &mut Frame, screen: &mut ProjectScreen, workspace: &Workspace) {
    let project = &workspace.projects[screen.p_idx];

    let [title_area, content_area, _, input_area, help_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Length(3),
    ])
    .areas(frame.area());

    let title = format!(
        "Project: {}   {}/{}",
        project.name,
        project.complete_tasks(),
        project.task_count()
    );
    frame.render_widget(
        Paragraph::new(title).block(Block::bordered().title("rtodo")),
        title_area,
    );

    let items: Vec<ListItem> = project.tasks.iter().map(format_task).collect();
    let list = List::new(items)
        .block(Block::bordered().title("Tasks"))
        .highlight_style(Style::new().bg(Color::Cyan).fg(Color::Black))
        .highlight_symbol("▶  ");
    frame.render_stateful_widget(list, content_area, &mut screen.list);

    let input_text = match (&screen.focus, &screen.mode) {
        (ProjectFocus::NewTask | ProjectFocus::NewSubtask, ScreenMode::Insert(draft)) => {
            format!("Task description: {}_", draft)
        }
        (ProjectFocus::NewTask, ScreenMode::Navigate) => "+ New task...".to_string(),
        (ProjectFocus::NewSubtask, ScreenMode::Navigate) => "+ New subtask...".to_string(),
        _ => String::new(),
    };
    let input_border = match screen.focus {
        ProjectFocus::NewTask | ProjectFocus::NewSubtask => {
            Block::bordered().style(Style::new().fg(Color::Cyan))
        }
        _ => Block::bordered(),
    };
    frame.render_widget(Paragraph::new(input_text).block(input_border), input_area);

    let help_text = match (&screen.focus, &screen.mode) {
        (ProjectFocus::TaskList, ScreenMode::Navigate) => {
            "j/k navigate   ↵/l open   i new   dd delete   q back"
        }
        (ProjectFocus::NewTask, ScreenMode::Navigate) => "↵/l/i create   k back   q back",
        (ProjectFocus::NewTask | ProjectFocus::NewSubtask, ScreenMode::Insert(_)) => {
            "↵ create   Esc cancel"
        }
        _ => "",
    };
    frame.render_widget(
        Paragraph::new(help_text).block(Block::bordered().title("Help")),
        help_area,
    );
}

fn format_task(task: &Task) -> ListItem<'_> {
    let status = match task.status {
        Status::New => "[ ]",
        Status::InProgress => "[~]",
        Status::Completed => "[x]",
    };
    let prio = match task.priority {
        Priority::Low => 'L',
        Priority::Medium => 'M',
        Priority::High => 'H',
    };
    let indent = if task.parent_id.is_some() { "  " } else { "" };
    ListItem::new(format!(
        "{}{} [{}] {}",
        indent, status, prio, task.description
    ))
}
