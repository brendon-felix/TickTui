use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};

use crate::ui::{
    taskeditor::TaskEditor,
    tasklist::{TaskItem, TaskList},
};

enum ActivePane {
    TaskList,
    TaskEditor,
}

pub struct NormalModeUI {
    task_list: TaskList,
    task_editor: TaskEditor,
    active_pane: ActivePane,
}
impl NormalModeUI {
    pub fn new() -> Self {
        let task_list = TaskList::new(vec![
            TaskItem::new("Go to the laundromat").with_description("Remember to bring quarters."),
            TaskItem::new("Buy groceries").with_description("- Milk\n- Eggs\n- Bread\n- Fruits"),
            TaskItem::new("Finish the report").with_description("Due by end of the week."),
            TaskItem::new("Call Alice").with_description("Discuss the project updates."),
            TaskItem::new("Plan weekend trip").with_description("Check the weather forecast."),
            TaskItem::new("Read a book").with_description("Start with chapter 1."),
            TaskItem::new("Exercise for 30 minutes")
                .with_description("Focus on cardio.\nWarm up first."),
            TaskItem::new("Clean the house"),
            TaskItem::new("Prepare presentation").with_description(
                r#"Rough draft:
- Introduction
  - Objectives
  - Agenda
  - Key topics
- Main points
- Conclusion
"#,
            ),
        ]);
        let mut task_editor = TaskEditor::new();
        task_editor.deactivate();
        Self {
            task_list,
            task_editor,
            active_pane: ActivePane::TaskList,
        }
    }

    pub fn is_in_insert_mode(&self) -> bool {
        false
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        match self.active_pane {
            ActivePane::TaskList => match key_event.code {
                KeyCode::Enter => {
                    self.active_pane = ActivePane::TaskEditor;
                    self.task_list.deactivate();
                    self.task_editor.activate();
                }
                _ => self.task_list.handle_key_event(key_event),
            },
            ActivePane::TaskEditor => match key_event.code {
                KeyCode::Esc => {
                    if self.task_editor.is_in_insert_mode() {
                        self.task_editor.handle_key_event(key_event);
                    } else {
                        self.active_pane = ActivePane::TaskList;
                        self.task_editor.deactivate();
                        self.task_list.activate();
                    }
                }
                _ => self.task_editor.handle_key_event(key_event),
            },
        }
        if self.task_list.task_changed {
            if let Some(selected_task) = self.task_list.get_current_task() {
                self.task_editor.load_task(&selected_task);
            }
            self.task_list.task_changed = false;
        }
    }

    pub fn handle_mouse_event(&mut self, _mouse_event: MouseEvent) {
        // Handle mouse events specific to Normal Mode here
    }

    pub fn draw(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::new(
            Direction::Horizontal,
            vec![Constraint::Percentage(40), Constraint::Percentage(60)],
        )
        .split(area);
        self.task_list.draw(f, chunks[0]);
        self.task_editor.draw(f, chunks[1]);
    }
}
