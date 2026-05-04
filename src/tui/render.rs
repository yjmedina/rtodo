//! Pure painter.
//!
//! Reads `App`, writes nothing. Produces display by building `Line`s of
//! styled `Span`s — the ratatui idiom for mixed inline colours.

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};

use crate::models::{Priority, Status};

use super::app::{App, ProjectFocus, ScreenMode, TreeRowId};
use super::overlay::Overlay;
use super::sidebar;
use super::tree::{self, RowKind, TreeRow};

const SIDEBAR_MIN_WIDTH: u16 = 100;
const SIDEBAR_WIDTH: u16 = 22;
const HELP_HEIGHT: u16 = 1;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let [main, help_area] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(HELP_HEIGHT)]).areas(area);

    if area.width >= SIDEBAR_MIN_WIDTH {
        let [sidebar_area, content_area] =
            Layout::horizontal([Constraint::Length(SIDEBAR_WIDTH), Constraint::Fill(1)])
                .areas(main);
        sidebar::render(frame, sidebar_area, app);
        render_tree_pane(frame, content_area, app);
    } else {
        render_tree_pane(frame, main, app);
    }

    render_help(frame, help_area, app);

    if let Some(overlay) = &app.overlay {
        render_overlay(frame, overlay, area);
    }
}

// ── Tree pane ─────────────────────────────────────────────────────────────────

fn render_tree_pane(frame: &mut Frame, area: Rect, app: &App) {
    let p_idx = match app.screen.p_idx {
        Some(i) => i,
        None => {
            let block = Block::default().borders(Borders::ALL).border_style(
                Style::default().fg(pane_border_color(app.screen.focus == ProjectFocus::Tree)),
            );
            frame.render_widget(
                Paragraph::new("No projects — press `i` to create one").block(block),
                area,
            );
            return;
        }
    };

    let project = &app.workspace.projects[p_idx];
    let rows = tree::flatten(project, &app.screen.tree.expanded);

    // Draft row splice — insert into the visual row list.
    // Skip when drafting on the sidebar (target == Project): that draft
    // belongs to the sidebar pane only.
    let (list_rows, draft_row_idx): (Vec<TreeRow>, Option<usize>) = match &app.screen.mode {
        ScreenMode::Insert(draft)
            if !matches!(draft.target, super::draft::InsertTarget::Project) =>
        {
            let (rows, idx) = splice_draft(rows, draft);
            (rows, Some(idx))
        }
        _ => (rows, None),
    };

    let cursor_idx = app
        .screen
        .tree
        .cursor
        .and_then(|c| tree::cursor_index(&list_rows, c))
        .or(draft_row_idx);

    let items: Vec<ListItem> = list_rows
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let is_cursor = cursor_idx == Some(i);
            format_row(row, project, is_cursor)
        })
        .collect();

    let title = format!(" {} ", project.name);
    let border_color = pane_border_color(app.screen.focus == ProjectFocus::Tree);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(border_color));

    let mut list_state = ListState::default();
    list_state.select(cursor_idx);

    frame.render_stateful_widget(
        List::new(items)
            .block(block)
            .highlight_style(Style::default().bg(Color::DarkGray)),
        area,
        &mut list_state,
    );
}

fn splice_draft(rows: Vec<TreeRow>, draft: &super::draft::Draft) -> (Vec<TreeRow>, usize) {
    use super::draft::InsertTarget;
    use super::tree::RowKind;

    let draft_row = TreeRow {
        id: TreeRowId::Task(u32::MAX), // sentinel — never matched
        depth: match draft.target {
            InsertTarget::Subtask { .. } => 1,
            _ => 0,
        },
        last_in_group: true,
        kind: RowKind::Drafting {
            text: draft.text.clone(),
        },
    };

    let insert_after = match draft.target {
        InsertTarget::Project => None,
        InsertTarget::TaskRoot => None,
        InsertTarget::TaskSibling { after } => {
            rows.iter().position(|r| r.id == TreeRowId::Task(after))
        }
        InsertTarget::Subtask { task } => {
            // Insert after last visible subtask of this parent, or after parent.
            rows.iter()
                .rposition(|r| matches!(r.id, TreeRowId::Subtask { task: t, .. } if t == task))
                .or_else(|| rows.iter().position(|r| r.id == TreeRowId::Task(task)))
        }
    };

    let idx = match insert_after {
        Some(i) => i + 1,
        None => rows.len(),
    };

    let mut out = rows;
    out.insert(idx, draft_row);
    (out, idx)
}

