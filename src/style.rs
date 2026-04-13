//! Terminal styling helpers for `rtodo`.
//!
//! All functions produce strings that either contain ANSI escape codes (when
//! writing to a real TTY) or plain text (when piped / redirected).
//!
//! ## How TTY detection works
//!
//! [`owo_colors`] has two kinds of color APIs:
//!
//! - **Direct**: `.red()`, `.bold()`, … — always emit ANSI codes, no TTY check.
//! - **Guarded**: `.if_supports_color(stream, |v| v.style(...))` — checks whether
//!   `stream` is a TTY (via the `supports-colors` feature) and emits codes only
//!   when it is.
//!
//! We use the guarded form everywhere via [`owo_colors::Style`], which combines
//! multiple attributes into one owned value — avoiding the temporary-borrow
//! lifetime error you'd get from chaining `.green().bold()` inside a closure.
//!
//! ## Color semantics
//!
//! | Color  | Meaning                                           |
//! |--------|---------------------------------------------------|
//! | Green  | Success / creation / completion                   |
//! | Cyan   | Active / focus state                              |
//! | Red    | Destructive operations, high priority             |
//! | Yellow | In-progress / warning                             |
//! | Blue   | New / informational                               |
//! | Dim    | Low-signal metadata (IDs, dates, separators)      |

use chrono::{DateTime, Utc};
use owo_colors::{OwoColorize, Stream, Style};

use crate::models::{Priority, Status, Task};

const DATE_FORMAT: &str = "%Y-%m-%d";
const STDOUT: Stream = Stream::Stdout;
const STDERR: Stream = Stream::Stderr;

// ── Action prefix helpers ────────────────────────────────────────────────────

/// Bold green prefix, right-padded to 8 chars. Use for creation/success operations.
///
/// Padding is applied *before* color so ANSI bytes don't skew the field width.
pub fn action_green(label: &str) -> String {
    let padded = format!("{:<8}", label);
    format!(
        "{}",
        padded.if_supports_color(STDOUT, |v| v.style(Style::new().green().bold()))
    )
}

/// Bold cyan prefix, right-padded to 8 chars. Use for focus / active operations.
pub fn action_cyan(label: &str) -> String {
    let padded = format!("{:<8}", label);
    format!(
        "{}",
        padded.if_supports_color(STDOUT, |v| v.style(Style::new().cyan().bold()))
    )
}

/// Bold red prefix, right-padded to 8 chars. Use for destructive operations.
pub fn action_red(label: &str) -> String {
    let padded = format!("{:<8}", label);
    format!(
        "{}",
        padded.if_supports_color(STDOUT, |v| v.style(Style::new().red().bold()))
    )
}

/// Bold red prefix for stderr. Use for error messages.
pub fn error_prefix() -> String {
    format!(
        "{}",
        "error:".if_supports_color(STDERR, |v| v.style(Style::new().red().bold()))
    )
}

// ── Field formatters ─────────────────────────────────────────────────────────

/// Dimmed `[n]` ID bracket.
pub fn fmt_id(id: u32) -> String {
    format!(
        "{}",
        format!("[{}]", id).if_supports_color(STDOUT, |v| v.style(Style::new().dimmed()))
    )
}

/// Dimmed date string (`YYYY-MM-DD`).
pub fn fmt_date(dt: &DateTime<Utc>) -> String {
    let s = dt.format(DATE_FORMAT).to_string();
    format!(
        "{}",
        s.if_supports_color(STDOUT, |v| v.style(Style::new().dimmed()))
    )
}

/// Two-char priority symbol with semantic color.
///
/// - `!!` bold red  — high
/// -  ` ·` white    — medium
/// -  ` ·` dim      — low
pub fn fmt_priority_symbol(priority: &Priority) -> String {
    match priority {
        Priority::High => format!(
            "{}",
            "!!".if_supports_color(STDOUT, |v| v.style(Style::new().red().bold()))
        ),
        Priority::Medium => format!(
            " {}",
            "·".if_supports_color(STDOUT, |v| v.style(Style::new().white()))
        ),
        Priority::Low => format!(
            " {}",
            "·".if_supports_color(STDOUT, |v| v.style(Style::new().dimmed()))
        ),
    }
}

