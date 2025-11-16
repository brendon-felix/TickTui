mod focused;
pub mod state;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::Text,
    widgets::{Block, BorderType, Borders, Paragraph, StatefulWidget, Widget},
};

use state::FocusListState;

use crate::ui::focuslist::focused::{FocusedItem, NextPrevItem};

#[derive(Clone)]
pub struct FocusListItem<'a> {
    content: Text<'a>,
    style: Style,
}

impl<'a> FocusListItem<'a> {
    pub fn new<T>(content: T) -> Self
    where
        T: Into<Text<'a>>,
    {
        Self {
            content: content.into(),
            style: Style::default(),
        }
    }
}

impl<'a, T> From<T> for FocusListItem<'a>
where
    T: Into<Text<'a>>,
{
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

#[derive(Default)]
pub struct FocusList<'a> {
    focused_index: Option<usize>,
    block: Option<Block<'a>>,
    items: Vec<FocusListItem<'a>>,
    style: Style,
}

#[allow(dead_code)]
impl<'a> FocusList<'a> {
    pub fn new<T>(items: T) -> Self
    where
        T: IntoIterator,
        T::Item: Into<FocusListItem<'a>>,
    {
        Self {
            items: items.into_iter().map(Into::into).collect(),
            ..Self::default()
        }
    }

    pub fn with_block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn with_focused_index(mut self, index: usize) -> Self {
        self.focused_index = Some(index);
        self
    }

    pub fn set_items<T>(&mut self, items: T)
    where
        T: IntoIterator,
        T::Item: Into<FocusListItem<'a>>,
    {
        self.items = items.into_iter().map(Into::into).collect();
    }

    pub fn focused_index(&self) -> Option<usize> {
        self.focused_index
    }

