use std::fmt;
use tui_textarea::CursorMove;

#[derive(Debug, Clone, Copy)]
pub enum TextObject {
    Char,
    WordInner,
    WordAround,
    Line,
    ParagraphInner,
    ParagraphAround,
    Selection,
    To(CursorMove),
}

#[derive(Debug, Clone, Copy)]
pub enum TextObjectModifier {
    Inner,
    Around,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisualMode {
    Char,
    Line,
    // Block,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    Normal,
    Insert,
    Replace,
    Visual(VisualMode),
}

impl fmt::Display for EditorMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::Normal => write!(f, "NORMAL"),
            Self::Insert => write!(f, "INSERT"),
            Self::Replace => write!(f, "REPLACE"),
            Self::Visual(_) => write!(f, "VISUAL"),
        }
    }
}

mod actions;
mod composite;
mod editor;
mod handlers;
mod helpers;

pub use actions::{EditorAction, EditorActions, EditorPendingAction};
pub use composite::CompositeEditor;
pub use editor::Editor;
pub use handlers::{handle_input, handle_pending_action_input};
pub use helpers::{create_block, cursor_style, is_movement_key, match_movement_key};