/// Priority as a full colored text label. Used in single-task action lines.
pub fn fmt_priority_label(priority: &Priority) -> String {
    match priority {
        Priority::High => format!(
            "{}",
            "high".if_supports_color(STDOUT, |v| v.style(Style::new().red().bold()))
        ),
        Priority::Medium => "medium".to_string(),
        Priority::Low => format!(
            "{}",
            "low".if_supports_color(STDOUT, |v| v.style(Style::new().dimmed()))
        ),
    }
}

// ── List row formatters ──────────────────────────────────────────────────────

/// Bold, colored section header with a dim separator line.
///
/// ```text
/// ● In Progress ──────────────────────────────────────────────
/// ● New ──────────────────────────────────────────────────────
/// ✓ Completed ────────────────────────────────────────────────
/// ```
pub fn fmt_status_header(status: &Status) -> String {
    let (symbol, label) = match status {
        Status::InProgress => (
            "●",
            format!(
                "{}",
                "In Progress".if_supports_color(STDOUT, |v| v.style(Style::new().yellow().bold()))
            ),
        ),
        Status::New => (
            "●",
            format!(
                "{}",
                "New".if_supports_color(STDOUT, |v| v.style(Style::new().blue().bold()))
            ),
        ),
        Status::Completed => (
            "✓",
            format!(
                "{}",
                "Completed".if_supports_color(STDOUT, |v| v.style(Style::new().green().bold()))
            ),
        ),
    };
    // Fixed-length separator — avoids needing to know terminal width.
    let sep = format!(
        "{}",
        "─"
            .repeat(44)
            .if_supports_color(STDOUT, |v| v.style(Style::new().dimmed()))
    );
    format!("{symbol} {label} {sep}")
}

/// Format a top-level task row for the task list.
///
/// Columns: indent · dim ID · active/priority marker · description
/// (padded to `desc_width`) · priority symbol · dim date
///
/// ```text
///   [1] * Fix bug in module           !!  2026-04-13
///   [0]   Write documentation          ·  2026-04-13
/// ```
pub fn fmt_task_line(task: &Task, is_active: bool, desc_width: usize) -> String {
    let id = fmt_id(task.id);
    let marker = if is_active {
        format!(
            "{}",
            "*".if_supports_color(STDOUT, |v| v.style(Style::new().cyan().bold()))
        )
    } else if task.priority == Priority::High {
        format!(
            "{}",
            "!".if_supports_color(STDOUT, |v| v.style(Style::new().red().bold()))
        )
    } else {
        " ".to_string()
    };
    // Pad plain string before any color — format width is measured correctly.
    let desc = format!("{:<width$}", task.description, width = desc_width);
    let prio = fmt_priority_symbol(&task.priority);
    let date = fmt_date(&task.created_at);
    format!("  {} {} {}  {}  {}", id, marker, desc, prio, date)
}

/// Format a subtask row for the task list, indented with a dim `↳` arrow.
///
/// ```text
///       ↳ [3]   Subtask detail          ·  2026-04-13
/// ```
pub fn fmt_sub_line(task: &Task, desc_width: usize) -> String {
    let id = fmt_id(task.id);
    let desc = format!("{:<width$}", task.description, width = desc_width);
    let prio = fmt_priority_symbol(&task.priority);
    let date = fmt_date(&task.created_at);
    let arrow = format!(
        "{}",
        "↳".if_supports_color(STDOUT, |v| v.style(Style::new().dimmed()))
    );
    format!("      {} {} {}  {}  {}", arrow, id, desc, prio, date)
}

/// Format a task as a compact single-line string for action output
/// (used after add / start / complete / edit / delete).
///
/// ```text
/// [0] Write documentation   medium   2026-04-13
/// ```
pub fn fmt_task_action(task: &Task) -> String {
    let id = fmt_id(task.id);
    let prio = fmt_priority_label(&task.priority);
    let date = fmt_date(&task.created_at);
    format!("{} {}   {}   {}", id, task.description, prio, date)
}
