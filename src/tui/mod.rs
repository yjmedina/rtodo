//! Terminal UI entry point.
//!
//! The TUI is split into orthogonal layers:
//!
//! - [`app`]   — runtime state (`App`, `ProjectScreen`, `TreeState`, ...).
//! - [`intent`] — what a key *means* (pure `KeyEvent → Option<Intent>`).
//! - [`resolve`] — what an intent *does* (pure `Intent → Vec<Effect>`).
//! - [`apply`]  — the **only** place mutations happen (`App::apply`).
//! - [`render`] — pure paint; reads `App`, writes nothing.
//!
//! See `tui-design.md` at the repo root for the rationale behind each split.

mod app;
mod apply;
mod draft;
mod effect;
mod intent;
mod overlay;
mod render;
mod resolve;
mod sidebar;
mod tree;

use crate::AppError;
use crate::workspace::Workspace;
use crossterm::event::{Event, KeyEventKind};
use ratatui::DefaultTerminal;

use app::App;
use intent::{AppIntent, Intent};

fn run(terminal: &mut DefaultTerminal, app: &mut App) -> Result<(), AppError> {
    while !app.should_quit {
        terminal.draw(|frame| render::render(frame, app))?;

        let Event::Key(key) = crossterm::event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        let next_last_key = if let Some(intent) = intent::keymap(app, &key) {
            // After a chord fires, or any confirm intent (overlay dismiss),
            // forget last_key so the next press isn't mistaken for the
            // second key of a chord.
            let reset = matches!(
                intent,
                Intent::App(AppIntent::RequestDelete) | Intent::Confirm(_)
            );
            for effect in resolve::resolve(app, &intent) {
                app.apply(effect)?;
            }
            if reset { None } else { Some(key.code) }
        } else {
            Some(key.code)
        };
        app.last_key = next_last_key;
    }
    Ok(())
}

/// Boot the TUI against a loaded workspace.
pub fn main(ws: &mut Workspace) -> Result<(), AppError> {
    let mut app = App::new(ws);
    let mut terminal = ratatui::init();
    let r = run(&mut terminal, &mut app);
    ratatui::restore();
    r
}