    pub fn next_index(&self) -> Option<usize> {
        if let Some(current) = self.focused_index {
            let next = current.saturating_add(1);
            if next < self.items.len() {
                Some(next)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn previous_index(&self) -> Option<usize> {
        if let Some(current) = self.focused_index {
            if current > 0 {
                Some(current.saturating_sub(1))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn focus(&mut self, index: Option<usize>) {
        if let Some(i) = index
        // && !(self.focused_index() == Some(i))
        {
            if i >= self.items.len() {
                self.focused_index = Some(self.items.len().saturating_sub(1));
            } else {
                self.focused_index = Some(i);
            }
        } else {
            self.focused_index = None;
        }
    }

    pub fn focus_next(&mut self) {
        let next = self.focused_index.map_or(0, |i| i.saturating_add(1));
        self.focus(Some(next));
    }

    pub fn focus_previous(&mut self) {
        let previous = self
            .focused_index
            .map_or(usize::MAX, |i| i.saturating_sub(1));
        self.focus(Some(previous));
    }

    pub fn focus_first(&mut self) {
        self.focus(Some(0));
    }

    pub fn focus_last(&mut self) {
        self.focus(Some(usize::MAX));
    }
}

impl<'a> StatefulWidget for &FocusList<'a> {
    type State = FocusListState<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(7), // 1: previous item
                Constraint::Max(3),
                Constraint::Length(7), // 3: focused item
                Constraint::Max(3),
                Constraint::Length(7), // 5: next item
                Constraint::Fill(1),
            ])
            .split(area)
            .iter()
            .enumerate()
            .for_each(|(i, rect)| match i {
                1 => {
                    if let Some(area) = state.get_prev_area() {
                        if let Some(previous) = self.previous_index() {
                            if previous < self.items.len() {
                                let item = self.items[previous].clone();
                                if state.prev_area.as_ref().unwrap().is_completed() {
                                    state.prev_old_item = Some(item.clone());
                                    NextPrevItem::new(item).render(area, buf);
                                } else {
                                    if let Some(old_item) = state.prev_old_item.clone() {
                                        NextPrevItem::new(old_item).render(area, buf);
                                    }
                                }
                            }
                        }
                    } else {
                        let centered = Layout::default()
                            .direction(Direction::Horizontal)
                            .constraints(vec![
                                Constraint::Fill(1),
                                Constraint::Max(50),
                                Constraint::Fill(1),
                            ])
                            .split(*rect)[1];
                        state.set_prev_area(centered.clone());
                        if let Some(previous) = self.previous_index() {
                            if previous < self.items.len() {
                                let item = self.items[previous].clone();
                                if state.prev_area.as_ref().unwrap().is_completed() {
                                    state.prev_old_item = Some(item.clone());
                                    NextPrevItem::new(item).render(area, buf);
                                } else {
                                    if let Some(old_item) = state.prev_old_item.clone() {
                                        NextPrevItem::new(old_item).render(area, buf);
                                    }
                                }
                            }
                        }
                    }
                }
                3 => {
                    if let Some(area) = state.get_focused_area() {
                        if let Some(focused) = self.focused_index() {
                            if focused < self.items.len() {
                                let item = self.items[focused].clone();
                                if state.focused_area.as_ref().unwrap().is_completed() {
                                    state.focused_old_item = Some(item.clone());
                                    FocusedItem::new(item).render(area, buf);
                                } else {
                                    if let Some(old_item) = state.focused_old_item.clone() {
                                        NextPrevItem::new(old_item).render(area, buf);
                                    }
                                }
                            }
                        }
                    } else {
                        let centered = Layout::default()
                            .direction(Direction::Horizontal)
                            .constraints(vec![
                                Constraint::Fill(1),
                                Constraint::Max(50),
                                Constraint::Fill(1),
                            ])
                            .split(*rect)[1];
                        state.set_focused_area(centered.clone());
                        if let Some(focused) = self.focused_index() {
                            if focused < self.items.len() {
                                let item = self.items[focused].clone();
                                if state.focused_area.as_ref().unwrap().is_completed() {
                                    state.focused_old_item = Some(item.clone());
                                    FocusedItem::new(item).render(area, buf);
                                } else {
                                    if let Some(old_item) = state.focused_old_item.clone() {
                                        NextPrevItem::new(old_item).render(area, buf);
                                    }
                                }
                            }
                        }
                    }
                }
                5 => {
                    if let Some(area) = state.get_next_area() {
                        if let Some(next) = self.next_index() {
                            if next < self.items.len() {
                                let item = self.items[next].clone();
                                if state.next_area.as_ref().unwrap().is_completed() {
                                    state.next_old_item = Some(item.clone());
                                    NextPrevItem::new(item).render(area, buf);
                                } else {
                                    if let Some(old_item) = state.next_old_item.clone() {
                                        NextPrevItem::new(old_item).render(area, buf);
                                    }
                                }
                            }
                        }
                    } else {
                        let centered = Layout::default()
                            .direction(Direction::Horizontal)
                            .constraints(vec![
                                Constraint::Fill(1),
                                Constraint::Max(50),
                                Constraint::Fill(1),
                            ])
                            .split(*rect)[1];
                        state.set_next_area(centered.clone());
                        if let Some(next) = self.next_index() {
                            if next < self.items.len() {
                                let item = self.items[next].clone();
                                if state.next_area.as_ref().unwrap().is_completed() {
                                    state.next_old_item = Some(item.clone());
                                    NextPrevItem::new(item).render(area, buf);
                                } else {
                                    if let Some(old_item) = state.next_old_item.clone() {
                                        NextPrevItem::new(old_item).render(area, buf);
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            });

        // if let Some(focused) = self.focused_index() {
        //     let centered = Layout::default()
        //         .direction(Direction::Horizontal)
        //         .constraints(vec![
        //             Constraint::Fill(1),
        //             Constraint::Max(50),
        //             Constraint::Fill(1),
        //         ])
        //         .split(layout[3])[1];
        //     state.set_focused_area(centered.clone());
        //     if focused < self.items.len() {
        //         let item = self.items[focused].clone();
        //         FocusedItem::new(item).render(centered, buf);
        //     }

        //     let centered = Layout::default()
        //         .direction(Direction::Horizontal)
        //         .constraints(vec![
        //             Constraint::Fill(1),
        //             Constraint::Max(50),
        //             Constraint::Fill(1),
        //         ])
        //         .split(layout[1])[1];
        //     state.set_prev_area(centered.clone());
        //     if let Some(previous) = self.previous_index() {
        //         if previous < self.items.len() {
        //             let item = &self.items[previous];
        //             let block = Block::default()
        //                 .borders(Borders::ALL)
        //                 .border_set(BorderType::Rounded.to_border_set())
        //                 .style(Style::default().dim());
        //             let inner = block.inner(centered);
        //             block.render(centered, buf);
        //             Widget::render(item.content.clone().centered(), inner, buf);
        //         }
        //     }

        //     let centered = Layout::default()
        //         .direction(Direction::Horizontal)
        //         .constraints(vec![
        //             Constraint::Fill(1),
        //             Constraint::Max(50),
        //             Constraint::Fill(1),
        //         ])
        //         .split(layout[5])[1];
        //     state.set_next_area(centered.clone());
        //     if let Some(next) = self.next_index() {
        //         if next < self.items.len() {
        //             let item = &self.items[next];
        //             let block = Block::default()
        //                 .borders(Borders::ALL)
        //                 .border_set(BorderType::Rounded.to_border_set())
        //                 .style(Style::default().dim());
        //             let inner = block.inner(centered);
        //             block.render(centered, buf);
        //             Widget::render(item.content.clone().centered(), inner, buf);
        //         }
        //     }
        // }
    }
}
