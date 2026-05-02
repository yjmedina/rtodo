//! Modal layer painted on top of the screen.
//!
//! Today there is only `Confirm`; the enum exists so adding `Help`,
//! `Picker`, etc. later is a one-arm change.

use super::effect::Effect;

#[derive(Debug)]
pub enum Overlay {
    Confirm(ConfirmOverlay),
}

#[derive(Debug)]
pub struct ConfirmOverlay {
    pub prompt: String,
    /// Boxed because of the type cycle: `Effect::OpenOverlay` carries an
    /// `Overlay`, which carries a `ConfirmOverlay`, which holds an `Effect`.
    /// Indirection breaks the otherwise-infinite size.
    pub on_confirm: Box<Effect>,
}
