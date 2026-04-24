use crate::models::Task;
use ratatui::{buffer::Buffer, layout::Rect, style::Style, widgets::Widget};

pub struct TaskListWidget<'a> {
    tasks: &'a [Task],
}

impl<'a> TaskListWidget<'a> {
    pub fn new(tasks: &'a [Task]) -> Self {
        TaskListWidget { tasks }
    }
}

impl Widget for TaskListWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for (i, task) in self.tasks.iter().enumerate() {
            buf.set_string(
                area.x,
                area.y + i as u16,
                &task.description,
                Style::default(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Priority, Status};
    use ratatui::{Terminal, backend::TestBackend};

    #[test]
    fn test_task_widget() {
        let t1 = Task::new(
            0,
            String::from("Testing task"),
            Priority::Medium,
            Status::New,
            None,
        );
        let t2 = Task::new(
            0,
            String::from("Testing task"),
            Priority::Medium,
            Status::New,
            None,
        );
        let tasks = vec![t1, t2];

        let backend = TestBackend::new(80, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let widget = TaskListWidget::new(&tasks);
                frame.render_widget(widget, frame.area());
            })
            .unwrap();

        let buf = terminal.backend().buffer().clone();

        let task_desc_size: u16 = tasks[0].description.len() as u16;
        let row0: String = (0..task_desc_size)
            .map(|x| buf.cell((x, 0)).unwrap().symbol().to_owned())
            .collect();
        assert_eq!(row0, tasks[0].description);
    }
}
