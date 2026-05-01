use crate::AppError;
use crate::tui::effect::Effect;
use crate::tui::overlay::Overlay;
use crate::tui::workspace::{self, WorkspaceScreen};
use crate::workspace::Workspace;
use crossterm::event::KeyCode;
pub enum ScreenMode {
    Navigate,
    Insert(String),
}

pub enum Screen {
    Workspace(WorkspaceScreen),
    Project,
    Task,
}

pub struct App<'a> {
    pub workspace: &'a mut Workspace,
    pub screens: Vec<Screen>,
    pub last_key: Option<KeyCode>,
    pub should_quit: bool,
    pub overlay: Option<Overlay>,
}

impl<'a> App<'a> {
    pub fn new(workspace: &'a mut Workspace) -> Self {
        let workspace_screen = WorkspaceScreen::new(workspace.projects.len());
        let screens = vec![Screen::Workspace(workspace_screen)];
        App {
            screens,
            workspace,
            last_key: None,
            should_quit: false,
            overlay: None,
        }
    }

    pub fn view(&self) -> &Screen {
        self.screens.last().expect("Screens can't be empty")
    }

    pub fn view_mut(&mut self) -> &mut Screen {
        self.screens.last_mut().expect("Screens can't be empty")
    }

    pub fn apply(&mut self, effect: Effect) -> Result<(), AppError> {
        match self.view_mut() {
            Screen::Workspace(_) => workspace::apply::apply_effect(self, effect)?,
            _ => {}
        }
        Ok(())
    }
}
