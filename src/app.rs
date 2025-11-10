use crate::{
    editor::EditorMode,
    term::{self, TerminalInterface},
    ui::UserInterface,
};

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Position;
// use std::sync::Arc;
// use ticks::TickTick;
use tokio::sync::mpsc::{self, UnboundedSender};
// use tui_text::EditorMode;

enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    Click(Position),
    Quit,
    // Error(String),
    // RefreshTasks,
    // Interact(UserInteraction),
}

pub struct TickTui {
    // client: Arc<TickTick>,
    ti: TerminalInterface,
    ui: UserInterface,
    quitting: bool,
}

impl TickTui {
    // pub fn new(client: Arc<TickTick>) -> Result<Self> {
    //     let ti = TerminalInterface::new()?;
    //     let ui = UserInterface::new();
    //     let quitting = false;
    //     Ok(Self {
    //         client,
    //         ti,
    //         ui,
    //         quitting,
    //     })
    // }

    pub fn new() -> Result<Self> {
        let ti = TerminalInterface::new()?;
        let ui = UserInterface::new();
        let quitting = false;
        Ok(Self { ti, ui, quitting })
    }

    pub async fn run(&mut self) -> Result<()> {
        let (tx, mut rx) = mpsc::unbounded_channel();
        self.ti.enter()?;

        loop {
            if let Some(event) = self.ti.next().await {
                self.handle_event(event, &tx)?;
            }

            while let Ok(action) = rx.try_recv() {
                self.update(action)?;
            }

            if self.quitting {
                break;
            }
        }

        self.ti.exit()?;
        Ok(())
    }

    fn handle_event(&mut self, event: term::Event, tx: &UnboundedSender<Action>) -> Result<()> {
        match event {
            term::Event::Quit => tx.send(Action::Quit)?,
            term::Event::Tick => tx.send(Action::Tick)?,
            term::Event::Render => tx.send(Action::Render)?,
            term::Event::Resize(w, h) => tx.send(Action::Resize(w, h))?,
            term::Event::Key(key) => self.handle_key_event(key, tx)?,
            term::Event::Mouse(mouse) => self.handle_mouse_event(mouse, tx)?,
            term::Event::Paste(_content) => {}
            _ => {}
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent, tx: &UnboundedSender<Action>) -> Result<()> {
        match key.code {
            KeyCode::Char('q') => match self.ui.get_composite_editor_mode() {
                Some(EditorMode::Normal) => tx.send(Action::Quit)?,
                None => match self.ui.get_main_editor_mode() {
                    EditorMode::Normal => tx.send(Action::Quit)?,
                    _ => self.ui.handle_editor_event(key),
                },
                _ => self.ui.handle_editor_event(key),
            },
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                tx.send(Action::Quit)?
            }
            _ => self.ui.handle_editor_event(key),
        }
        Ok(())
    }

    fn handle_mouse_event(
        &mut self,
        mouse: MouseEvent,
        tx: &UnboundedSender<Action>,
    ) -> Result<()> {
        match mouse.kind {
            MouseEventKind::Down(button) => match button {
                MouseButton::Left => tx.send(Action::Click(Position {
                    x: mouse.column,
                    y: mouse.row,
                }))?,
                _ => {}
            },
            _ => {}
        }
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<()> {
        match action {
            Action::Tick => {}
            Action::Render => self.render()?,
            Action::Resize(w, h) => self.ti.resize(w, h)?,
            Action::Click(pos) => self.ui.handle_mouse_click(pos),
            Action::Quit => self.quitting = true,
            // Action::Error(msg) => self.error(msg),
            // _ => {}
        }
        Ok(())
    }

    fn render(&mut self) -> Result<()> {
        self.ti.draw(|f| {
            let _ = self.ui.draw(f, f.area());
        })?;
        Ok(())
    }

    // fn error(&mut self, _message: String) {}
}
