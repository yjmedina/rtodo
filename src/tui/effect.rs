use super::overlay::Overlay;

pub enum Effect {
    // Screen
    PushScreen,
    PopScreen,

    // Navigation
    SelectNext,
    SelectPrev,

    // Insert
    EnterInsert,
    CancelInsert,
    DraftPush(char),
    DraftPop,

    // Overlay
    OpenOverlay(Overlay),
    CloseOverlay,

    // Project
    CreateProject { name: String },
    DeleteProject { pid: u32 },

    // Task
    CreateTask,
    DeleteTask,
    ToggleTaskStatus,
}
