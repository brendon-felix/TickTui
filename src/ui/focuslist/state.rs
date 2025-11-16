use std::time::Instant;

use ratatui::layout::Rect;

use crate::ui::{
    animate::{AnimatedArea, Animation},
    focuslist::FocusListItem,
};

#[derive(Default)]
pub struct FocusListState<'a> {
    pub prev_area: Option<AnimatedArea>,
    pub prev_old_item: Option<FocusListItem<'a>>,
    pub focused_area: Option<AnimatedArea>,
    pub focused_old_item: Option<FocusListItem<'a>>,
    pub next_area: Option<AnimatedArea>,
    pub next_old_item: Option<FocusListItem<'a>>,
    // last_frame: Option<Instant>,
    // prev_buf: Option<Buffer>,
    // focused_buf: Option<Buffer>,
    // next_buf: Option<Buffer>,
}

impl FocusListState<'_> {
    pub fn update_animations(&mut self) {
        if let Some(animated_area) = &mut self.prev_area {
            animated_area.update();
        }
        if let Some(animated_area) = &mut self.focused_area {
            animated_area.update();
        }
        if let Some(animated_area) = &mut self.next_area {
            animated_area.update();
        }
    }

    pub fn set_prev_area(&mut self, area: Rect) {
        self.prev_area = Some(AnimatedArea::new(area));
    }

    pub fn start_prev_animation(&mut self, animation: Animation) {
        if let Some(animated_area) = &mut self.prev_area {
            animated_area.start_animation(animation);
        }
    }

    pub fn get_prev_area(&mut self) -> Option<Rect> {
        if let Some(animated_area) = &self.prev_area {
            Some(animated_area.get_area())
        } else {
            None
        }
    }

    pub fn set_focused_area(&mut self, area: Rect) {
        self.focused_area = Some(AnimatedArea::new(area));
    }

    pub fn start_focused_animation(&mut self, animation: Animation) {
        if let Some(animated_area) = &mut self.focused_area {
            animated_area.start_animation(animation);
        }
    }

    pub fn get_focused_area(&mut self) -> Option<Rect> {
        if let Some(animated_area) = &self.focused_area {
            Some(animated_area.get_area())
        } else {
            None
        }
    }

    pub fn set_next_area(&mut self, area: Rect) {
        self.next_area = Some(AnimatedArea::new(area));
    }

    pub fn start_next_animation(&mut self, animation: Animation) {
        if let Some(animated_area) = &mut self.next_area {
            animated_area.start_animation(animation);
        }
    }

    pub fn get_next_area(&mut self) -> Option<Rect> {
        if let Some(animated_area) = &self.next_area {
            Some(animated_area.get_area())
        } else {
            None
        }
    }

    // pub fn prev_area(&mut self, area: &mut Rect) {
    //     if let Some(animated_area) = &self.prev_area {
    //         *area = animated_area.get_area();
    //     } else {
    //         self.prev_area = Some(AnimatedArea::new(*area));
    //     }
    // }

    // pub fn focused_area(&mut self, area: &mut Rect) {
    //     if let Some(animated_area) = &self.prev_area {
    //         *area = animated_area.get_area();
    //     } else {
    //         self.focused_area = Some(AnimatedArea::new(*area));
    //     }
    // }

    // pub fn next_area(&mut self, area: &mut Rect) {
    //     if let Some(animated_area) = &self.prev_area {
    //         *area = animated_area.get_area();
    //     } else {
    //         self.next_area = Some(AnimatedArea::new(*area));
    //     }
    // }

    // pub fn set_prev_buf(&mut self, buf: Buffer) {
    //     self.prev_buf = Some(buf);
    // }
    // pub fn set_focused_buf(&mut self, buf: Buffer) {
    //     self.focused_buf = Some(buf);
    // }
    // pub fn set_next_buf(&mut self, buf: Buffer) {
    //     self.next_buf = Some(buf);
    // }
    // pub fn get_sub_bufs_owned(&self) -> (Option<Buffer>, Option<Buffer>, Option<Buffer>) {
    //     (
    //         self.prev_buf.clone(),
    //         self.focused_buf.clone(),
    //         self.next_buf.clone(),
    //     )
    // }
}
