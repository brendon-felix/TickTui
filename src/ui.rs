use anyhow::Result;
use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Position, Rect},
    widgets::{Block, Paragraph},
};
use tui_textarea::Input;
// use tui_text::{EditorEventHandler, EditorMode, EditorState, EditorTheme, EditorView};

// enum ActiveWidget {
//     MainEditor,
//     CompositeEditor,
//     // TaskList,
//     // Popup,
// }

const EDITOR_CONTENT: &str = r#"fn factorial(n: u32) -> u32 {
    if n == 0 {
        1
    } else {
        n * factorial(n - 1)
    }
}

fn main() {
    let result = factorial(5);
    println!("The factorial of 5 is: {}", result);
}"#;

use crate::editor::{
    CompositeEditor, Editor, EditorAction, EditorActions, EditorMode, handle_input,
    handle_pending_action_input,
};

pub struct UserInterface {
    content: String,
    editor: Editor,
    composite_editor: CompositeEditor,
    current_area: Option<Rect>,
}

impl UserInterface {
    pub fn new() -> Self {
        #[rustfmt::skip]
        let composite_editors = vec![
            Editor::default()
                .with_title("Editor 1")
                .with_single_line(true)
                .with_placeholder("Single line editor"),
            Editor::default()
                .with_title("Editor 2")
                .with_content(EDITOR_CONTENT),
            Editor::default()
                .with_title("Editor 3")
                .with_content(EDITOR_CONTENT),
        ];
        let constraints = vec![
            Constraint::Length(3),
            Constraint::Min(3),
            Constraint::Min(3),
        ];
        let mut composite_editor = CompositeEditor::new(composite_editors, constraints);
        composite_editor.set_active_editor(Some(0));
        let editor = Editor::default()
            .with_title("Main Editor")
            .with_content(EDITOR_CONTENT)
            .with_block(
                Block::default()
                    .title("Main Editor")
                    .borders(ratatui::widgets::Borders::ALL),
            );

        UserInterface {
            content: String::new(),
            editor,
            composite_editor,
            current_area: None,
        }
    }

    pub fn get_main_editor_mode(&self) -> EditorMode {
        self.editor.get_mode()
    }

    pub fn get_composite_editor_mode(&self) -> Option<EditorMode> {
        self.composite_editor.get_mode()
    }

    pub fn handle_action(&mut self, action: EditorAction) {
        let action_clone = action.clone();
        match action_clone {
            EditorAction::ApplyInput(_) => {}
            EditorAction::MoveCursor(_) => {}
            _ => {
                self.content
                    .push_str(&format!("Handling action: {:?}\n", action_clone));
            }
        }
        self.composite_editor.execute_action(action);
    }

    pub fn handle_editor_event(&mut self, event: KeyEvent) {
        let input: Input = event.into();
        if let Some(mode) = self.composite_editor.get_mode() {
            let action_opt =
                if let Some(pending_action) = self.composite_editor.get_pending_action() {
                    match handle_pending_action_input(input, pending_action) {
                        Some(action) => Some(action),
                        None => {
                            self.composite_editor.set_pending_action(None);
                            None
                        }
                    }
                } else {
                    handle_input(input, mode)
                };
            match action_opt {
                Some(action) => self.handle_action(action),
                None => {}
            }
        }
    }

    pub fn handle_mouse_click(&mut self, pos: Position) {
        if let Some(area) = &self.current_area {
            let main_chunks = main_chunks(area.clone());
            if main_chunks[0].contains(pos) {
                let content_chunks = content_chunks(main_chunks[0]);
                if content_chunks[1].contains(pos) {
                    let local_pos = Position {
                        x: pos.x.saturating_sub(content_chunks[1].x),
                        y: pos.y.saturating_sub(content_chunks[1].y),
                    };
                    self.content
                        .push_str(&format!("Main editor click: {:?}\n", pos));
                    self.editor.on_click(local_pos);
                } else if content_chunks[2].contains(pos) {
                    self.content
                        .push_str(&format!("Composite editor click: {:?}\n", pos));
                    self.composite_editor.on_click(pos);
                }
            }
        }
        // let content_chunks = content_chunks(main_chunks[0]);
        // self.mouse_clicked = Some((x, y));
        // self.composite_editor.set_active_editor_at_position(x, y);
    }

    pub fn draw(&mut self, f: &mut Frame, area: Rect) -> Result<()> {
        let main_chunks = main_chunks(area);
        self.current_area = Some(area.clone());
        let content_chunks = content_chunks(main_chunks[0]);
        // self.render_header(f, main_chunks[0]);
        self.render_content(f, content_chunks)?;
        self.render_footer(f, main_chunks[1])?;
        Ok(())
    }

    // fn render_header(&self, f: &mut Frame, area: Rect) {
    //     let style = Style::default().fg(Color::Yellow);
    //     let title = "TickTUI";
    //     let header = Paragraph::new(title)
    //         .style(style)
    //         .alignment(ratatui::layout::Alignment::Center)
    //         .block(Block::default());
    //     f.render_widget(header, area);
    // }

    fn render_content(&mut self, f: &mut Frame, areas: Vec<Rect>) -> Result<()> {
        let p = Paragraph::new(self.content.as_str())
            .block(
                Block::default()
                    .title("Content")
                    .borders(ratatui::widgets::Borders::ALL),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });
        f.render_widget(p, areas[0]);
        f.render_widget(&self.editor, areas[1]);
        self.composite_editor.set_last_area(areas[2].clone());
        f.render_widget(&self.composite_editor, areas[2]);
        Ok(())
    }

    fn render_footer(&self, f: &mut Frame, area: Rect) -> Result<()> {
        let footer_text = "?: Help | q: Quit";
        let footer = Paragraph::new(footer_text).alignment(Alignment::Center);
        // .block(Block::default().borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT));

        f.render_widget(footer, area);
        Ok(())
    }
}

fn main_chunks(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(1)])
        .split(area)
        .to_vec()
}

fn content_chunks(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(40),
            Constraint::Percentage(40),
        ])
        .split(area)
        .to_vec()
}

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
