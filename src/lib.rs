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
pub mod models;
pub mod style;
pub mod ui;
pub mod workspace;
