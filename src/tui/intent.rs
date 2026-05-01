pub enum NavIntent {
    Up,
    Down,
    Enter,
    Back,
}

pub enum EditIntent {
    Start,
    Cancel,
    Submit,
    NewChar(char),
    DeleteChar,
}

pub enum ConfirmIntent {
    Yes,
    No,
}

pub enum AppIntent {
    Quit,
    RequestDelete,
    ToggleStatus,
}

pub enum Intent {
    Nav(NavIntent),
    Edit(EditIntent),
    Confirm(ConfirmIntent),
    App(AppIntent),
}
