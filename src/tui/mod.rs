mod widgets;
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    style::{Color, Style},
    widgets::{Block, List, ListItem, ListState, Paragraph},
};
pub use widgets::TaskListWidget;

use crate::{AppError, workspace::Workspace};
use crossterm::event::{Event, KeyCode, KeyEventKind};

enum Screen {
    Projects,
    Tasks,
}

enum Mode {
    Normal,
    Creating { input: String },
    Confirming { target: String },
}

struct App<'a> {
    workspace: &'a mut Workspace,
    project_state: ListState,
    mode: Mode,
    last_key: Option<KeyCode>,
    screen: Screen,
    input_focused: bool,
}

impl<'a> App<'a> {
    fn new(workspace: &'a mut Workspace) -> Self {
        let mut project_state = ListState::default();
        if !workspace.projects.is_empty() {
            project_state.select(Some(0));
        }
        App {
            workspace,
            project_state,
            mode: Mode::Normal,
            last_key: None,
            screen: Screen::Projects,
            input_focused: false,
        }
    }

    fn project_count(&self) -> usize {
        self.workspace.projects.len()
    }
}

fn render(frame: &mut Frame, app: &mut App) {
    match app.screen {
        Screen::Projects => render_projects(frame, app),
        Screen::Tasks => render_tasks(frame, app),
    }
}

fn render_projects(frame: &mut Frame, app: &mut App) {
    let [title_area, content_area, _, input_area, help_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Length(3),
    ])
    .areas(frame.area());

    frame.render_widget(
        Paragraph::new("An awesome todo list").block(Block::bordered().title("rtodo")),
        title_area,
    );

    let project_widgets: Vec<ListItem<'_>> = app
        .workspace
        .projects
        .iter()
        .map(|p| {
            ListItem::new(format!(
                "{:<40} {}/{}",
                p.name,
                p.complete_tasks(),
                p.task_count()
            ))
        })
        .collect();

    let project_widget = List::new(project_widgets)
        .block(Block::bordered().title("Projects"))
        .highlight_style(Style::new().bg(Color::Cyan).fg(Color::Black))
        .highlight_symbol("▶  ");

    frame.render_stateful_widget(project_widget, content_area, &mut app.project_state);

    let input_text = match &app.mode {
        Mode::Creating { input } => format!("Project Name: {}_", input),
        _ if app.input_focused => "+ New project...".to_string(),
        _ => String::new(),
    };

    let input_border = if app.input_focused {
        Block::bordered().style(Style::new().fg(Color::Cyan))
    } else {
        Block::bordered()
    };

    frame.render_widget(Paragraph::new(input_text).block(input_border), input_area);

    let help_text = match &app.mode {
        Mode::Confirming { target } => format!("⚠  Delete \"{}\"?   y confirm   n cancel", target),
        Mode::Creating { .. } => "↵ create   Esc cancel".to_string(),
        _ => "j/k navigate   ↵/l open   dd delete   q quit".to_string(),
    };

    frame.render_widget(
        Paragraph::new(help_text).block(Block::bordered().title("Help")),
        help_area,
    );
}

fn render_tasks(frame: &mut Frame, app: &mut App) {
    let project_name = app
        .project_state
        .selected()
        .and_then(|i| app.workspace.projects.get(i))
        .map(|p| p.name.as_str())
        .unwrap_or("unknown");

    let [title_area, content_area, help_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    frame.render_widget(
        Paragraph::new(format!("rtodo  ›  {}", project_name)),
        title_area,
    );
    frame.render_widget(
        Paragraph::new("Tasks view — coming soon").block(Block::bordered()),
        content_area,
    );
    frame.render_widget(Paragraph::new("h/Esc back   q quit"), help_area);
}

fn run(terminal: &mut DefaultTerminal, app: &mut App) -> Result<(), AppError> {
    loop {
        terminal.draw(|frame| render(frame, app))?;

        if let Event::Key(key) = crossterm::event::read()?
            && key.kind == KeyEventKind::Press
        {
            match app.screen {
                Screen::Tasks => match key.code {
                    KeyCode::Char('h') | KeyCode::Esc => app.screen = Screen::Projects,
                    KeyCode::Char('q') => break,
                    _ => {}
                },
                Screen::Projects => match &mut app.mode {
                    Mode::Normal => match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('j') => {
                            if app.input_focused {
                                // already at bottom, do nothing
                            } else if app.project_state.selected()
                                == Some(app.project_count().saturating_sub(1))
                            {
                                app.project_state.select(None);
                                app.input_focused = true;
                            } else {
                                app.project_state.select_next();
                            }
                        }
                        KeyCode::Char('k') => {
                            if app.input_focused {
                                app.input_focused = false;
                                let last = app.project_count().saturating_sub(1);
                                app.project_state.select(Some(last));
                            } else {
                                app.project_state.select_previous();
                            }
                        }
                        KeyCode::Char('l') | KeyCode::Enter => {
                            if app.input_focused {
                                app.mode = Mode::Creating {
                                    input: String::new(),
                                };
                            } else if app.project_state.selected().is_some() {
                                app.screen = Screen::Tasks;
                            }
                        }
                        KeyCode::Char('d') => {
                            if app.last_key == Some(KeyCode::Char('d'))
                                && let Some(selected) = app.project_state.selected()
                            {
                                let idx = app.workspace.get_project(selected as u32)?;
                                let name = app.workspace.projects[idx].name.clone();
                                app.mode = Mode::Confirming { target: name };
                            }
                        }
                        _ => {}
                    },
                    Mode::Creating { input } => match key.code {
                        KeyCode::Char(c) => input.push(c),
                        KeyCode::Backspace => {
                            input.pop();
                        }
                        KeyCode::Enter => {
                            let name = input.clone();
                            app.workspace.add_project(name);
                            app.mode = Mode::Normal;
                            app.input_focused = false;
                        }
                        KeyCode::Esc => {
                            app.mode = Mode::Normal;
                            app.input_focused = false;
                        }
                        _ => {}
                    },
                    Mode::Confirming { .. } => {
                        if key.code == KeyCode::Char('y')
                            && let Some(selected) = app.project_state.selected()
                        {
                            app.workspace.delete_project(selected as u32)?;
                            let new_len = app.project_count();
                            if new_len == 0 {
                                app.project_state.select(None);
                            } else {
                                app.project_state.select(Some(selected.min(new_len - 1)));
                            }
                        }
                        app.mode = Mode::Normal;
                    }
                },
            }
            app.last_key = Some(key.code);
        }
    }

    Ok(())
}

pub fn main(workspace: &mut Workspace) -> Result<(), AppError> {
    let mut app = App::new(workspace);
    let mut terminal = ratatui::init();
    let r = run(&mut terminal, &mut app);
    ratatui::restore();
    r
}
