use anyhow::Result;
// use anyhow::Result;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{Frame, layout::Rect};

use crate::ui::editor::CompositeEditorWidget;

// use crate::ui::tasklist::TaskList;

pub mod editor;
mod multiselect;
mod tasklist;

enum AppUIMode {
    Focus,
    Normal,
}

struct FocusModeUI {
    // tasks: Vec<Task>,
    // active_task: Option<usize>,
}
impl FocusModeUI {
    fn new() -> Self {
        Self {
            // tasks: Vec::new(),
            // active_task: None,
        }
    }
}

struct TaskEditor {
    // editor: CompositeEditor,
    // editor_state: CompositeEditorState,
}

struct NormalModeUI {
    // task_list: TaskList,
    // task_editor: TaskEditor,
}
impl NormalModeUI {
    fn new() -> Self {
        Self {
            // task_list: TaskList::new(),
            // task_editor: TaskEditor::new(),
        }
    }
}

pub struct AppUI {
    // task_list: TaskList,
    // composite_editor: CompositeEditor,
    // current_area: Option<Rect>,
    // active_widget: Option<ActiveWidget>,
    mode: AppUIMode,
    focus_ui: FocusModeUI,
    normal_ui: NormalModeUI,
}

impl AppUI {
    pub fn new() -> Self {
        Self {
            mode: AppUIMode::Normal,
            focus_ui: FocusModeUI::new(),
            normal_ui: NormalModeUI::new(),
        }
    }

    pub fn is_quittable(&self) -> bool {
        true
    }

    pub fn handle_key_event(&mut self, _key_event: KeyEvent) {
        let _ = ();
    }

    pub fn handle_mouse_event(&mut self, _mouse_event: MouseEvent) {
        let _ = ();
    }

    pub fn draw(&mut self, f: &mut Frame, area: Rect) -> Result<()> {
        let _ = (f, area);
        Ok(())
    }
}
