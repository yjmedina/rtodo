use std::fmt;

#[derive(Debug)]
pub enum AppError {
    WorkspaceAlreadyInit,
    WorkspaceNotFound,
    ProjectNotFound { id: u32 },
    NoActiveProject,
    TaskNotFound { id: u32 },
    SubtaskDepthExceeded,
    TaskHasIncompleteSubtasks { id: u32 },
    NoActiveTask,
    Io(std::io::Error),
    Json(serde_json::Error),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::WorkspaceAlreadyInit => {
                write!(f, "workspace already initialized in this directory")
            }
            Self::WorkspaceNotFound => write!(f, "no workspace found — run `rtodo init` first"),
            Self::ProjectNotFound { id } => write!(f, "project {id} not found"),
            Self::NoActiveProject => write!(
                f,
                "no active project — run `rtodo project switch <id>` first"
            ),
            Self::TaskNotFound { id } => write!(f, "task {id} not found"),
            Self::SubtaskDepthExceeded => {
                write!(f, "cannot add subtask to a subtask (max depth: 2)")
            }
            Self::TaskHasIncompleteSubtasks { id } => {
                write!(f, "task {id} has incomplete subtasks — complete them first")
            }
            Self::NoActiveTask => write!(f, "no active task"),
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::Json(e) => write!(f, "JSON error: {e}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Json(e)
    }
}
