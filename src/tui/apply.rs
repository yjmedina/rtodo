//! Single mutation surface.
//!
//! Every state change — runtime *and* domain — flows through `App::apply`.
//! That centrality is what lets `workspace.save()` live in one place at the
//! tail of each domain arm.

use crate::AppError;
use crate::models::{Priority, Status};

use super::app::{App, ProjectFocus, ScreenMode, TreeRowId, TreeState};
use super::draft::Draft;
use super::effect::Effect;

impl App<'_> {
    pub fn apply(&mut self, effect: Effect) -> Result<(), AppError> {
        // Clear ephemeral error on every action.
        if !matches!(effect, Effect::ShowError(_)) {
            self.error = None;
        }

        match effect {
            // ── runtime ──────────────────────────────────────────────────
            Effect::Quit => self.should_quit = true,

            // ── focus ───────────────────────────────────────────────────
            Effect::NextFocus => {
                self.screen.focus = self.screen.focus.next();
            }

            // ── sidebar cursor (phase 5 uses these for selection) ────────
            Effect::SelectNextProject => {
                let len = self.workspace.projects.len();
                if len > 0 {
                    self.screen.sidebar_cursor = (self.screen.sidebar_cursor + 1).min(len - 1);
                }
            }
            Effect::SelectPrevProject => {
                self.screen.sidebar_cursor = self.screen.sidebar_cursor.saturating_sub(1);
            }
            Effect::SelectProject { p_idx } => {
                self.screen.p_idx = Some(p_idx);
                // Match boot behavior: land cursor on the first task so the
                // user sees a highlight without having to press `j`.
                let project = &self.workspace.projects[p_idx];
                let initial_cursor = project.tasks.first().map(|t| TreeRowId::Task(t.id));
                let pid = project.id;
                self.screen.tree = TreeState {
                    expanded: std::collections::HashSet::new(),
                    cursor: initial_cursor,
                };
                // Persist active project so the next boot re-opens it.
                self.workspace.set_active_project(pid)?;
                self.workspace.save()?;
            }

            // ── tree cursor / expand (phase 3 drives these) ─────────────
            Effect::TreeMoveDown => {
                if let Some(p_idx) = self.screen.p_idx {
                    let project = &self.workspace.projects[p_idx];
                    let rows = super::tree::flatten(project, &self.screen.tree.expanded);
                    if self.screen.tree.cursor.is_none() {
                        // First keypress — land on the first row.
                        self.screen.tree.cursor = rows.first().map(|r| r.id);
                    } else {
                        let current = self
                            .screen
                            .tree
                            .cursor
                            .and_then(|c| super::tree::cursor_index(&rows, c))
                            .unwrap_or(0);
                        if let Some(next) = rows.get(current + 1) {
                            self.screen.tree.cursor = Some(next.id);
                        }
                    }
                }
            }
            Effect::TreeMoveUp => {
                if let Some(p_idx) = self.screen.p_idx {
                    let project = &self.workspace.projects[p_idx];
                    let rows = super::tree::flatten(project, &self.screen.tree.expanded);
                    if self.screen.tree.cursor.is_none() {
                        self.screen.tree.cursor = rows.last().map(|r| r.id);
                    } else {
                        let current = self
                            .screen
                            .tree
                            .cursor
                            .and_then(|c| super::tree::cursor_index(&rows, c))
                            .unwrap_or(0);
                        if current > 0
                            && let Some(prev) = rows.get(current - 1)
                        {
                            self.screen.tree.cursor = Some(prev.id);
                        }
                    }
                }
            }
            Effect::TreeExpandOrDescend => {
                if let Some(TreeRowId::Task(tid)) = self.screen.tree.cursor {
                    let p_idx = match self.screen.p_idx {
                        Some(i) => i,
                        None => return Ok(()),
                    };
                    let project = &self.workspace.projects[p_idx];
                    let has_subs = project
                        .tasks
                        .iter()
                        .find(|t| t.id == tid)
                        .map(|t| !t.subtasks.is_empty())
                        .unwrap_or(false);

                    if !has_subs {
                        return Ok(()); // leaf — no-op
                    }

                    if self.screen.tree.expanded.contains(&tid) {
                        // Already expanded — move cursor to first subtask.
                        let rows = super::tree::flatten(project, &self.screen.tree.expanded);
                        if let Some(first_sub) = rows.iter().find(
                            |r| matches!(r.id, TreeRowId::Subtask { task, .. } if task == tid),
                        ) {
                            self.screen.tree.cursor = Some(first_sub.id);
                        }
                    } else {
                        self.screen.tree.expanded.insert(tid);
                    }
                }
            }
            Effect::TreeCollapseOrAscend => {
                match self.screen.tree.cursor {
                    Some(TreeRowId::Task(tid)) => {
                        // Collapse if expanded, no-op if already collapsed.
                        self.screen.tree.expanded.remove(&tid);
                    }
                    Some(TreeRowId::Subtask { task: tid, .. }) => {
                        // Move cursor up to parent task.
                        self.screen.tree.cursor = Some(TreeRowId::Task(tid));
                    }
                    None => {}
                }
            }

            // ── mode / draft ────────────────────────────────────────────
            Effect::EnterInsert(target) => {
                self.screen.mode = ScreenMode::Insert(Draft::new(target));
            }
            Effect::CancelInsert => {
                self.screen.mode = ScreenMode::Navigate;
            }
            Effect::DraftPush(c) => {
                if let ScreenMode::Insert(ref mut d) = self.screen.mode {
                    d.text.push(c);
                }
            }
            Effect::DraftPop => {
                if let ScreenMode::Insert(ref mut d) = self.screen.mode {
                    d.text.pop();
                }
            }

            // ── overlay ─────────────────────────────────────────────────
            Effect::OpenOverlay(o) => {
                self.overlay = Some(o);
            }
            Effect::CloseOverlay => {
                self.overlay = None;
            }
            // Confirm pump: take the stored effect out of the overlay and
            // apply it. Only consume the overlay if it's a Confirm so future
            // overlay variants (Help, Picker, ...) survive the pump.
            Effect::ConfirmYes => {
                if matches!(self.overlay, Some(super::overlay::Overlay::Confirm(_)))
                    && let Some(super::overlay::Overlay::Confirm(c)) = self.overlay.take()
                {
                    self.apply(*c.on_confirm)?;
                }
            }

            // ── error display ───────────────────────────────────────────
            Effect::ShowError(msg) => {
                self.error = Some(msg);
            }

            // ── domain mutations ─────────────────────────────────────────
            Effect::CreateProject { name } => {
                let new_id = {
                    let p = self.workspace.add_project(name);
                    p.id
                };
                // Focus the new project.
                let p_idx = self
                    .workspace
                    .projects
                    .iter()
                    .position(|p| p.id == new_id)
                    .unwrap();
                self.screen.p_idx = Some(p_idx);
                self.screen.sidebar_cursor = p_idx;
                self.screen.tree = TreeState::default();
                self.screen.focus = ProjectFocus::Tree;
                self.screen.mode = ScreenMode::Navigate;
                // Mark the freshly-created project as active so it survives
                // a restart.
                self.workspace.set_active_project(new_id)?;
                self.workspace.save()?;
            }

            Effect::CreateTask { p_idx, description } => {
                let new_id = {
                    let project = &mut self.workspace.projects[p_idx];
                    let t = project.add_task(description, Priority::Medium);
                    t.id
                };
                self.screen.tree.cursor = Some(TreeRowId::Task(new_id));
                self.screen.mode = ScreenMode::Navigate;
                self.workspace.save()?;
            }

            Effect::DeleteTask { p_idx, task_id } => {
                self.workspace.projects[p_idx].delete_task(task_id)?;
                // Fix cursor.
                let rows = super::tree::flatten(
                    &self.workspace.projects[p_idx],
                    &self.screen.tree.expanded,
                );
                self.screen.tree.cursor = rows.first().map(|r| r.id);
                self.workspace.save()?;
            }

            Effect::CreateSubtask {
                p_idx,
                task_id,
                description,
            } => {
                let sub_id = {
                    let project = &mut self.workspace.projects[p_idx];
                    let s = project.add_subtask(task_id, description, Priority::Medium)?;
                    s.id
                };
                // Auto-expand the parent task.
                self.screen.tree.expanded.insert(task_id);
                self.screen.tree.cursor = Some(TreeRowId::Subtask {
                    task: task_id,
                    sub: sub_id,
                });
                self.screen.mode = ScreenMode::Navigate;
                self.workspace.save()?;
            }

            Effect::DeleteSubtask {
                p_idx,
                task_id,
                sub_id,
            } => {
                self.workspace.projects[p_idx].delete_subtask(task_id, sub_id)?;
                // Move cursor back to parent task.
                self.screen.tree.cursor = Some(TreeRowId::Task(task_id));
                self.workspace.save()?;
            }

            Effect::ToggleStatus(row_id) => {
                let p_idx = match self.screen.p_idx {
                    Some(i) => i,
                    None => return Ok(()),
                };
                match row_id {
                    TreeRowId::Subtask {
                        task: tid,
                        sub: sid,
                    } => {
                        let project = &mut self.workspace.projects[p_idx];
                        let task = project.get_task_mut(tid)?;
                        let sub_idx = task.get_subtask(sid)?;
                        if task.subtasks[sub_idx].completed {
                            project.uncomplete_subtask(tid, sid)?;
                        } else {
                            project.complete_subtask(tid, sid)?;
                        }
                        self.workspace.save()?;
                    }
                    TreeRowId::Task(tid) => {
                        let project = &mut self.workspace.projects[p_idx];
                        let task = project.get_task_mut(tid)?;
                        let next = match task.status() {
                            Status::New => Status::InProgress,
                            Status::InProgress => Status::Completed,
                            Status::Completed => Status::New,
                        };
                        match task.set_status(next) {
                            Ok(()) => self.workspace.save()?,
                            Err(e) => {
                                self.apply(Effect::ShowError(e.to_string()))?;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
