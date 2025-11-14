use crate::{
    term::{self, AppTerminal},
    ui::AppUI,
};

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};
// use std::sync::Arc;
// use ticks::TickTick;
use tokio::sync::mpsc::{self, UnboundedSender};

enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    Quit,
    // Error(String),
}

pub struct App {
    // client: Arc<TickTick>,
    ti: AppTerminal,
    ui: AppUI,
    quitting: bool,
}

impl App {
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
        let ti = AppTerminal::new()?;
        let ui = AppUI::new();
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

    fn handle_key_event(
        &mut self,
        key_event: KeyEvent,
        tx: &UnboundedSender<Action>,
    ) -> Result<()> {
        match key_event.code {
            KeyCode::Char('q') => {
                if self.ui.is_quittable() {
                    tx.send(Action::Quit)?;
                } else {
                    self.ui.handle_key_event(key_event);
                }
            }
            KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                tx.send(Action::Quit)?
            }
            _ => self.ui.handle_key_event(key_event),
        }
        Ok(())
    }

    fn handle_mouse_event(
        &mut self,
        mouse_event: MouseEvent,
        _tx: &UnboundedSender<Action>,
    ) -> Result<()> {
        self.ui.handle_mouse_event(mouse_event);
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<()> {
        match action {
            Action::Tick => {}
            Action::Render => self.render()?,
            Action::Resize(w, h) => self.ti.resize(w, h)?,
            // Action::Click(pos) => self.ui.handle_mouse_event(pos),
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
