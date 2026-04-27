use crate::{
    AppError,
    tui::app::{Action, App, View},
};
use crossterm::event::KeyEvent;
use ratatui::Frame;

pub fn handle_key_event(_app: &App, _key: &KeyEvent) -> Option<Action> {
    todo!()
}

pub fn handle_action(_app: &mut App, _action: Action) -> Result<View, AppError> {
    todo!()
}

pub fn view(_frame: &mut Frame, _app: &mut App) {
    todo!()
}
