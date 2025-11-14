use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Text},
    widgets::{Block, HighlightSpacing, StatefulWidget, Widget, WidgetRef},
};

pub use self::state::MultiSelectListState;

const ITEM_HEIGHT: u16 = 3;

mod state;

// fn popup_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
//     let popup_layout = Layout::default()
//         .direction(ratatui::layout::Direction::Vertical)
//         .constraints([
//             Constraint::Percentage((100 - percent_y) / 2),
//             Constraint::Percentage(percent_y),
//             Constraint::Percentage((100 - percent_y) / 2),
//         ])
//         .split(r);

//     Layout::default()
//         .direction(ratatui::layout::Direction::Horizontal)
//         .constraints([
//             Constraint::Percentage((100 - percent_x) / 2),
//             Constraint::Percentage(percent_x),
//             Constraint::Percentage((100 - percent_x) / 2),
//         ])
//         .split(popup_layout[1])[1]
// }

pub struct MultiSelectListItem<'a> {
    content: Text<'a>,
    style: Style,
}

impl<'a> MultiSelectListItem<'a> {
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

impl<'a, T> From<T> for MultiSelectListItem<'a>
where
    T: Into<Text<'a>>,
{
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

// #[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
#[derive(Default)]
pub struct MultiSelectList<'a> {
    block: Option<Block<'a>>,
    items: Vec<MultiSelectListItem<'a>>,
    style: Style,
    highlight_style: Style,
    highlight_symbol: Option<Line<'a>>,
    repeat_highlight_symbol: bool,
    highlight_spacing: HighlightSpacing,
    scroll_padding: usize,
}

impl<'a> MultiSelectList<'a> {
    pub fn new<T>(items: T) -> Self
    where
        T: IntoIterator,
        T::Item: Into<MultiSelectListItem<'a>>,
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

    pub fn with_highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = style;
        self
    }

    pub fn set_style(&mut self, style: Style) {
        self.style = style;
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl Widget for MultiSelectList<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Widget::render(&self, area, buf);
    }
}

impl WidgetRef for MultiSelectList<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let mut state = MultiSelectListState::default();
        StatefulWidget::render(self, area, buf, &mut state);
    }
}

impl StatefulWidget for MultiSelectList<'_> {
    type State = MultiSelectListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        StatefulWidget::render(&self, area, buf, state);
    }
}

impl StatefulWidget for &MultiSelectList<'_> {
    type State = MultiSelectListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        buf.set_style(area, self.style);
        if let Some(block) = self.block.as_ref() {
            block.render(area, buf);
        }
        let list_area = if let Some(block) = self.block.as_ref() {
            block.inner(area)
        } else {
            area
        };

        if list_area.is_empty() {
            return;
        }

        if self.items.is_empty() {
            state.select(None);
            return;
        }

        // If the selected index is out of bounds, set it to the last item
        if state.selected.is_some_and(|s| s >= self.items.len()) {
            state.select(Some(self.items.len().saturating_sub(1)));
        }

        let list_height = list_area.height as usize;

        let (first_visible_index, last_visible_index) =
            self.get_items_bounds(state.selected, state.offset, list_height);

        // Important: this changes the state's offset to be the beginning of the now viewable items
        state.offset = first_visible_index;

        // Get our set highlighted symbol (if one was set)
        let default_highlight_symbol = Line::default();
        let highlight_symbol = self
            .highlight_symbol
            .as_ref()
            .unwrap_or(&default_highlight_symbol);
        let highlight_symbol_width = highlight_symbol.width() as u16;
        let empty_symbol = " ".repeat(highlight_symbol_width as usize);
        // let empty_symbol = empty_symbol.to_line();
        let empty_symbol = Line::from(empty_symbol);

        let mut current_height = 0;
        // let selection_spacing = self.highlight_spacing.should_add(state.selected.is_some());
        let selection_spacing = match self.highlight_spacing {
            HighlightSpacing::Always => true,
            HighlightSpacing::WhenSelected => state.selected.is_some(),
            HighlightSpacing::Never => false,
        };
        for (i, item) in self
            .items
            .iter()
            .enumerate()
            .skip(state.offset)
            .take(last_visible_index - first_visible_index)
        {
            let pos = (list_area.left(), list_area.top() + current_height);
            current_height += ITEM_HEIGHT;
            let (x, y) = pos;

            let row_area = Rect::new(x, y, list_area.width, ITEM_HEIGHT);

            let item_style = self.style.patch(item.style);
            buf.set_style(row_area, item_style);

            // let is_selected = state.selected == Some(i)
            //     || (state
            //         .visual_start
            //         .as_ref()
            //         .is_some_and(|start| start <= &i));

            let is_selected = if let Some(curr) = state.selected {
                if curr == i {
                    true
                } else if let Some(visual_start) = state.visual_start {
                    (visual_start <= i && i <= curr) || (curr <= i && i <= visual_start)
                } else {
                    false
                }
            } else {
                false
            };

            let item_area = if selection_spacing {
                Rect {
                    x: row_area.x + highlight_symbol_width,
                    width: row_area.width.saturating_sub(highlight_symbol_width),
                    ..row_area
                }
            } else {
                row_area
            };
            Widget::render(&item.content, item_area, buf);

            if is_selected {
                buf.set_style(row_area, self.highlight_style);
            }
            if selection_spacing {
                for j in 0..item.content.height() {
                    // if the item is selected, we need to display the highlight symbol:
                    // - either for the first line of the item only,
                    // - or for each line of the item if the appropriate option is set
                    let line = if is_selected && (j == 0 || self.repeat_highlight_symbol) {
                        highlight_symbol
                    } else {
                        &empty_symbol
                    };
                    let highlight_area = Rect::new(x, y + j as u16, highlight_symbol_width, 1);
                    line.render(highlight_area, buf);
                }
            }
        }
    }
}

