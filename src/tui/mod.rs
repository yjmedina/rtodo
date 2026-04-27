mod app;
mod views;
mod widgets;

pub use app::{Action, App, Mode, View};
pub use widgets::TaskListWidget;

use crate::{AppError, workspace::Workspace};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{DefaultTerminal, Frame};

fn handle_key_event(app: &App, key: &KeyEvent) -> Option<Action> {
    match app.view {
        View::Workspace => views::workspace::handle_key_event(app, key),
        View::Project { .. } => views::project::handle_key_event(app, key),
        View::Task { .. } => views::task::handle_key_event(app, key),
    }
}

fn handle_action(app: &mut App, action: Action) -> Result<View, AppError> {
    match app.view {
        View::Workspace => views::workspace::handle_action(app, action),
        View::Project { .. } => views::project::handle_action(app, action),
        View::Task { .. } => views::task::handle_action(app, action),
    }
}

fn view(frame: &mut Frame, app: &mut App) {
    match app.view {
        View::Workspace => views::workspace::view(frame, app),
        View::Project { .. } => views::project::view(frame, app),
        View::Task { .. } => views::task::view(frame, app),
    }
}

fn run(terminal: &mut DefaultTerminal, app: &mut App) -> Result<(), AppError> {
    loop {
        terminal.draw(|frame| view(frame, app))?;

        if let Event::Key(key) = crossterm::event::read()?
            && key.kind == KeyEventKind::Press
        {
            if key.code == KeyCode::Char('q') && !matches!(app.mode, Mode::CreateProject) {
                break;
            }
            if let Some(action) = handle_key_event(app, &key) {
                app.view = handle_action(app, action)?;
            }
            app.last_key = Some(key.code);
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
