use crate::tui::effect::Effect;
use crate::tui::intent::{ConfirmIntent, Intent};

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Clear, Paragraph},
};

use crossterm::event::{KeyCode, KeyEvent};

pub enum Overlay {
    Confirm(ConfirmOverlay),
}

pub struct ConfirmOverlay {
    pub prompt: String,
    pub effect: Box<Effect>,
}

impl ConfirmOverlay {
    pub fn new(prompt: String, effect: Effect) -> Self {
        Self {
            prompt,
            effect: Box::new(effect),
        }
    }

    pub fn prompt(&self) -> &str {
        &self.prompt
    }
}

pub fn get_intent(key: &KeyEvent) -> Intent {
    match key.code {
        KeyCode::Char('y') | KeyCode::Enter => Intent::Confirm(ConfirmIntent::Yes),
        _ => Intent::Confirm(ConfirmIntent::No),
    }
}

pub fn get_effect(intent: &Intent, overlay: Overlay) -> Vec<Effect> {
    match intent {
        Intent::Confirm(ConfirmIntent::Yes) => {
            let Overlay::Confirm(oc) = overlay;
            vec![*oc.effect, Effect::CloseOverlay]
        }
        _ => vec![Effect::CloseOverlay],
    }
}

pub fn render(frame: &mut Frame, overlay: &Overlay) {
    match overlay {
        Overlay::Confirm(c) => render_confirm(frame, c),
    }
}

fn render_confirm(frame: &mut Frame, c: &ConfirmOverlay) {
    let area = centered_rect(50, 20, frame.area());
    frame.render_widget(Clear, area);
    let text = format!("{}\n\ny/↵ confirm   n cancel", c.prompt());
    let block = Block::bordered()
        .title("Confirm")
        .style(Style::new().fg(Color::Yellow));
    let para = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);
    frame.render_widget(para, area);
}

fn centered_rect(pct_x: u16, pct_y: u16, area: Rect) -> Rect {
    let v = Layout::vertical([
        Constraint::Percentage((100 - pct_y) / 2),
        Constraint::Percentage(pct_y),
        Constraint::Percentage((100 - pct_y) / 2),
    ])
    .split(area);
    let h = Layout::horizontal([
        Constraint::Percentage((100 - pct_x) / 2),
        Constraint::Percentage(pct_x),
        Constraint::Percentage((100 - pct_x) / 2),
    ])
    .split(v[1]);
    h[1]
}
