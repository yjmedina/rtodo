use crossterm::event::KeyCode;
use ratatui::widgets::ListState;

use crate::workspace::Workspace;

pub enum Mode {
    Navigate,
    FocusingInput,
    CreateProject,
    Confirmation,
}

pub enum View {
    Workspace,
    Project { pid: u32 },
    Task { tid: u32 },
}

pub struct App<'a> {
    pub view: View,
    pub workspace: &'a mut Workspace,
    pub project_list_state: ListState,

    pub draft: Option<String>,
    pub last_key: Option<KeyCode>,
    pub mode: Mode,
}

pub enum Action {
    NextProject,
    PreviousProject,
    DeleteProject { id: u32 },
    CreateProject,
    UpdateProjectDraft { c: char },
    PopProjectDraft,
    OpenProject { id: u32 },
    SetMode(Mode),
}

impl<'a> App<'a> {
    pub fn new(workspace: &'a mut Workspace) -> Self {
        let mut project_list_state = ListState::default();
        if !workspace.projects.is_empty() {
            project_list_state.select(Some(0));
        }
        App {
            view: View::Workspace,
            workspace,
            project_list_state,
            draft: None,
            last_key: None,
            mode: Mode::Navigate,
        }
    }
}
