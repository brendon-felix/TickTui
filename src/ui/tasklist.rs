use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Block,
};

use crate::{
    // ui::editor::create_block,
    ui::multiselect::{MultiSelectList, MultiSelectListState},
    // ui::{AppWidget, WidgetStyle},
};

// const ITEM_HEIGHT: u16 = 3;

// pub enum TaskListMode {
//     Normal,
//     Visual,
// }

pub struct TaskItem {
    name: String,
    description: String,
}

impl TaskItem {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: String::new(),
        }
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_description(&self) -> &str {
        &self.description
    }
}

#[derive(Default)]
pub struct TaskList {
    tasks: Vec<TaskItem>,
    list_state: MultiSelectListState,
    style: Style,
    current_block: Option<Block<'static>>,
    pub task_changed: bool,
}

#[allow(dead_code)]
impl TaskList {
    pub fn new(tasks: Vec<TaskItem>) -> Self {
        let mut list_state = MultiSelectListState::default();
        list_state.select(Some(0));
        let current_block = Some(
            Block::default()
                .title("Tasks")
                .borders(ratatui::widgets::Borders::ALL),
        );
        Self {
            tasks,
            list_state,
            style: Style::default(),
            current_block,
            task_changed: true,
        }
    }

    pub fn activate(&mut self) {
        if self.tasks.is_empty() {
            self.list_state.select(None);
        } else if self.list_state.selected().is_none() {
            self.list_state.select(Some(0));
        }
        self.current_block = Some(
            Block::default()
                .title("Tasks")
                .borders(ratatui::widgets::Borders::ALL),
        );
        self.style = Style::default();
    }

    pub fn deactivate(&mut self) {
        self.current_block = Some(
            Block::default()
                .title("Tasks")
                .borders(ratatui::widgets::Borders::ALL)
                .style(Style::default().add_modifier(Modifier::DIM)),
        );
        self.style = Style::default().add_modifier(Modifier::DIM);
    }

    pub fn with_tasks(mut self, tasks: Vec<TaskItem>) -> Self {
        self.tasks = tasks;
        self
    }

    pub fn add_task(&mut self, name: &str) {
        self.tasks.push(TaskItem::new(name));
    }

    pub fn remove_task(&mut self, index: usize) {
        if index < self.tasks.len() {
            self.tasks.remove(index);
        }
    }

    pub fn remove_range_inclusive(&mut self, range: (usize, usize)) {
        let (start, end) = range;
        if start < self.tasks.len() && end < self.tasks.len() && start <= end {
            self.tasks.drain(start..=end);
        }
    }

    pub fn remove_selected_tasks(&mut self) {
        if let Some(curr) = self.list_state.selected() {
            if let Some(start) = self.list_state.visual_start {
                let (s, e) = if curr >= start {
                    (start, curr)
                } else {
                    (curr, start)
                };
                self.remove_range_inclusive((s, e));
                // self.list_state.select_next();
                self.list_state.select(Some(s));
                self.list_state.end_visual_selection();
            } else {
                self.remove_task(curr);
                self.list_state.select(Some(curr));
            }
        }
    }

    pub fn set_block(&mut self, block: Block<'static>) {
        self.current_block = Some(block);
    }

    pub fn get_tasks(&self) -> &Vec<TaskItem> {
        &self.tasks
    }

    pub fn get_list_state(&self) -> &MultiSelectListState {
        &self.list_state
    }

    pub fn get_current_task(&self) -> Option<&TaskItem> {
        if let Some(idx) = self.list_state.selected() {
            self.tasks.get(idx)
        } else {
            None
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        let idx = self.list_state.selected();
        if self.list_state.is_in_visual_mode() {
            match key.code {
                KeyCode::Char('j') | KeyCode::Down => self.list_state.select_next(),
                KeyCode::Char('k') | KeyCode::Up => self.list_state.select_previous(),
                KeyCode::Char('g') => self.list_state.select_first(),
                KeyCode::Char('G') => self.list_state.select_last(),
                KeyCode::Char('d') => self.remove_selected_tasks(),
                KeyCode::Esc => self.list_state.end_visual_selection(),
                _ => {}
            }
            return;
        } else {
            match key.code {
                KeyCode::Char('j') | KeyCode::Down => self.list_state.select_next(),
                KeyCode::Char('k') | KeyCode::Up => self.list_state.select_previous(),
                KeyCode::Char('g') => self.list_state.select_first(),
                KeyCode::Char('G') => self.list_state.select_last(),
                KeyCode::Char('v') | KeyCode::Char('V') => self.list_state.start_visual_selection(),
                KeyCode::Char('d') => self.remove_selected_tasks(),
                _ => {}
            }
        }
        self.task_changed = idx != self.list_state.selected();
    }

    pub fn draw(&mut self, f: &mut Frame, area: Rect) {
        let items: Vec<String> = self.tasks.iter().map(|task| task.name.clone()).collect();
        let mut task_list = MultiSelectList::new(items)
            .with_style(self.style)
            .with_highlight_style(
                Style::new()
                    .bg(Color::Rgb(50, 50, 50))
                    .add_modifier(Modifier::BOLD),
            );

        if let Some(block) = self.current_block.clone() {
            task_list = task_list.with_block(block);
        }
        f.render_stateful_widget(task_list, area, &mut self.list_state);
    }
}

// impl AppWidget for TaskList {
//     fn set_widget_style(&mut self, style: WidgetStyle) {
//         match style {
//             WidgetStyle::Active => {
//                 let is_active = true;
//                 let borders = Borders::ALL;
//                 self.set_block(create_block(self.get_title_owned(), is_active, borders));
//                 self.set_style(Style::default());
//             }
//             WidgetStyle::Inactive => {
//                 let is_active = false;
//                 let borders = Borders::ALL;
//                 self.set_block(create_block(self.get_title_owned(), is_active, borders));
//                 self.set_style(Style::default().add_modifier(Modifier::DIM));
//             }
//         }
//     }

//     fn set_last_area_pos(&mut self, area_pos: Position) {
//         self.last_area_pos = Some(area_pos);
//     }

//     fn on_click(&mut self, pos: Position) {
//         if let Some(area_pos) = self.last_area_pos {
//             let local = Position::new(
//                 pos.x.saturating_sub(area_pos.x),
//                 pos.y.saturating_sub(area_pos.y),
//             );
//             let (_x, y) = if let Some(_block) = &self.current_block {
//                 (local.x.saturating_sub(1), local.y.saturating_sub(1))
//             } else {
//                 local.into()
//             };
//             let index = (y / ITEM_HEIGHT) as usize;
//             if index < self.tasks.len() {
//                 self.list_state.select(Some(index));
//             }
//         }
//     }
// }
