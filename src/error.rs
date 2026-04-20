use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("workspace already initialized in this directory")]
    WorkspaceAlreadyInit,
    #[error("no workspace found — run `rtodo init` first")]
    WorkspaceNotFound,
    #[error("project {id} not found")]
    ProjectNotFound { id: u32 },
    #[error("no active project — run `rtodo project switch <id>` first")]
    NoActiveProject,
    #[error("task {id} not found")]
    TaskNotFound { id: u32 },
    #[error("cannot add subtask to a subtask (max depth: 2)")]
    SubtaskDepthExceeded,
    #[error("task {id} has incomplete subtasks — complete them first")]
    TaskHasIncompleteSubtasks { id: u32 },
    #[error("no active task")]
    NoActiveTask,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
