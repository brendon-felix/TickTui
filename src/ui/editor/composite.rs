use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Position, Rect},
    widgets::{StatefulWidget, Widget, WidgetRef},
};
use tui_textarea::CursorMove;

// use crate::ui::{AppWidget, WidgetStyle};

use super::{EditorAction, EditorActions, EditorMode, EditorPendingAction, EditorWidget};
#[allow(dead_code)]
pub struct CompositeEditorWidget {
    editors: Vec<EditorWidget>,
    active_index: Option<usize>,
    constraints: Vec<Constraint>,
    last_area_pos: Option<Position>,
}

pub struct CompositeEditorWidgetState {
    position: Position,
    sub_positions: Vec<Position>,
}
impl CompositeEditorWidgetState {
    pub fn new(num_editors: usize) -> Self {
        Self {
            position: Position::default(),
            sub_positions: vec![Position::default(); num_editors],
        }
    }

    pub fn set_position(&mut self, position: Position) {
        self.position = position;
    }

    pub fn set_sub_positions(&mut self, positions: Vec<Position>) {
        self.sub_positions = positions;
    }
}

#[allow(dead_code)]
impl CompositeEditorWidget {
    pub fn new(editors: Vec<EditorWidget>, constraints: Vec<Constraint>) -> Self {
        let active_index = if editors.is_empty() { None } else { Some(0) };
        let mut composite = Self {
            editors,
            active_index,
            constraints,
            last_area_pos: None,
        };
        composite.set_active_editor(active_index);
        composite
    }

    pub fn set_active_editor(&mut self, index: Option<usize>) {
        self.active_index = index;
        // self.editors.iter_mut().enumerate().for_each(|(i, editor)| {
        //     if Some(i) == index {
        //         editor.set_widget_style(WidgetStyle::Active);
        //     } else {
        //         editor.set_widget_style(WidgetStyle::Inactive);
        //     }
        // });
    }

    pub fn get_active_editor(&mut self) -> Option<&mut EditorWidget> {
        self.active_index
            .and_then(|index| self.editors.get_mut(index))
    }

    pub fn get_mode(&self) -> Option<EditorMode> {
        self.active_index
            .and_then(|index| self.editors.get(index))
            .map(|editor| editor.get_mode())
    }

    pub fn create_chunks(&self, area: Rect) -> Vec<Rect> {
        Layout::vertical(self.constraints.clone())
            .split(area)
            .to_vec()
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
            // editor.set_widget_style(WidgetStyle::Inactive);
        });
    }

    fn set_sub_positions(&mut self, positions: Vec<Position>) {
        self.editors
            .iter_mut()
            .zip(positions.into_iter())
            .for_each(|(editor, pos)| {
                // editor.set_last_area_pos(pos);
            });
    }
}

impl EditorActions for CompositeEditorWidget {
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

// impl AppWidget for CompositeEditorWidget {
//     fn set_widget_style(&mut self, style: WidgetStyle) {
//         match style {
//             WidgetStyle::Active => {
//                 if let Some(active_index) = self.active_index {
//                     self.editors.iter_mut().enumerate().for_each(|(i, editor)| {
//                         if i == active_index {
//                             editor.set_widget_style(WidgetStyle::Active);
//                         } else {
//                             editor.set_widget_style(WidgetStyle::Inactive);
//                         }
//                     });
//                 }
//             }
//             WidgetStyle::Inactive => {
//                 self.editors.iter_mut().for_each(|editor| {
//                     editor.set_widget_style(WidgetStyle::Inactive);
//                 });
//             }
//         }
//     }

//     fn on_click(&mut self, pos: Position) {
//         if let Some(area_pos) = self.last_area_pos {
//             // let chunks = self.create_chunks(area);
//             // for (i, chunk) in chunks.iter().enumerate() {
//             //     if chunk.contains(pos) {
//             //         self.set_active_editor(Some(i));
//             //         if let Some(editor) = self.get_active_editor() {
//             //             editor.set_last_area_pos(chunk.as_position());
//             //             editor.on_click(pos);
//             //         }
//             //         break;
//             //     }
//             // }
//         }
//     }

//     fn set_last_area_pos(&mut self, area_pos: Position) {
//         self.last_area_pos = Some(area_pos);
//         // let chunks = self.create_chunks(area);
//         // for chunk in chunks.iter() {
//         //     if let Some(editor) = self.get_active_editor() {
//         //         editor.set_last_area(chunk.clone());
//         //     }
//         // }
//     }
// }

impl Widget for CompositeEditorWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = self.create_chunks(area);
        for (i, editor) in self.editors.into_iter().enumerate() {
            editor.render(chunks[i], buf);
        }
    }
}

impl WidgetRef for CompositeEditorWidget {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let chunks = self.create_chunks(area);
        for (i, editor) in self.editors.iter().enumerate() {
            editor.render_ref(chunks[i], buf);
        }
    }
}

// impl StatefulWidget for CompositeEditorWidget {
//     type State = CompositeEditorWidgetState;

//     fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
//         let chunks = self.create_chunks(area);
//         state.set_sub_positions(chunks.iter().map(|chunk| chunk.as_position()).collect());
//         for (i, editor) in self.editors.into_iter().enumerate() {
//             editor.render(chunks[i], buf);
//         }
//     }
// }
