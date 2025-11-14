use ratatui::widgets::Widget;

pub struct FocusList<T> {
    items: Vec<T>,
    focused_index: Option<usize>,
}

impl<T> FocusList<T> {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            focused_index: None,
        }
    }

    pub fn with_items(mut self, items: Vec<T>) -> Self {
        self.items = items;
        self
    }

    pub fn set_items(&mut self, items: Vec<T>) {
        self.items = items;
        self.focused_index = None;
    }

    pub fn items(&self) -> &Vec<T> {
        &self.items
    }

    pub fn focused_index(&self) -> Option<usize> {
        self.focused_index
    }

    pub fn focus_previous(&mut self) {
        if self.items.is_empty() {
            self.focused_index = None;
            return;
        }

        match self.focused_index {
            Some(i) => {
                if i > 0 {
                    self.focused_index = Some(i - 1);
                }
                // If i == 0, stay at first item (don't deselect)
            }
            None => {
                // No focus, focus last item
                self.focused_index = Some(self.items.len() - 1);
            }
        }
    }

    pub fn focus_next(&mut self) {
        if self.items.is_empty() {
            self.focused_index = None;
            return;
        }

        match self.focused_index {
            Some(i) => {
                if i + 1 < self.items.len() {
                    self.focused_index = Some(i + 1);
                }
                // If i is at the last item, stay there (don't deselect)
            }
            None => {
                // No focus, focus first item
                self.focused_index = Some(0);
            }
        }
    }
}

impl Widget for FocusList<String> {
    fn render(self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        for (i, item) in self.items.iter().enumerate() {
            let style = if Some(i) == self.focused_index {
                ratatui::style::Style::default().fg(ratatui::style::Color::Yellow)
            } else {
                ratatui::style::Style::default()
            };
            buf.set_string(area.x, area.y + i as u16, item, style);
        }
    }
}
