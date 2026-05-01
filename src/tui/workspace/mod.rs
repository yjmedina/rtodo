pub mod apply;
pub mod handle;
pub mod render;
use crate::tui::app::ScreenMode;
use ratatui::widgets::ListState;

pub struct WorkspaceScreen {
    pub list: ListState,
    pub mode: ScreenMode,
    pub focus: WorkspaceFocus,
}

pub enum WorkspaceFocus {
    Content,
    NewProject,
}

impl WorkspaceScreen {
    pub fn new(project_len: usize) -> Self {
        let mut list = ListState::default();
        let index = (project_len > 0).then_some(0);
        list.select(index);
        let focus = if project_len > 0 {
            WorkspaceFocus::Content
        } else {
            WorkspaceFocus::NewProject
        };

        Self {
            list,
            mode: ScreenMode::Navigate,
            focus,
        }
    }
}
