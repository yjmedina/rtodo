use crossterm::event::KeyCode;
use ratatui::widgets::ListState;

use crate::workspace::Workspace;

pub struct ProjectsState {
    pub list_state: ListState,
    pub input_focused: bool,
    pub last_key: Option<KeyCode>,
}

impl ProjectsState {
    pub fn new(project_count: usize) -> Self {
        let mut list_state = ListState::default();
        if project_count > 0 {
            list_state.select(Some(0));
        }
        Self {
            list_state,
            input_focused: false,
            last_key: None,
        }
    }
}

pub struct TasksState {
    pub project_idx: usize,
    pub list_state: ListState,
}

impl TasksState {
    pub fn new(project_idx: usize) -> Self {
        Self {
            project_idx,
            list_state: ListState::default(),
        }
    }
}

pub enum Screen {
    Projects(ProjectsState),
    Tasks(TasksState),
}

pub enum Mode {
    Normal,
    Creating { input: String },
    Confirming { target: String },
}

pub struct App<'a> {
    pub(crate) workspace: &'a mut Workspace,
    pub(crate) screen: Screen,
    pub(crate) mode: Mode,
}

impl<'a> App<'a> {
    pub fn new(workspace: &'a mut Workspace) -> Self {
        let project_count = workspace.projects.len();
        App {
            workspace,
            screen: Screen::Projects(ProjectsState::new(project_count)),
            mode: Mode::Normal,
        }
    }

    pub fn project_count(&self) -> usize {
        self.workspace.projects.len()
    }
}
