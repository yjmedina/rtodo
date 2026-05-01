pub mod apply;
pub mod handle;
pub mod render;
use crate::tui::app::ScreenMode;
use ratatui::widgets::ListState;

pub struct ProjectScreen {
    pub p_idx: usize,
    pub list: ListState,
    pub mode: ScreenMode,
    pub focus: ProjectFocus,
}

pub enum ProjectFocus {
    StatusFilter(TaskFilter),
    TaskList,
    NewTask,
    NewSubtask,
}

pub enum TaskFilter {
    Pending,
    InProgress,
    Done,
    All,
}

impl ProjectScreen {
    pub fn new(p_idx: usize, task_len: usize) -> Self {
        let mut list = ListState::default();
        let index = (task_len > 0).then_some(0);
        list.select(index);
        let focus = if task_len > 0 {
            ProjectFocus::TaskList
        } else {
            ProjectFocus::NewTask
        };

        Self {
            p_idx,
            list,
            mode: ScreenMode::Navigate,
            focus,
        }
    }
}
