use chrono::{DateTime, Local, Utc};
use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, Clear, Widget},
};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tachyonfx::EffectManager;
use ticks::tasks::Task;

use crate::{
    tasks::{is_due_today, is_overdue},
    ui::{
        animate::{Animation, AnimationDirection, AnimationType},
        focuslist::{FocusList, FocusListItem, state::FocusListState},
    },
};

#[derive(Default)]
pub struct FocusModeUI {
    // test_content: String,
    tasks: Vec<Arc<Task>>,
    list: FocusList<'static>,
    list_state: FocusListState<'static>,
    // prev_buf: Buffer,
    // focus_buf: Buffer,
    // next_buf: Buffer,
    effects: EffectManager<()>,
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
        let idx = self.list.focused_index();
        match key_event.code {
            KeyCode::Char('j') => {
                self.list.focus_next();
                if idx != self.list.focused_index() {
                    // let animation_type = AnimationType::Resize {
                    //     dir: AnimationDirection::Vertical,
                    //     amount: 10,
                    // };
                    let animation_type = AnimationType::Translate { x: 0, y: -10 };
                    // let duration = Duration::from_millis(200);
                    // let animation = Animation::new(animation_type, duration);
                    // self.list_state.start_focused_animation(animation);
                    // let animation_type = AnimationType::Translate { x: 0, y: -10 };
                    let duration = Duration::from_millis(200);
                    let animation = Animation::new(animation_type, duration);
                    self.list_state.start_focused_animation(animation.clone());
                    self.list_state.start_prev_animation(animation.clone());
                    self.list_state.start_next_animation(animation);
                }
            }
            KeyCode::Char('k') => {
                self.list.focus_previous();
                if idx != self.list.focused_index() {
                    // let animation_type = AnimationType::Resize {
                    //     dir: AnimationDirection::Vertical,
                    //     amount: -10,
                    // };
                    let animation_type = AnimationType::Translate { x: 0, y: 10 };
                    let duration = Duration::from_millis(200);
                    let animation = Animation::new(animation_type, duration);
                    self.list_state.start_focused_animation(animation.clone());
                    self.list_state.start_prev_animation(animation.clone());
                    self.list_state.start_next_animation(animation);
                }
            }
            _ => {}
        }
        if idx != self.list.focused_index() {
            //     let (prev, focus, next) = self.list_state.get_sub_areas();
            //     let c = Color::Rgb(25, 25, 25);
            //     let timer = EffectTimer::from_ms(100, Interpolation::Linear);
            //     if let Some(a) = prev {
            //         let fx = fx::fade_from_fg(c, timer).with_area(a);
            //         self.effects.add_effect(fx);
            //     }
            //     if let Some(a) = focus {
            //         let fx = fx::fade_from_fg(c, timer).with_area(a);
            //         self.effects.add_effect(fx);
            //     }
            //     if let Some(a) = next {
            //         let fx = fx::fade_from_fg(c, timer).with_area(a);
            //         self.effects.add_effect(fx);
            //     }
        }
    }

    pub fn handle_mouse_event(&mut self, _mouse_event: MouseEvent) {
        // Handle mouse events specific to Focus Mode here
    }

    pub fn draw(&mut self, f: &mut Frame, area: Rect, last_frame: Instant) {
        Clear.render(f.area(), f.buffer_mut());
        Block::default()
            .style(Style::default().bg(Color::Rgb(25, 25, 25)))
            .render(f.area(), f.buffer_mut());
        let items: Vec<FocusListItem> = self
            .tasks
            .iter()
            .map(|task| create_list_item(task))
            .collect();

        self.list.set_items(items);
        // self.list_state.set_last_frame(last_frame);
        self.list_state.update_animations();

        // if let Some(block) = self.current_block.clone() {
        //     task_list = task_list.with_block(block);
        // }
        f.render_stateful_widget(&self.list, area, &mut self.list_state);
        let elapsed = last_frame.elapsed();
        self.effects
            .process_effects(elapsed.into(), f.buffer_mut(), area);
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
        match (is_today, is_all_day) {
            (true, _) => Some(local.format("Today %I:%M %p").to_string()),
            (false, true) => Some(local.format("%m/%d/%Y").to_string()),
            (false, false) => Some(local.format("%m/%d/%Y %I:%M %p").to_string()),
        }
    }
}
