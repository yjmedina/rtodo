//! `rtodo` — a terminal-based todo manager built in Rust.
//!
//! # Architecture
//!
//! ```text
//! cli  ──►  dispatch  ──►  models
//!                    ──►  workspace
//! ```
//!
//! - [`cli`]: Clap-derived CLI definitions (commands, subcommands, arguments).
//! - [`dispatch`]: Maps parsed CLI arguments to workspace and model operations.
//! - [`models`]: Core domain types — [`models::Project`], [`models::Task`],
//!   [`models::Status`], [`models::Priority`].
//! - [`workspace`]: Persistence layer — discovers, loads, and saves `state.json`.

pub mod cli;
pub mod dispatch;
pub mod error;
pub use error::AppError;
pub mod models;
pub mod style;
pub mod ui;
pub mod workspace;
// `tui` and `view` modules are temporarily disabled during the Task/Subtask
// refactor. They will be restored in a follow-up challenge.
// pub mod tui;
// pub mod view;