impl MultiSelectList<'_> {
    /// Given an offset, calculate which items can fit in a given area
    fn get_items_bounds(
        &self,
        selected: Option<usize>,
        offset: usize,
        max_height: usize,
    ) -> (usize, usize) {
        let offset = offset.min(self.items.len().saturating_sub(1));

        // Note: visible here implies visible in the given area
        let mut first_visible_index = offset;
        let mut last_visible_index = offset;

        // Current height of all items in the list to render, beginning at the offset
        let mut height_from_offset = 0;

        // Calculate the last visible index and total height of the items
        // that will fit in the available space
        for _item in self.items.iter().skip(offset) {
            if height_from_offset + ITEM_HEIGHT > (max_height as u16) {
                break;
            }

            height_from_offset += ITEM_HEIGHT;

            last_visible_index += 1;
        }

        // Get the selected index and apply scroll_padding to it, but still honor the offset if
        // nothing is selected. This allows for the list to stay at a position after select()ing
        // None.
        let index_to_display = self
            .apply_scroll_padding_to_selected_index(
                selected,
                max_height,
                first_visible_index,
                last_visible_index,
            )
            .unwrap_or(offset);

        // Recall that last_visible_index is the index of what we
        // can render up to in the given space after the offset
        // If we have an item selected that is out of the viewable area (or
        // the offset is still set), we still need to show this item
        while index_to_display >= last_visible_index {
            height_from_offset = height_from_offset.saturating_add(ITEM_HEIGHT);

            last_visible_index += 1;

            // Now we need to hide previous items since we didn't have space
            // for the selected/offset item
            while height_from_offset > ITEM_HEIGHT {
                height_from_offset = height_from_offset.saturating_sub(ITEM_HEIGHT);

                // Remove this item to view by starting at the next item index
                first_visible_index += 1;
            }
        }

        // Here we're doing something similar to what we just did above
        // If the selected item index is not in the viewable area, let's try to show the item
        while index_to_display < first_visible_index {
            first_visible_index -= 1;

            height_from_offset = height_from_offset.saturating_add(ITEM_HEIGHT);

            // Don't show an item if it is beyond our viewable height
            while height_from_offset > (max_height as u16) {
                last_visible_index -= 1;

                height_from_offset = height_from_offset.saturating_sub(ITEM_HEIGHT);
            }
        }

        (first_visible_index, last_visible_index)
    }

    /// Applies scroll padding to the selected index, reducing the padding value to keep the
    /// selected item on screen even with items of inconsistent sizes
    ///
    /// This function is sensitive to how the bounds checking function handles item height
    fn apply_scroll_padding_to_selected_index(
        &self,
        selected: Option<usize>,
        max_height: usize,
        first_visible_index: usize,
        last_visible_index: usize,
    ) -> Option<usize> {
        let last_valid_index = self.items.len().saturating_sub(1);
        let selected = selected?.min(last_valid_index);

        // The bellow loop handles situations where the list item sizes may not be consistent,
        // where the offset would have excluded some items that we want to include, or could
        // cause the offset value to be set to an inconsistent value each time we render.
        // The padding value will be reduced in case any of these issues would occur
        let mut scroll_padding = self.scroll_padding;
        while scroll_padding > 0 {
            let mut height_around_selected = 0;
            for _index in selected.saturating_sub(scroll_padding)
                ..=selected
                    .saturating_add(scroll_padding)
                    .min(last_valid_index)
            {
                height_around_selected += ITEM_HEIGHT;
            }
            if height_around_selected <= (max_height as u16) {
                break;
            }
            scroll_padding -= 1;
        }

        Some(
            if (selected + scroll_padding).min(last_valid_index) >= last_visible_index {
                selected + scroll_padding
            } else if selected.saturating_sub(scroll_padding) < first_visible_index {
                selected.saturating_sub(scroll_padding)
            } else {
                selected
            }
            .min(last_valid_index),
        )
    }
}
// impl Styled for MultiSelectList<'_> {
//     type Item = Self;

//     fn style(&self) -> Style {
//         self.style
//     }

//     fn set_style<S: Into<Style>>(self, style: S) -> Self::Item {
//         self.style(style)
//     }
// }

impl<'a, Item> FromIterator<Item> for MultiSelectList<'a>
where
    Item: Into<MultiSelectListItem<'a>>,
{
    fn from_iter<Iter: IntoIterator<Item = Item>>(iter: Iter) -> Self {
        Self::new(iter)
    }
}
