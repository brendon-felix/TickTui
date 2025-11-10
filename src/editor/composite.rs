use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Position, Rect},
    widgets::{Widget, WidgetRef},
};
use tui_textarea::CursorMove;

use crate::editor::EditorStyle;

use super::{Editor, EditorAction, EditorActions, EditorMode, EditorPendingAction};
#[allow(dead_code)]
pub struct CompositeEditor {
    editors: Vec<Editor>,
    active_index: Option<usize>,
    constraints: Vec<Constraint>,
    last_area: Option<Rect>,
}

#[allow(dead_code)]
impl CompositeEditor {
    pub fn new(editors: Vec<Editor>, constraints: Vec<Constraint>) -> Self {
        let active_index = if editors.is_empty() { None } else { Some(0) };
        let mut composite = Self {
            editors,
            active_index,
            constraints,
            last_area: None,
        };
        composite.set_active_editor(active_index);
        composite
    }

    pub fn set_active_editor(&mut self, index: Option<usize>) {
        self.active_index = index;
        self.editors.iter_mut().enumerate().for_each(|(i, editor)| {
            if Some(i) == index {
                editor.set_editor_style(super::EditorStyle::Active);
            } else {
                editor.set_editor_style(super::EditorStyle::Inactive);
            }
        });
    }

    pub fn get_active_editor(&mut self) -> Option<&mut Editor> {
        self.active_index
            .and_then(|index| self.editors.get_mut(index))
    }

    pub fn get_mode(&self) -> Option<EditorMode> {
        self.active_index
            .and_then(|index| self.editors.get(index))
            .map(|editor| editor.get_mode())
    }

    pub fn set_last_area(&mut self, area: Rect) {
        self.last_area = Some(area);
    }

    fn create_chunks(&self, area: Rect) -> Vec<Rect> {
        Layout::vertical(self.constraints.clone())
            .split(area)
            .to_vec()
    }

    pub fn on_click(&mut self, pos: Position) {
        if let Some(area) = self.last_area {
            let chunks = self.create_chunks(area);
            for (i, chunk) in chunks.iter().enumerate() {
                if chunk.contains(pos) {
                    self.set_active_editor(Some(i));
                    if let Some(editor) = self.get_active_editor() {
                        let local = Position::new(
                            pos.x.saturating_sub(chunk.x),
                            pos.y.saturating_sub(chunk.y),
                        );
                        editor.on_click(local);
                    }
                    break;
                }
            }
        }
    }

    pub fn is_cursor_at_line_start(&mut self) -> bool {
        if let Some(editor) = self.get_active_editor() {
            editor.is_cursor_at_line_start()
        } else {
            false
        }
    }

    pub fn style_all_inactive(&mut self) {
        self.editors.iter_mut().for_each(|editor| {
            editor.set_editor_style(EditorStyle::Inactive);
        });
    }

    pub fn restyle_active(&mut self) {
        if let Some(active_index) = self.active_index {
            self.editors.iter_mut().enumerate().for_each(|(i, editor)| {
                if i == active_index {
                    editor.set_editor_style(EditorStyle::Active);
                } else {
                    editor.set_editor_style(EditorStyle::Inactive);
                }
            });
        }
    }
}

impl EditorActions for CompositeEditor {
    fn execute_action(&mut self, action: EditorAction) {
        if let Some(active_index) = self.active_index {
            let num_editors = self.editors.len();
            let mut cursor_movement = None;
            if let Some(editor) = self.get_active_editor() {
                match action {
                    EditorAction::MoveCursor(CursorMove::Up) => match editor.get_cursor_pos() {
                        (row, _col) if row == 0 && active_index > 0 => {
                            cursor_movement =
                                Some((editor.get_desired_column(), CursorMove::Bottom));
                            self.set_active_editor(Some(active_index - 1));
                        }
                        _ => editor.execute_action(action),
                    },
                    EditorAction::MoveCursor(CursorMove::Down) => match editor.get_cursor_pos() {
                        (row, _col)
                            if row >= editor.get_lines().len().saturating_sub(1)
                                && active_index + 1 < num_editors =>
                        {
                            cursor_movement = Some((editor.get_desired_column(), CursorMove::Top));
                            self.set_active_editor(Some(active_index + 1));
                        }
                        _ => editor.execute_action(action),
                    },
                    EditorAction::MoveCursor(CursorMove::Top) if active_index <= 0 => {
                        cursor_movement = Some((editor.get_desired_column(), CursorMove::Top));
                    }
                    EditorAction::MoveCursor(CursorMove::Top) => {
                        cursor_movement = Some((editor.get_desired_column(), CursorMove::Top));
                        self.set_active_editor(Some(0));
                    }
                    EditorAction::MoveCursor(CursorMove::Bottom)
                        if active_index >= num_editors - 1 =>
                    {
                        cursor_movement = Some((editor.get_desired_column(), CursorMove::Bottom));
                    }
                    EditorAction::MoveCursor(CursorMove::Bottom) => {
                        cursor_movement = Some((editor.get_desired_column(), CursorMove::Bottom));
                        self.set_active_editor(Some(num_editors - 1));
                    }
                    _ => editor.execute_action(action),
                }
            };
            if let Some((col, movement)) = cursor_movement {
                if let Some(editor) = self.get_active_editor() {
                    editor.set_desired_column(col);
                    editor.execute_action(EditorAction::MoveCursor(movement));
                }
            }
        }
    }

    fn set_pending_action(&mut self, pending: Option<EditorPendingAction>) {
        if let Some(active_index) = self.active_index {
            if let Some(editor) = self.editors.get_mut(active_index) {
                editor.set_pending_action(pending);
            }
        }
    }

    fn get_pending_action(&mut self) -> Option<EditorPendingAction> {
        if let Some(active_index) = self.active_index {
            if let Some(editor) = self.editors.get_mut(active_index) {
                return editor.get_pending_action();
            }
        }
        None
    }
}

impl Widget for CompositeEditor {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = self.create_chunks(area);
        for (i, editor) in self.editors.into_iter().enumerate() {
            editor.render(chunks[i], buf);
        }
    }
}

impl WidgetRef for CompositeEditor {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let chunks = self.create_chunks(area);
        for (i, editor) in self.editors.iter().enumerate() {
            editor.render_ref(chunks[i], buf);
        }
    }
}
