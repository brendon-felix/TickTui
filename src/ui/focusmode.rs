use chrono::{DateTime, Local, Utc};
use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::Line,
};
use std::sync::Arc;
use ticks::tasks::Task;

use crate::{
    tasks::{is_due_today, is_overdue},
    ui::focuslist::{FocusList, FocusListItem},
};

#[derive(Default)]
pub struct FocusModeUI {
    // test_content: String,
    tasks: Vec<Arc<Task>>,
    list: FocusList<'static>,
    // list_state: FocusListState,
}

impl FocusModeUI {
    // pub fn with_tasks(mut self, tasks: Vec<Arc<Task>>) -> Self {
    //     self.tasks = tasks;
    //     self
    // }

    pub fn set_tasks(&mut self, tasks: Vec<Arc<Task>>) {
        self.tasks = tasks;
        if self.list.focused_index().is_none() && !self.tasks.is_empty() {
            self.list.focus(Some(0));
        } else if self.tasks.is_empty() {
            self.list.focus(None);
        }
    }

    pub fn filter_tasks<F>(&mut self, filter_fn: F)
    where
        F: Fn(DateTime<Local>, &Task) -> bool,
    {
        let now = Local::now();
        self.tasks.retain(|task| filter_fn(now, task));
        if self.tasks.is_empty() {
            self.list.focus(None);
        } else if let Some(selected) = self.list.focused_index() {
            if selected >= self.tasks.len() {
                self.list.focus(Some(self.tasks.len() - 1));
            }
        }
    }

    pub fn update_tasks(&mut self, tasks: Vec<Arc<Task>>) {
        self.set_tasks(tasks);
        self.filter_tasks(|now, task| is_due_today(now, task) | is_overdue(now, task));
        // self.task_list.tasks_loaded = true;
    }

    pub fn is_in_insert_mode(&self) -> bool {
        false
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('j') => {
                self.list.focus_next();
            }
            KeyCode::Char('k') => {
                self.list.focus_previous();
            }
            _ => {}
        }
    }

    pub fn handle_mouse_event(&mut self, _mouse_event: MouseEvent) {
        // Handle mouse events specific to Focus Mode here
    }

    pub fn draw(&mut self, f: &mut Frame, area: Rect) {
        let items: Vec<FocusListItem> = self
            .tasks
            .iter()
            .map(|task| create_list_item(task))
            .collect();

        self.list.set_items(items);

        // if let Some(block) = self.current_block.clone() {
        //     task_list = task_list.with_block(block);
        // }
        f.render_widget(&self.list, area);
    }
}

fn create_list_item(task: &Arc<Task>) -> FocusListItem<'static> {
    let now = chrono::Local::now();
    let is_today = is_due_today(now, task);

    let line1 = Line::from("");
    let line2 = Line::from(task.title.clone());
    let line3 = if let Some(date_str) = format_date(&task.due_date, task.is_all_day, is_today) {
        let mut line = Line::from(date_str);
        if is_overdue(now, task) {
            line = line.style(Style::default().fg(Color::Red).dim());
        } else {
            line = line.style(Style::default().dim());
        }
        line
    } else {
        Line::from("")
    };
    FocusListItem::new(vec![line1, line2, line3])
}

fn format_date(dt: &DateTime<Utc>, is_all_day: bool, is_today: bool) -> Option<String> {
    if dt.timestamp() == 0 {
        None
    } else {
        let local: DateTime<Local> = dt.with_timezone(&Local);
        // if is_all_day {
        //     Some(local.format("%m/%d/%Y").to_string())
        // } else {
        //     Some(local.format("%m/%d/%Y %I:%M %p").to_string())
        // }
        match (is_today, is_all_day) {
            (true, _) => Some(local.format("Today %I:%M %p").to_string()),
            (false, true) => Some(local.format("%m/%d/%Y").to_string()),
            (false, false) => Some(local.format("%m/%d/%Y %I:%M %p").to_string()),
        }
    }
}
