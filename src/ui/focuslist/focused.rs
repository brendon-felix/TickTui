use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style, Stylize},
    widgets::{Block, BorderType, Borders, Widget},
};

use crate::ui::focuslist::FocusListItem;

pub struct FocusedItem<'a> {
    item: FocusListItem<'a>,
}

impl<'a> FocusedItem<'a> {
    pub fn new(item: FocusListItem<'a>) -> Self {
        Self { item }
    }
}

impl<'a> Widget for FocusedItem<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_set(BorderType::Rounded.to_border_set())
            .style(self.item.style.add_modifier(Modifier::BOLD));
        let inner = block.inner(area);
        block.render(area, buf);
        Widget::render(self.item.content.clone().centered(), inner, buf);
    }
}

//
//
//

pub struct NextPrevItem<'a> {
    item: FocusListItem<'a>,
}

impl<'a> NextPrevItem<'a> {
    pub fn new(item: FocusListItem<'a>) -> Self {
        Self { item }
    }
}

impl<'a> Widget for NextPrevItem<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_set(BorderType::Rounded.to_border_set())
            .style(Style::default().dim());
        let inner = block.inner(area);
        block.render(area, buf);
        Widget::render(self.item.content.clone().centered(), inner, buf);
    }
}
