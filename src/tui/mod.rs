mod widgets;
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    widgets::{Block, List, ListItem, Paragraph},
};
pub use widgets::TaskListWidget;

use crate::{AppError, workspace::Workspace};
use crossterm::event::{Event, KeyCode, KeyEventKind};

struct App<'a> {
    workspace: &'a mut Workspace,
}

impl<'a> App<'a> {
    fn new(workspace: &'a mut Workspace) -> Self {
        App { workspace }
    }
}

fn render(frame: &mut Frame, app: &mut App) {
    let [title_area, content_area, help_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    let title = Paragraph::new("rtodo").block(Block::bordered().title("rtodo"));

    frame.render_widget(title, title_area);

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

    let project_widget = List::new(project_widgets).block(Block::bordered().title("Projects"));
    frame.render_widget(project_widget, content_area);

    let help = Paragraph::new("q: quit").block(Block::bordered().title("Help"));
    frame.render_widget(help, help_area);
}

fn run(terminal: &mut DefaultTerminal, app: &mut App) -> Result<(), AppError> {
    loop {
        terminal.draw(|frame| render(frame, app))?;

        if let Event::Key(key) = crossterm::event::read()?
            && key.kind == KeyEventKind::Press
            && key.code == KeyCode::Char('q')
        {
            break;
        }
    }

    Ok(())
}

pub fn main(workspace: &mut Workspace) -> Result<(), AppError> {
    let mut app = App::new(workspace);
    let mut terminal = ratatui::init();
    let r = run(&mut terminal, &mut app);
    ratatui::restore();
    r
}
