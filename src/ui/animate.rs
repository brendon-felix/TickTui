use std::time::{Duration, Instant};

use ratatui::layout::Rect;
use tachyonfx::fx::Direction;

#[derive(Debug, Clone)]
pub enum AnimationDirection {
    Left,
    Right,
    Up,
    Down,
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone)]
pub enum AnimationType {
    Resize {
        dir: AnimationDirection,
        amount: i32,
    },
    Translate {
        x: i32,
        y: i32,
    },
    Composite(Vec<AnimationType>),
}

#[derive(Debug, Clone)]
pub struct Animation {
    anim_type: AnimationType,
    start_instant: Instant,
    duration: Duration,
}
impl Animation {
    pub fn new(anim_type: AnimationType, duration: Duration) -> Self {
        Self {
            anim_type,
            start_instant: Instant::now(),
            duration,
        }
    }
}

// pub trait AnimatableWidget {
//     fn start_animation(&mut self, duration: Duration);
//     fn update_animation(&mut self, now: Instant);
//     fn is_animating(&self) -> bool;
// }

#[derive(Debug)]
pub struct AnimatedArea {
    area: Rect,
    initial_area: Rect,
    starting_area: Rect,
    animation: Option<Animation>,
}

impl AnimatedArea {
    pub fn new(area: Rect) -> Self {
        Self {
            area,
            initial_area: area,
            starting_area: area,
            animation: None,
        }
    }

    pub fn start_animation(&mut self, animation: Animation) {
        // self.initial_area = self.area;
        self.starting_area = self.area;
        self.animation = Some(animation);
    }

    pub fn is_completed(&self) -> bool {
        self.animation.is_none()
    }

    pub fn reset_to_initial(&mut self) {
        self.area = self.initial_area;
        self.animation = None;
    }

    pub fn reset_to_start(&mut self) {
        self.area = self.starting_area;
        self.animation = None;
    }

    pub fn update(&mut self) {
        // let mut complete = false;
        // if let Some(animation) = &self.animation {
        //     let now = Instant::now();
        //     let elapsed = now.duration_since(animation.start_instant);
        //     if elapsed >= animation.duration {
        //         complete = true;
        //     } else {
        //         let progress = elapsed.as_secs_f32() / animation.duration.as_secs_f32();
        //         match &animation.anim_type {
        //             AnimationType::Resize { dir, amount } => {
        //                 let total_change = (*amount as f32 * progress).round() as i32;
        //                 match dir {
        //                     AnimationDirection::Left => {
        //                         let x_before = self.area.x;
        //                         self.area.x = (self.initial_area.x as i32)
        //                             .saturating_sub(total_change)
        //                             as u16;
        //                         self.area.width = (self.initial_area.width as i32
        //                             + (self.area.x - x_before) as i32)
        //                             .max(0)
        //                             as u16;
        //                     }
        //                     AnimationDirection::Right => {
        //                         self.area.width =
        //                             (self.initial_area.width as i32 + total_change) as u16;
        //                     }
        //                     AnimationDirection::Up => {
        //                         self.area.y = (self.initial_area.y as i32 - total_change) as u16;
        //                         self.area.height =
        //                             (self.initial_area.height as i32 + total_change).max(0) as u16;
        //                     }
        //                     AnimationDirection::Down => {
        //                         self.area.height =
        //                             (self.initial_area.height as i32 + total_change) as u16;
        //                     }
        //                     AnimationDirection::Horizontal => {
        //                         // self.area.width =
        //                         //     (self.initial_area.width as i32 + total_change) as u16;
        //                         self.area.x =
        //                             (self.initial_area.x as i32 - total_change / 2) as u16;
        //                         self.area.width =
        //                             (self.initial_area.width as i32 + total_change) as u16;
        //                     }
        //                     AnimationDirection::Vertical => {
        //                         self.area.y =
        //                             (self.initial_area.y as i32 - total_change / 2) as u16;
        //                         self.area.height =
        //                             (self.initial_area.height as i32 + total_change) as u16;
        //                     }
        //                 }
        //             }
        //             AnimationType::Translate { x, y } => {
        //                 let total_change_x = (*x as f32 * progress).round() as i32;
        //                 let total_change_y = (*y as f32 * progress).round() as i32;
        //                 self.area.x = (self.initial_area.x as i32 + total_change_x) as u16;
        //                 self.area.y = (self.initial_area.y as i32 + total_change_y) as u16;
        //             }
        //             AnimationType::Composite(types) => {}
        //         }
        //     }
        // }
        // if complete {
        //     self.animation = None;
        //     self.reset_to_initial();
        // }
        self.animation = None;
    }

    pub fn get_area(&self) -> Rect {
        self.area
    }

    pub fn current_animation(&self) -> Option<&Animation> {
        self.animation.as_ref()
    }
}