fn format_row<'a>(
    row: &TreeRow,
    project: &'a crate::models::Project,
    _is_cursor: bool,
) -> ListItem<'a> {
    let indent = "  ".repeat(row.depth as usize);
    let glyph = branch_glyph(row);

    match &row.kind {
        RowKind::Drafting { text } => ListItem::new(Line::from(vec![
            Span::raw(format!("{indent}{glyph}  ")),
            Span::styled(format!("{text}▌"), Style::default().fg(Color::Yellow)),
        ])),

        RowKind::Subtask => {
            let id = match row.id {
                TreeRowId::Subtask {
                    task: tid,
                    sub: sid,
                } => project
                    .tasks
                    .iter()
                    .find(|t| t.id == tid)
                    .and_then(|t| t.subtasks.iter().find(|s| s.id == sid)),
                _ => None,
            };
            let (checkbox, desc, prio_span) = match id {
                Some(s) => {
                    let check = if s.completed { "[x]" } else { "[ ]" };
                    let prio = priority_span(&s.priority);
                    (check, s.description.as_str(), prio)
                }
                None => ("[ ]", "?", Span::raw("")),
            };
            let line = Line::from(vec![
                Span::raw(format!("{indent}{glyph} {checkbox} ")),
                prio_span,
                Span::raw(format!(" {desc}")),
            ]);
            ListItem::new(line)
        }

        RowKind::Leaf | RowKind::Parent { .. } => {
            let task = match row.id {
                TreeRowId::Task(tid) => project.tasks.iter().find(|t| t.id == tid),
                _ => None,
            };
            match task {
                None => ListItem::new(Line::from("?")),
                Some(t) => {
                    let expand_glyph = match &row.kind {
                        RowKind::Parent { expanded: true } => "▼ ",
                        RowKind::Parent { expanded: false } => "▶ ",
                        _ => "  ",
                    };
                    let status_span = status_span(&t.status());
                    let prio_span = priority_span(&t.priority);

                    let mut spans = vec![
                        Span::raw(format!("{indent}{glyph}{expand_glyph}")),
                        status_span,
                        Span::raw(" "),
                        prio_span,
                        Span::raw(format!(" {}", t.description)),
                    ];

                    if !t.subtasks.is_empty() {
                        let done = t.completed_subtask_count();
                        let total = t.subtasks.len();
                        let bar = progress_bar(done, total, 8);
                        spans.push(Span::raw("  "));
                        spans.push(Span::styled(bar, Style::default().fg(Color::Green)));
                        spans.push(Span::raw(format!(" {done}/{total}")));
                    }

                    ListItem::new(Line::from(spans))
                }
            }
        }
    }
}

fn branch_glyph(row: &TreeRow) -> &'static str {
    if row.depth == 0 {
        return "";
    }
    if row.last_in_group {
        "└─"
    } else {
        "├─"
    }
}

fn status_span(status: &Status) -> Span<'static> {
    match status {
        Status::New => Span::styled("New", Style::default().fg(Color::Gray)),
        Status::InProgress => Span::styled("WIP", Style::default().fg(Color::Yellow)),
        Status::Completed => Span::styled("Done", Style::default().fg(Color::Green)),
    }
}

fn priority_span(priority: &Priority) -> Span<'static> {
    match priority {
        Priority::High => Span::styled("!", Style::default().fg(Color::Red)),
        Priority::Medium => Span::styled("·", Style::default().fg(Color::Yellow)),
        Priority::Low => Span::styled(" ", Style::default()),
    }
}

fn progress_bar(done: usize, total: usize, width: usize) -> String {
    if total == 0 {
        return "░".repeat(width);
    }
    let filled = (done * width) / total;
    let empty = width - filled;
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

// ── Overlay ───────────────────────────────────────────────────────────────────

fn render_overlay(frame: &mut Frame, overlay: &Overlay, area: Rect) {
    let Overlay::Confirm(c) = overlay;

    let width = (c.prompt.len() as u16 + 4).min(area.width.saturating_sub(4));
    let height = 3u16;
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    let popup = Rect {
        x,
        y,
        width,
        height,
    };

    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(c.prompt.as_str())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .style(Style::default()),
        popup,
    );
}

// ── Help ──────────────────────────────────────────────────────────────────────

fn render_help(frame: &mut Frame, area: Rect, app: &App) {
    let style = Style::default().fg(Color::Gray);
    let text = if let Some(err) = &app.error {
        Span::styled(format!(" ! {err}"), Style::default().fg(Color::Red))
    } else if let ScreenMode::Insert(_) = &app.screen.mode {
        Span::styled(" Enter commit  ·  Esc cancel", style)
    } else {
        let hint = match app.screen.focus {
            ProjectFocus::Sidebar => " j/k select  ·  Enter open  ·  i new project  ·  Tab tree",
            ProjectFocus::Tree => {
                " j/k move  ·  l/h expand  ·  space toggle  ·  i new  ·  I child  ·  dd delete  ·  Tab sidebar"
            }
        };
        Span::styled(hint, style)
    };
    frame.render_widget(Paragraph::new(Line::from(text)), area);
}

fn pane_border_color(focused: bool) -> Color {
    if focused { Color::Cyan } else { Color::Gray }
}
