use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, StatefulWidget},
};

use crate::{
    ui::editor::create_block,
    ui::multiselect::{MultiSelectList, MultiSelectListItem, MultiSelectListState},
    // ui::{AppWidget, WidgetStyle},
};

const ITEM_HEIGHT: u16 = 3;

pub enum TaskListMode {
    Normal,
    Visual,
}

pub struct Task {
    name: String,
}

impl Task {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

// #[derive(Default)]
// pub struct TaskListStyle {
//     selected: Style,
//     unselected: Style,
// }

#[derive(Default)]
pub struct TaskList {
    tasks: Vec<Task>,
    pub list_state: MultiSelectListState,
    current_block: Option<Block<'static>>,
    title: Option<String>,
    // style: TaskListStyle,
    style: Style,
    last_area_pos: Option<Position>,
}

impl TaskList {
    pub fn with_tasks(mut self, tasks: Vec<Task>) -> Self {
        self.tasks = tasks;
        self
    }

    pub fn add_task(&mut self, name: &str) {
        self.tasks.push(Task {
            name: name.to_string(),
        });
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
                self.list_state.visual_start = None;
            } else {
                self.remove_task(curr);
                self.list_state.select(None);
            }
        }
    }

    pub fn set_block(&mut self, block: Block<'static>) {
        self.current_block = Some(block);
    }

    pub fn set_style(&mut self, style: Style) {
        self.style = style;
    }

    pub fn get_tasks(&self) -> &Vec<Task> {
        &self.tasks
    }

    pub fn get_list_state(&self) -> &MultiSelectListState {
        &self.list_state
    }

    pub fn get_title_owned(&self) -> Option<String> {
        self.title.clone()
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

// impl Widget for TaskList {
//     fn render(self, area: Rect, buf: &mut Buffer) {
//         let items: Vec<String> = self.tasks.iter().map(|task| task.name.clone()).collect();
//         let mut task_list = List::default().scroll_padding(2).items(items);
//         if let Some(block) = self.current_block {
//             task_list = task_list.block(block);
//         }
//         task_list.render(area, buf);
//     }
// }

// impl WidgetRef for TaskList {
//     fn render_ref(&self, area: Rect, buf: &mut Buffer) {
//         let items: Vec<String> = self.tasks.iter().map(|task| task.name.clone()).collect();
//         let mut task_list = List::default().scroll_padding(2).items(items);
//         if let Some(block) = self.current_block.clone() {
//             task_list = task_list.block(block);
//         }
//         task_list.render(area, buf);
//     }
// }

impl StatefulWidget for &mut TaskList {
    type State = MultiSelectListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let items: Vec<MultiSelectListItem> = self
            .tasks
            .iter()
            .map(|task| MultiSelectListItem::new(vec![Line::from(task.name.clone())]))
            .collect();
        let mut task_list = MultiSelectList::new(items).with_highlight_style(
            Style::new()
                .bg(Color::Rgb(50, 50, 50))
                .add_modifier(Modifier::BOLD),
        );

        if let Some(block) = self.current_block.clone() {
            task_list = task_list.with_block(block);
        }
        task_list.render(area, buf, state);
    }
}
