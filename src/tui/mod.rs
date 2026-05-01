mod app;
mod effect;
mod intent;
mod overlay;
mod project;
mod widgets;
mod workspace;

pub use app::{App, Screen};
use effect::Effect;
pub use intent::Intent;
pub use widgets::TaskListWidget;

use crate::AppError;
use crate::workspace::Workspace;
use crossterm::event::{Event, KeyEvent, KeyEventKind};
use ratatui::{DefaultTerminal, Frame};

fn handle_key_event(app: &App, key: &KeyEvent) -> Option<Intent> {
    if app.overlay.is_some() {
        return Some(overlay::get_intent(key));
    }

    match app.view() {
        Screen::Workspace(s) => workspace::handle::get_intent(s, key, app.last_key),
        Screen::Project(s) => project::handle::get_intent(s, key, app.last_key),
        Screen::Task => todo!(),
    }
}

fn get_effect(app: &mut App, intent: &Intent) -> Vec<Effect> {
    if let Some(overlay) = app.overlay.take() {
        return overlay::get_effect(intent, overlay);
    }

    match app.view() {
        Screen::Workspace(s) => workspace::handle::get_effect(s, intent, app.workspace),
        Screen::Project(s) => project::handle::get_effect(s, intent, app.workspace),
        Screen::Task => todo!(),
    }
}

fn render(frame: &mut Frame, app: &mut App) {
    let screen = app.screens.last_mut().expect("Screens can't be empty");
    match screen {
        Screen::Workspace(s) => workspace::render::render_frame(frame, s, &*app.workspace),
        Screen::Project(s) => project::render::render_frame(frame, s, app.workspace),
        Screen::Task => todo!(),
    }
    if let Some(o) = &app.overlay {
        overlay::render(frame, o);
    }
}

fn run(terminal: &mut DefaultTerminal, app: &mut App) -> Result<(), AppError> {
    while !app.should_quit {
        terminal.draw(|frame| render(frame, app))?;

        if let Event::Key(key) = crossterm::event::read()?
            && key.kind == KeyEventKind::Press
        {
            if let Some(intent) = handle_key_event(app, &key) {
                for effect in get_effect(app, &intent) {
                    app.apply(effect)?;
                }
            }
            app.last_key = Some(key.code);
        }
    }
    Ok(())
}

pub fn main(ws: &mut Workspace) -> Result<(), AppError> {
    let mut app = App::new(ws);
    let mut terminal = ratatui::init();
    let r = run(&mut terminal, &mut app);
    ratatui::restore();
    r
}
