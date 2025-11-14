use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{Frame, layout::Rect, text::Text, widgets::Paragraph};

use crate::ui::{focuslist::FocusList, tasklist::TaskItem};

#[allow(dead_code)]
pub struct FocusModeUI {
    test_content: String,
    focus_list: FocusList<TaskItem>,
}

#[allow(dead_code)]
impl FocusModeUI {
    pub fn new() -> Self {
        Self {
            test_content: String::from("Focus Mode"),
            focus_list: FocusList::new(),
        }
    }

    pub fn set_items(&mut self, items: Vec<TaskItem>) {
        self.focus_list.set_items(items);
    }

    pub fn is_in_insert_mode(&self) -> bool {
        false
    }

    pub fn handle_key_event(&mut self, _key_event: KeyEvent) {
        // Handle key events specific to Focus Mode here
    }

    pub fn handle_mouse_event(&mut self, _mouse_event: MouseEvent) {
        // Handle mouse events specific to Focus Mode here
    }

    pub fn draw(&mut self, f: &mut Frame, area: Rect) {
        let paragraph = Paragraph::new(Text::from(self.test_content.as_str()));
        f.render_widget(paragraph, area);
    }
}
