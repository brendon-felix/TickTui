use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
};
use tui_textarea::{CursorMove, Input};

use crate::ui::{
    composite::{CompositeEditor, CompositeEditorState},
    editor::{
        Editor, EditorMode,
        actions::{EditorAction, EditorActions},
        handlers::{handle_input, handle_pending_action_input},
    },
    tasklist::TaskItem,
};

// const SAMPLE_DESCRIPTION: &str = r#"This is a description.
// You can write multiple lines here,
// or edit the content as needed.

// This is another paragraph to demonstrate the editor functionality.
// Next we have a line that is really long ..."#;

pub struct TaskEditor {
    editor: CompositeEditor,
    editor_state: CompositeEditorState,
}

impl TaskEditor {
    pub fn new() -> Self {
        let editors = vec![
            Editor::default().with_single_line(true).with_title("Title"),
            Editor::default().with_title("Description"),
        ];
        let editor_state = CompositeEditorState::new(editors.len());
        let editor = CompositeEditor::new(editors)
            .with_constraints(vec![Constraint::Length(3), Constraint::Min(3)]);
        Self {
            editor,
            editor_state,
        }
    }

    pub fn deactivate(&mut self) {
        self.editor.set_pending_action(None);
        self.editor.set_active_editor(None);
    }

    pub fn activate(&mut self) {
        self.editor.set_active_editor(Some(0));
    }

    pub fn set_title_content(&mut self, title: &str) {
        self.editor.editors[0].set_content(title);
    }

    pub fn set_description_content(&mut self, title: &str) {
        self.editor.editors[1].set_content(title);
    }

    pub fn load_task(&mut self, task: &TaskItem) {
        self.set_title_content(task.get_name());
        self.set_description_content(task.get_description());
    }

    pub fn is_in_insert_mode(&self) -> bool {
        if let Some(mode) = self.editor.get_mode() {
            mode == EditorMode::Insert
        } else {
            false
        }
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        let input: Input = key_event.into();
        if let Some(mode) = self.editor.get_mode() {
            let action_opt = if let Some(pending_action) = self.editor.get_pending_action() {
                match handle_pending_action_input(input, pending_action) {
                    Some(action) => Some(action),
                    None => {
                        self.editor.set_pending_action(None);
                        None
                    }
                }
            } else {
                handle_input(input, mode)
            };
            match action_opt {
                Some(action) => match action {
                    EditorAction::ApplyInput(_) => self.editor.execute_action(action),
                    EditorAction::MoveCursor(mvmt) => match mvmt {
                        CursorMove::Left if self.editor.is_cursor_at_line_start() => {}
                        _ => self.editor.execute_action(action),
                    },
                    _ => self.editor.execute_action(action),
                },
                None => {}
            }
        }
    }

    pub fn draw(&mut self, f: &mut Frame, area: Rect) {
        f.render_stateful_widget(&mut self.editor, area, &mut self.editor_state);
    }
}
