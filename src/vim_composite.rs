use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders};
use std::env;
use std::fmt;
use std::fs;
use std::io;
use std::io::BufRead;
use tui_textarea::{CursorMove, Input, Key, Scrolling, TextArea};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Normal,
    Insert,
    Visual,
    Operator(char),
}

impl Mode {
    fn block<'a>(
        &self,
        is_active: bool,
        is_single_line: bool,
        is_valid: Option<bool>,
    ) -> Block<'a> {
        let help = match self {
            Self::Normal => {
                "type q to quit, type i to enter insert mode, j/k to navigate between fields"
            }
            Self::Insert => "type Esc to back to normal mode",
            Self::Visual => "type y to yank, type d to delete, type Esc to back to normal mode",
            Self::Operator(_) => "move cursor to apply operator",
        };

        let title = if is_single_line {
            let validation_status = match is_valid {
                Some(true) => "OK",
                Some(false) => "ERROR: Invalid float",
                None => "Empty",
            };
            if is_active {
                format!(
                    "{} MODE - Active Single Line ({}) - {}",
                    self, help, validation_status
                )
            } else {
                format!(
                    "{} MODE - Inactive Single Line - {}",
                    self, validation_status
                )
            }
        } else if is_active {
            format!("{} MODE - Active ({})", self, help)
        } else {
            format!("{} MODE - Inactive", self)
        };

        let style = if is_active {
            Style::default()
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let border_color = if is_single_line {
            match is_valid {
                Some(true) => Color::LightGreen,
                Some(false) => Color::LightRed,
                None => {
                    if is_active {
                        Color::Reset
                    } else {
                        Color::DarkGray
                    }
                }
            }
        } else if is_active {
            Color::Reset
        } else {
            Color::DarkGray
        };

        Block::default()
            .borders(Borders::ALL)
            .style(style)
            .border_style(border_color)
            .title(title)
    }

    fn cursor_style(&self, is_active: bool) -> Style {
        if !is_active {
            return Style::default();
        }

        let color = match self {
            Self::Normal => Color::Reset,
            Self::Insert => Color::LightBlue,
            Self::Visual => Color::LightYellow,
            Self::Operator(_) => Color::LightGreen,
        };
        Style::default().fg(color).add_modifier(Modifier::REVERSED)
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::Normal => write!(f, "NORMAL"),
            Self::Insert => write!(f, "INSERT"),
            Self::Visual => write!(f, "VISUAL"),
            Self::Operator(c) => write!(f, "OPERATOR({})", c),
        }
    }
}

// How the Vim emulation state transitions
enum Transition {
    Nop,
    Mode(Mode),
    Pending(Input),
    Quit,
    SwitchFieldDown, // Navigate to field below
    SwitchFieldUp,   // Navigate to field above
    GoToTop,         // Go to first line of entire form (g command)
    GoToBottom,      // Go to last line of entire form (G command)
}

// State of Vim emulation
#[derive(Clone)]
struct Vim {
    mode: Mode,
    pending: Input,       // Pending input to handle a sequence with two keys like gg
    is_single_line: bool, // Whether this vim state controls a single-line input
    desired_column: Option<usize>, // Desired column for vertical navigation (like vim)
    line_op_cursor_col: Option<usize>, // Store cursor column before line operations (dd, yy, cc)
    cursor_history: Vec<(usize, usize)>, // Track cursor positions for undo/redo
    undo_count: usize,    // Track number of undo operations
}

impl Vim {
    fn new(mode: Mode, is_single_line: bool) -> Self {
        Self {
            mode,
            pending: Input::default(),
            is_single_line,
            desired_column: None,
            line_op_cursor_col: None,
            cursor_history: Vec::new(),
            undo_count: 0,
        }
    }

    fn with_pending(self, pending: Input) -> Self {
        Self {
            mode: self.mode,
            pending,
            is_single_line: self.is_single_line,
            desired_column: self.desired_column,
            line_op_cursor_col: self.line_op_cursor_col,
            cursor_history: self.cursor_history,
            undo_count: self.undo_count,
        }
    }

    fn transition(
        &mut self,
        input: Input,
        textarea: &mut TextArea<'_>,
        _which_field: usize,
    ) -> Transition {
        if input.key == Key::Null {
            return Transition::Nop;
        }

        match self.mode {
            Mode::Normal | Mode::Visual | Mode::Operator(_) => {
                match input {
                    Input {
                        key: Key::Char('h'),
                        ..
                    }
                    | Input { key: Key::Left, .. } => {
                        textarea.move_cursor(CursorMove::Back);
                        // In normal mode, ensure cursor doesn't go beyond valid position
                        constrain_cursor_for_normal_mode(textarea);
                        // Update desired column on horizontal movement
                        self.desired_column = Some(textarea.cursor().1);
                    }
                    Input {
                        key: Key::Char('j'),
                        ..
                    }
                    | Input { key: Key::Down, .. } => {
                        if self.is_single_line {
                            // Single line field - only move to next field if not at bottom
                            if _which_field < 2 {
                                return Transition::SwitchFieldDown;
                            }
                            // At bottom field, do nothing
                        } else {
                            // Multi-line field - check if we're on the last line
                            let (row, current_col) = textarea.cursor();
                            let total_lines = textarea.lines().len();
                            if row + 1 >= total_lines {
                                // On last line, check if we can move to next field
                                if _which_field < 2 {
                                    return Transition::SwitchFieldDown;
                                }
                                // At bottom field, do nothing
                            } else {
                                // Not on last line, move down within field
                                // Set desired column if not already set
                                if self.desired_column.is_none() {
                                    self.desired_column = Some(current_col);
                                }

                                textarea.move_cursor(CursorMove::Down);

                                // Try to reach desired column
                                if let Some(desired_col) = self.desired_column {
                                    let (new_row, _) = textarea.cursor();
                                    let line_len = textarea.lines()[new_row].len();
                                    // In normal mode, can't go to newline position
                                    let max_col = if line_len > 0 { line_len - 1 } else { 0 };
                                    let target_col = desired_col.min(max_col);
                                    textarea.move_cursor(CursorMove::Jump(
                                        new_row as u16,
                                        target_col as u16,
                                    ));
                                }
                            }
                        }
                    }
                    Input {
                        key: Key::Char('k'),
                        ..
                    }
                    | Input { key: Key::Up, .. } => {
                        if self.is_single_line {
                            // Single line field - only move to previous field if not at top
                            if _which_field > 0 {
                                return Transition::SwitchFieldUp;
                            }
                            // At top field, do nothing
                        } else {
                            // Multi-line field - check if we're on the first line
                            let (row, current_col) = textarea.cursor();
                            if row == 0 {
                                // On first line, check if we can move to previous field
                                if _which_field > 0 {
                                    return Transition::SwitchFieldUp;
                                }
                                // At top field, do nothing
                            } else {
                                // Not on first line, move up within field
                                // Set desired column if not already set
                                if self.desired_column.is_none() {
                                    self.desired_column = Some(current_col);
                                }

                                textarea.move_cursor(CursorMove::Up);

                                // Try to reach desired column
                                if let Some(desired_col) = self.desired_column {
                                    let (new_row, _) = textarea.cursor();
                                    let line_len = textarea.lines()[new_row].len();
                                    // In normal mode, can't go to newline position
                                    let max_col = if line_len > 0 { line_len - 1 } else { 0 };
                                    let target_col = desired_col.min(max_col);
                                    textarea.move_cursor(CursorMove::Jump(
                                        new_row as u16,
                                        target_col as u16,
                                    ));
                                }
                            }
                        }
                    }
                    Input {
                        key: Key::Char('l'),
                        ..
                    }
                    | Input {
                        key: Key::Right, ..
                    } => {
                        textarea.move_cursor(CursorMove::Forward);
                        // In normal mode, ensure cursor doesn't go beyond valid position
                        constrain_cursor_for_normal_mode(textarea);
                        // Update desired column on horizontal movement
                        self.desired_column = Some(textarea.cursor().1);
                    }
                    Input {
                        key: Key::Char('w'),
                        ..
                    } => {
                        textarea.move_cursor(CursorMove::WordForward);
                        // In normal mode, ensure cursor doesn't go beyond valid position
                        constrain_cursor_for_normal_mode(textarea);
                        // Update desired column on horizontal movement
                        self.desired_column = Some(textarea.cursor().1);
                    }
                    Input {
                        key: Key::Char('e'),
                        ctrl: false,
                        ..
                    } => {
                        textarea.move_cursor(CursorMove::WordEnd);
                        if matches!(self.mode, Mode::Operator(_)) {
                            textarea.move_cursor(CursorMove::Forward); // Include the text under the cursor
                        }
                        // In normal mode, ensure cursor doesn't go beyond valid position
                        constrain_cursor_for_normal_mode(textarea);
                        // Update desired column on horizontal movement
                        self.desired_column = Some(textarea.cursor().1);
                    }
                    Input {
                        key: Key::Char('b'),
                        ctrl: false,
                        ..
                    } => {
                        textarea.move_cursor(CursorMove::WordBack);
                        // In normal mode, ensure cursor doesn't go beyond valid position
                        constrain_cursor_for_normal_mode(textarea);
                        // Update desired column on horizontal movement
                        self.desired_column = Some(textarea.cursor().1);
                    }
                    Input {
                        key: Key::Char('^'),
                        ..
                    } => {
                        textarea.move_cursor(CursorMove::Head);
                        // Update desired column on horizontal movement
                        self.desired_column = Some(textarea.cursor().1);
                    }
                    Input {
                        key: Key::Char('$'),
                        ..
                    } => {
                        textarea.move_cursor(CursorMove::End);
                        // In normal mode, ensure cursor doesn't go beyond valid position
                        constrain_cursor_for_normal_mode(textarea);
                        // Update desired column on horizontal movement
                        self.desired_column = Some(textarea.cursor().1);
                    }
                    Input {
                        key: Key::Char('D'),
                        ..
                    } => {
                        // Store cursor position before deletion for undo history
                        let (row, col) = textarea.cursor();
                        self.add_to_cursor_history(row, col);
                        textarea.delete_line_by_end();
                        return Transition::Mode(Mode::Normal);
                    }
                    Input {
                        key: Key::Char('C'),
                        ..
                    } => {
                        // Store cursor position before deletion for undo history
                        let (row, col) = textarea.cursor();
                        self.add_to_cursor_history(row, col);
                        textarea.delete_line_by_end();
                        textarea.cancel_selection();
                        return Transition::Mode(Mode::Insert);
                    }
                    Input {
                        key: Key::Char('p'),
                        ..
                    } => {
                        // Store cursor position before paste for undo history
                        let (row, col) = textarea.cursor();
                        self.add_to_cursor_history(row, col);
                        textarea.paste();
                        return Transition::Mode(Mode::Normal);
                    }
                    Input {
                        key: Key::Char('u'),
                        ctrl: false,
                        ..
                    } => {
                        textarea.undo();
                        // Restore cursor position from history if available
                        self.undo_count += 1;
                        if self.undo_count <= self.cursor_history.len() {
                            let history_index = self.cursor_history.len() - self.undo_count;
                            let (row, col) = self.cursor_history[history_index];
                            let lines = textarea.lines();
                            if !lines.is_empty() && row < lines.len() {
                                let line_len = lines[row].len();
                                let max_col = if line_len > 0 { line_len - 1 } else { 0 };
                                let target_col = col.min(max_col);
                                textarea
                                    .move_cursor(CursorMove::Jump(row as u16, target_col as u16));
                            }
                        }
                        // Constrain cursor to normal mode limits
                        constrain_cursor_for_normal_mode(textarea);
                        return Transition::Mode(Mode::Normal);
                    }
                    Input {
                        key: Key::Char('r'),
                        ctrl: true,
                        ..
                    } => {
                        textarea.redo();
                        // Handle redo cursor positioning
                        if self.undo_count > 0 {
                            self.undo_count -= 1;
                            if self.undo_count < self.cursor_history.len() {
                                let history_index = self.cursor_history.len() - self.undo_count - 1;
                                if history_index < self.cursor_history.len() {
                                    let (row, col) = self.cursor_history[history_index];
                                    let lines = textarea.lines();
                                    if !lines.is_empty() && row < lines.len() {
                                        let line_len = lines[row].len();
                                        let max_col = if line_len > 0 { line_len - 1 } else { 0 };
                                        let target_col = col.min(max_col);
                                        textarea.move_cursor(CursorMove::Jump(
                                            row as u16,
                                            target_col as u16,
                                        ));
                                    }
                                }
                            }
                        }
                        // Constrain cursor to normal mode limits
                        constrain_cursor_for_normal_mode(textarea);
                        return Transition::Mode(Mode::Normal);
                    }
                    Input {
                        key: Key::Char('x'),
                        ctrl: false,
                        ..
                    } => {
                        // Store cursor position before deletion for undo history
                        let (row, col) = textarea.cursor();
                        self.add_to_cursor_history(row, col);
                        textarea.delete_next_char();
                        return Transition::Mode(Mode::Normal);
                    }
                    Input {
                        key: Key::Char('i'),
                        ..
                    } => {
                        textarea.cancel_selection();
                        // Reset desired column when entering insert mode
                        self.desired_column = None;
                        return Transition::Mode(Mode::Insert);
                    }
                    Input {
                        key: Key::Char('a'),
                        ..
                    } => {
                        textarea.cancel_selection();
                        textarea.move_cursor(CursorMove::Forward);
                        // Reset desired column when entering insert mode
                        self.desired_column = None;
                        return Transition::Mode(Mode::Insert);
                    }
                    Input {
                        key: Key::Char('A'),
                        ..
                    } => {
                        textarea.cancel_selection();
                        textarea.move_cursor(CursorMove::End);
                        // Reset desired column when entering insert mode
                        self.desired_column = None;
                        return Transition::Mode(Mode::Insert);
                    }
                    Input {
                        key: Key::Char('o'),
                        ..
                    } if !self.is_single_line => {
                        // Store cursor position before inserting newline for undo history
                        let (row, col) = textarea.cursor();
                        self.add_to_cursor_history(row, col);
                        textarea.move_cursor(CursorMove::End);
                        textarea.insert_newline();
                        // Reset desired column when entering insert mode
                        self.desired_column = None;
                        return Transition::Mode(Mode::Insert);
                    }
                    Input {
                        key: Key::Char('O'),
                        ..
                    } if !self.is_single_line => {
                        // Store cursor position before inserting newline for undo history
                        let (row, col) = textarea.cursor();
                        self.add_to_cursor_history(row, col);
                        textarea.move_cursor(CursorMove::Head);
                        textarea.insert_newline();
                        textarea.move_cursor(CursorMove::Up);
                        // Reset desired column when entering insert mode
                        self.desired_column = None;
                        return Transition::Mode(Mode::Insert);
                    }
                    Input {
                        key: Key::Char('I'),
                        ..
                    } => {
                        textarea.cancel_selection();
                        textarea.move_cursor(CursorMove::Head);
                        // Reset desired column when entering insert mode
                        self.desired_column = None;
                        return Transition::Mode(Mode::Insert);
                    }
                    Input {
                        key: Key::Char('q'),
                        ..
                    } => return Transition::Quit,
                    Input {
                        key: Key::Char('e'),
                        ctrl: true,
                        ..
                    } if !self.is_single_line => textarea.scroll((1, 0)),
                    Input {
                        key: Key::Char('y'),
                        ctrl: true,
                        ..
                    } if !self.is_single_line => textarea.scroll((-1, 0)),
                    Input {
                        key: Key::Char('d'),
                        ctrl: true,
                        ..
                    } if !self.is_single_line => textarea.scroll(Scrolling::HalfPageDown),
                    Input {
                        key: Key::Char('u'),
                        ctrl: true,
                        ..
                    } if !self.is_single_line => textarea.scroll(Scrolling::HalfPageUp),
                    Input {
                        key: Key::Char('f'),
                        ctrl: true,
                        ..
                    } if !self.is_single_line => textarea.scroll(Scrolling::PageDown),
                    Input {
                        key: Key::Char('b'),
                        ctrl: true,
                        ..
                    } if !self.is_single_line => textarea.scroll(Scrolling::PageUp),
                    Input {
                        key: Key::Char('v'),
                        ctrl: false,
                        ..
                    } if self.mode == Mode::Normal => {
                        textarea.start_selection();
                        return Transition::Mode(Mode::Visual);
                    }
                    Input {
                        key: Key::Char('V'),
                        ctrl: false,
                        ..
                    } if self.mode == Mode::Normal && !self.is_single_line => {
                        textarea.move_cursor(CursorMove::Head);
                        textarea.start_selection();
                        textarea.move_cursor(CursorMove::End);
                        return Transition::Mode(Mode::Visual);
                    }
                    Input { key: Key::Esc, .. }
                    | Input {
                        key: Key::Char('v'),
                        ctrl: false,
                        ..
                    } if self.mode == Mode::Visual => {
                        textarea.cancel_selection();
                        return Transition::Mode(Mode::Normal);
                    }
                    Input { key: Key::Esc, .. } if matches!(self.mode, Mode::Operator(_)) => {
                        textarea.cancel_selection();
                        return Transition::Mode(Mode::Normal);
                    }
                    Input {
                        key: Key::Char('g'),
                        ctrl: false,
                        ..
                    } if matches!(
                        self.pending,
                        Input {
                            key: Key::Char('g'),
                            ctrl: false,
                            ..
                        }
                    ) =>
                    {
                        // Handle 'gg' - go to first line of entire form
                        return Transition::GoToTop;
                    }
                    Input {
                        key: Key::Char('G'),
                        ctrl: false,
                        ..
                    } => {
                        // Handle 'G' - go to last line of entire form
                        return Transition::GoToBottom;
                    }
                    Input {
                        key: Key::Char(c),
                        ctrl: false,
                        ..
                    } if matches!(self.mode, Mode::Operator(op) if op == c)
                        && !self.is_single_line =>
                    {
                        // Handle yy, dd, cc with proper last line handling
                        // Store the current cursor position before any movements
                        let (current_row, current_col) = textarea.cursor();
                        if self.line_op_cursor_col.is_none() {
                            self.line_op_cursor_col = Some(current_col);
                        }
                        let total_lines = textarea.lines().len();

                        if current_row + 1 == total_lines && total_lines > 1 {
                            // Last line case: select from end of previous line to end of current line
                            textarea.move_cursor(CursorMove::Up);
                            textarea.move_cursor(CursorMove::End);
                            textarea.start_selection();
                            textarea.move_cursor(CursorMove::Down);
                            textarea.move_cursor(CursorMove::End);
                        } else {
                            // Normal case: select entire line including newline
                            textarea.move_cursor(CursorMove::Head);
                            textarea.start_selection();
                            let cursor = textarea.cursor();
                            textarea.move_cursor(CursorMove::Down);
                            if cursor == textarea.cursor() {
                                textarea.move_cursor(CursorMove::End); // At the last line, move to end of the line instead
                            }
                        }
                    }
                    Input {
                        key: Key::Char(c),
                        ctrl: false,
                        ..
                    } if matches!(self.mode, Mode::Operator(op) if op == c)
                        && self.is_single_line =>
                    {
                        // Handle dd, yy, cc for single line - operate on entire line
                        // Store the current cursor position before any movements
                        let (_, current_col) = textarea.cursor();
                        if self.line_op_cursor_col.is_none() {
                            self.line_op_cursor_col = Some(current_col);
                        }
                        textarea.move_cursor(CursorMove::Head);
                        textarea.start_selection();
                        textarea.move_cursor(CursorMove::End);
                    }
                    Input {
                        key: Key::Char(op @ ('y' | 'd' | 'c')),
                        ctrl: false,
                        ..
                    } if self.mode == Mode::Normal => {
                        textarea.start_selection();
                        return Transition::Mode(Mode::Operator(op));
                    }
                    Input {
                        key: Key::Char('y'),
                        ctrl: false,
                        ..
                    } if self.mode == Mode::Visual => {
                        textarea.move_cursor(CursorMove::Forward); // Vim's text selection is inclusive
                        textarea.copy();
                        return Transition::Mode(Mode::Normal);
                    }
                    Input {
                        key: Key::Char('d'),
                        ctrl: false,
                        ..
                    } if self.mode == Mode::Visual => {
                        // Store cursor position before cutting for undo history
                        let (row, col) = textarea.cursor();
                        self.add_to_cursor_history(row, col);
                        textarea.move_cursor(CursorMove::Forward); // Vim's text selection is inclusive
                        textarea.cut();
                        return Transition::Mode(Mode::Normal);
                    }
                    Input {
                        key: Key::Char('c'),
                        ctrl: false,
                        ..
                    } if self.mode == Mode::Visual => {
                        // Store cursor position before cutting for undo history
                        let (row, col) = textarea.cursor();
                        self.add_to_cursor_history(row, col);
                        textarea.move_cursor(CursorMove::Forward); // Vim's text selection is inclusive
                        textarea.cut();
                        return Transition::Mode(Mode::Insert);
                    }
                    Input {
                        key: Key::Enter, ..
                    } if self.is_single_line => {
                        // Enter in single-line field: move to next field
                        if _which_field < 2 {
                            // Set desired column to 0 when moving to next field
                            self.desired_column = Some(0);
                            return Transition::SwitchFieldDown;
                        }
                        // At bottom field, do nothing
                    }
                    Input {
                        key: Key::Enter, ..
                    } if !self.is_single_line => {
                        // Enter in normal mode: move to beginning of next line
                        let (current_row, _) = textarea.cursor();
                        let total_lines = textarea.lines().len();
                        if current_row + 1 < total_lines {
                            // Move to next line, beginning
                            textarea.move_cursor(CursorMove::Jump((current_row + 1) as u16, 0));
                            // Update desired column to beginning of line
                            self.desired_column = Some(0);
                        } else {
                            // On last line, try to move to next field
                            if _which_field < 2 {
                                // Set desired column to 0 when moving to next field
                                self.desired_column = Some(0);
                                return Transition::SwitchFieldDown;
                            }
                            // At bottom field, do nothing
                        }
                    }
                    input => return Transition::Pending(input),
                }

                // Handle the pending operator
                match self.mode {
                    Mode::Operator('y') => {
                        // Store cursor position for undo history before copying
                        if let Some(stored_col) = self.line_op_cursor_col {
                            let current_selection = textarea.selection_range();
                            if let Some((start, _)) = current_selection {
                                self.add_to_cursor_history(start.0, stored_col);
                            } else {
                                let (row, _) = textarea.cursor();
                                self.add_to_cursor_history(row, stored_col);
                            }
                        }
                        textarea.copy();
                        // For yy, preserve existing desired column or use stored cursor position
                        if self.desired_column.is_none() {
                            self.desired_column = self.line_op_cursor_col;
                        }
                        self.line_op_cursor_col = None; // Clear the stored position
                        Transition::Mode(Mode::Normal)
                    }
                    Mode::Operator('d') => {
                        // Store cursor position for undo history before deletion
                        if let Some(stored_col) = self.line_op_cursor_col {
                            // Store the original position before any selection happened
                            // We need to find which row the cursor was on when dd was initiated
                            let current_selection = textarea.selection_range();
                            if let Some((start, _)) = current_selection {
                                self.add_to_cursor_history(start.0, stored_col);
                            } else {
                                let (row, _) = textarea.cursor();
                                self.add_to_cursor_history(row, stored_col);
                            }
                        }

                        // Use the stored cursor column from before the line operation started
                        let preserved_col = self.line_op_cursor_col.unwrap_or(0);
                        textarea.cut();

                        // After dd, position cursor at preserved column on the resulting line
                        let (new_row, _) = textarea.cursor();
                        let lines = textarea.lines();
                        if !lines.is_empty() && new_row < lines.len() {
                            let line_len = lines[new_row].len();
                            let max_col = if line_len > 0 { line_len - 1 } else { 0 };
                            let target_col = preserved_col.min(max_col);
                            textarea
                                .move_cursor(CursorMove::Jump(new_row as u16, target_col as u16));
                        }

                        // Always update desired column to the original cursor position
                        self.desired_column = Some(preserved_col);
                        self.line_op_cursor_col = None; // Clear the stored position
                        Transition::Mode(Mode::Normal)
                    }
                    Mode::Operator('c') => {
                        // Store cursor position for undo history before cutting
                        if let Some(stored_col) = self.line_op_cursor_col {
                            let current_selection = textarea.selection_range();
                            if let Some((start, _)) = current_selection {
                                self.add_to_cursor_history(start.0, stored_col);
                            } else {
                                let (row, _) = textarea.cursor();
                                self.add_to_cursor_history(row, stored_col);
                            }
                        }
                        textarea.cut();
                        // Reset desired column when entering insert mode after cc
                        self.desired_column = None;
                        self.line_op_cursor_col = None; // Clear the stored position
                        Transition::Mode(Mode::Insert)
                    }
                    _ => Transition::Nop,
                }
            }
            Mode::Insert => match input {
                Input { key: Key::Esc, .. }
                | Input {
                    key: Key::Char('c'),
                    ctrl: true,
                    ..
                } => {
                    // When transitioning from insert to normal mode, move cursor back
                    // unless already at the beginning of the line
                    let (_row, col) = textarea.cursor();
                    if col > 0 {
                        textarea.move_cursor(CursorMove::Back);
                    }
                    // In normal mode, ensure cursor doesn't go beyond valid position
                    constrain_cursor_for_normal_mode(textarea);
                    // Reset desired column when leaving insert mode
                    self.desired_column = None;
                    Transition::Mode(Mode::Normal)
                }
                Input {
                    key: Key::Enter, ..
                } if self.is_single_line => {
                    // Enter in single-line insert mode: move to next field
                    if _which_field < 2 {
                        // Reset desired column when leaving insert mode and moving to next field
                        self.desired_column = None;
                        return Transition::SwitchFieldDown;
                    }
                    // At bottom field, do nothing
                    Transition::Nop
                }
                Input {
                    key: Key::Char('m'),
                    ctrl: true,
                    ..
                } if self.is_single_line => Transition::Nop, // Ignore Ctrl+M (same as Enter)
                input => {
                    // Store cursor position before text input for undo history
                    let (row, col) = textarea.cursor();
                    self.add_to_cursor_history(row, col);
                    textarea.input(input); // Use default key mappings in insert mode
                    Transition::Mode(Mode::Insert)
                }
            },
        }
    }
}

fn validate_float(textarea: &TextArea) -> Option<bool> {
    let text = &textarea.lines()[0];
    if text.is_empty() {
        None // Empty input
    } else {
        Some(text.parse::<f64>().is_ok())
    }
}

fn inactivate(
    textarea: &mut TextArea<'_>,
    mode: Mode,
    is_single_line: bool,
    is_valid: Option<bool>,
) {
    textarea.set_cursor_line_style(Style::default());
    textarea.set_cursor_style(mode.cursor_style(false));
    textarea.set_block(mode.block(false, is_single_line, is_valid));
    if is_single_line {
        let text_color = match is_valid {
            Some(true) => Color::LightGreen,
            Some(false) => Color::LightRed,
            None => Color::DarkGray,
        };
        textarea.set_style(Style::default().fg(text_color));
    }
}

fn activate(textarea: &mut TextArea<'_>, mode: Mode, is_single_line: bool, is_valid: Option<bool>) {
    textarea.set_cursor_line_style(Style::default());
    textarea.set_cursor_style(mode.cursor_style(true));
    textarea.set_block(mode.block(true, is_single_line, is_valid));
    if is_single_line {
        let text_color = match is_valid {
            Some(true) => Color::LightGreen,
            Some(false) => Color::LightRed,
            None => Color::Reset,
        };
        textarea.set_style(Style::default().fg(text_color));
    }
}

impl Vim {
    // Helper method to manage cursor history size
    fn add_to_cursor_history(&mut self, row: usize, col: usize) {
        const MAX_HISTORY_SIZE: usize = 100; // Limit history to prevent memory issues

        self.cursor_history.push((row, col));
        self.undo_count = 0; // Reset undo count when making new changes

        // Keep history size manageable
        if self.cursor_history.len() > MAX_HISTORY_SIZE {
            self.cursor_history.remove(0);
        }
    }
}

fn constrain_cursor_for_normal_mode(textarea: &mut TextArea<'_>) {
    let (row, col) = textarea.cursor();
    let lines = textarea.lines();
    if lines.is_empty() {
        return;
    }

    if row >= lines.len() {
        return;
    }

    let line_len = lines[row].len();
    if line_len == 0 {
        // Empty line, cursor should be at column 0
        if col > 0 {
            textarea.move_cursor(CursorMove::Jump(row as u16, 0));
        }
    } else {
        // Non-empty line, cursor should not be beyond last character (line_len - 1)
        let max_col = line_len.saturating_sub(1);
        if col > max_col {
            textarea.move_cursor(CursorMove::Jump(row as u16, max_col as u16));
        }
    }
}

fn move_to_field_with_column_preservation(
    _from_field: usize,
    to_field: usize,
    desired_column: usize,
    single_line_textarea: &mut TextArea<'_>,
    textarea: &mut [TextArea<'_>; 2],
    position: &str, // "first" or "last"
    is_normal_mode: bool,
) {
    if to_field == 0 {
        // Moving to single-line field
        let line_len = single_line_textarea.lines()[0].len();
        let max_col = if is_normal_mode && line_len > 0 {
            line_len - 1 // In normal mode, can't go to newline position
        } else {
            line_len // In insert mode, can go to newline position
        };
        let target_col = desired_column.min(max_col);
        single_line_textarea.move_cursor(CursorMove::Jump(0, target_col as u16));
    } else {
        // Moving to multi-line field
        let target_textarea = &mut textarea[to_field - 1];
        let lines = target_textarea.lines();

        if lines.is_empty() {
            // Empty field, just go to start
            target_textarea.move_cursor(CursorMove::Jump(0, 0));
        } else {
            let target_row = if position == "first" {
                0
            } else {
                lines.len() - 1
            };

            let line_len = lines[target_row].len();
            let max_col = if is_normal_mode && line_len > 0 {
                line_len - 1 // In normal mode, can't go to newline position
            } else {
                line_len // In insert mode, can go to newline position
            };
            let target_col = desired_column.min(max_col);
            target_textarea.move_cursor(CursorMove::Jump(target_row as u16, target_col as u16));
        }
    }
}

pub struct CompositeEditor {
    textareas: Vec<TextArea<'static>>,
}

fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    enable_raw_mode()?;
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;

    // Initialize three text areas: single-line input at top, two multi-line editors below
    let mut single_line_textarea = TextArea::default();
    single_line_textarea.set_cursor_line_style(Style::default());
    single_line_textarea.set_placeholder_text("Enter a valid float (e.g. 1.56)");
    single_line_textarea.insert_str("42.0");

    let mut textarea = [TextArea::default(), TextArea::default()];
    // Add default text to the first multi-line area
    textarea[0].insert_str("This is line 1\nThis is line 2 with more text\nThis is line 3\nShort\nThis is a longer line 5 for testing column preservation\nLine 6");
    // Add default text to the second multi-line area
    textarea[1].insert_str("First line of second area\nSecond line here\nThird line\nFourth line with some content\nLast line of default text");

    // Load files if provided as arguments
    if let Some(path) = env::args().nth(1) {
        let file = fs::File::open(path)?;
        textarea[0] = io::BufReader::new(file)
            .lines()
            .collect::<io::Result<_>>()?;
    }

    if let Some(path) = env::args().nth(2) {
        let file = fs::File::open(path)?;
        textarea[1] = io::BufReader::new(file)
            .lines()
            .collect::<io::Result<_>>()?;
    }

    // Create layout with three areas: single-line at top, two split areas below
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Single-line input
            Constraint::Min(1),    // Rest for the split areas
        ]);

    let split_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)]);

    let mut vim_mode = Mode::Normal; // Shared mode across all panes
    let mut vim = [
        Vim::new(vim_mode, true),  // Single-line input
        Vim::new(vim_mode, false), // Multi-line editor 1
        Vim::new(vim_mode, false), // Multi-line editor 2
    ];
    let mut which = 0; // 0 = single-line, 1 = top editor, 2 = bottom editor
    let mut is_valid = validate_float(&single_line_textarea);

    // Initially activate the single-line input
    activate(&mut single_line_textarea, vim_mode, true, is_valid);
    inactivate(&mut textarea[0], vim_mode, false, None);
    inactivate(&mut textarea[1], vim_mode, false, None);

    loop {
        term.draw(|f| {
            let main_chunks = main_layout.split(f.area());
            let split_chunks = split_layout.split(main_chunks[1]);

            // Render single-line input at top
            f.render_widget(&single_line_textarea, main_chunks[0]);

            // Render split editors below
            f.render_widget(&textarea[0], split_chunks[0]);
            f.render_widget(&textarea[1], split_chunks[1]);
        })?;

        let input: Input = crossterm::event::read()?.into();

        let transition = if which == 0 {
            // Single-line input
            vim[0].transition(input, &mut single_line_textarea, which)
        } else {
            // Multi-line editors
            vim[which].transition(input, &mut textarea[which - 1], which)
        };

        match transition {
            Transition::Mode(mode) if vim_mode != mode => {
                vim_mode = mode;
                // Update all vim states to the new mode
                vim[0] = Vim::new(vim_mode, true);
                vim[1] = Vim::new(vim_mode, false);
                vim[2] = Vim::new(vim_mode, false);

                if which == 0 {
                    is_valid = validate_float(&single_line_textarea);
                    activate(&mut single_line_textarea, vim_mode, true, is_valid);
                } else {
                    activate(&mut textarea[which - 1], vim_mode, false, None);
                }
            }
            Transition::Mode(_) => {
                if which == 0 {
                    is_valid = validate_float(&single_line_textarea);
                    activate(&mut single_line_textarea, vim_mode, true, is_valid);
                }
            }
            Transition::Nop => {}
            Transition::Pending(input) => {
                vim[which] = vim[which].clone().with_pending(input);
            }
            Transition::SwitchFieldDown => {
                // Get desired column for preservation
                let desired_column = vim[which].desired_column.unwrap_or_else(|| {
                    if which == 0 {
                        single_line_textarea.cursor().1
                    } else {
                        textarea[which - 1].cursor().1
                    }
                });

                // Deactivate current field
                if which == 0 {
                    inactivate(&mut single_line_textarea, vim_mode, true, is_valid);
                } else {
                    inactivate(&mut textarea[which - 1], vim_mode, false, None);
                }

                // Move to next field (no wrapping)
                let next_field = which + 1;

                // Move cursor with column preservation
                move_to_field_with_column_preservation(
                    which,
                    next_field,
                    desired_column,
                    &mut single_line_textarea,
                    &mut textarea,
                    "first",
                    matches!(
                        vim[next_field].mode,
                        Mode::Normal | Mode::Visual | Mode::Operator(_)
                    ),
                );

                which = next_field;

                // Update vim state with preserved desired column
                vim[which].desired_column = Some(desired_column);

                // Activate new field
                if which == 0 {
                    is_valid = validate_float(&single_line_textarea);
                    activate(&mut single_line_textarea, vim_mode, true, is_valid);
                } else {
                    activate(&mut textarea[which - 1], vim_mode, false, None);
                }
            }
            Transition::SwitchFieldUp => {
                // Get desired column for preservation
                let desired_column = vim[which].desired_column.unwrap_or_else(|| {
                    if which == 0 {
                        single_line_textarea.cursor().1
                    } else {
                        textarea[which - 1].cursor().1
                    }
                });

                // Deactivate current field
                if which == 0 {
                    inactivate(&mut single_line_textarea, vim_mode, true, is_valid);
                } else {
                    inactivate(&mut textarea[which - 1], vim_mode, false, None);
                }

                // Move to previous field (no wrapping)
                let prev_field = which - 1;

                // Move cursor with column preservation
                move_to_field_with_column_preservation(
                    which,
                    prev_field,
                    desired_column,
                    &mut single_line_textarea,
                    &mut textarea,
                    "last",
                    matches!(
                        vim[prev_field].mode,
                        Mode::Normal | Mode::Visual | Mode::Operator(_)
                    ),
                );

                which = prev_field;

                // Update vim state with preserved desired column
                vim[which].desired_column = Some(desired_column);

                // Activate new field
                if which == 0 {
                    is_valid = validate_float(&single_line_textarea);
                    activate(&mut single_line_textarea, vim_mode, true, is_valid);
                } else {
                    activate(&mut textarea[which - 1], vim_mode, false, None);
                }
            }
            Transition::GoToTop => {
                // Go to first line of entire form (first field)
                if which != 0 {
                    // Deactivate current field
                    inactivate(&mut textarea[which - 1], vim_mode, false, None);
                    which = 0;
                }

                // Move to start of single-line field
                single_line_textarea.move_cursor(CursorMove::Jump(0, 0));
                // Reset desired column for g movement
                vim[which].desired_column = Some(0);
                is_valid = validate_float(&single_line_textarea);
                activate(&mut single_line_textarea, vim_mode, true, is_valid);
            }
            Transition::GoToBottom => {
                // Go to last line of entire form (last field)
                if which != 2 {
                    // Deactivate current field
                    if which == 0 {
                        inactivate(&mut single_line_textarea, vim_mode, true, is_valid);
                    } else {
                        inactivate(&mut textarea[which - 1], vim_mode, false, None);
                    }
                    which = 2;
                }

                // Move to last line of bottom field
                let lines = textarea[1].lines();
                if lines.is_empty() {
                    textarea[1].move_cursor(CursorMove::Jump(0, 0));
                    vim[which].desired_column = Some(0);
                } else {
                    let last_row = lines.len() - 1;
                    let line_len = lines[last_row].len();
                    // In normal mode, cursor can't go to newline position
                    let last_col = if matches!(
                        vim[which].mode,
                        Mode::Normal | Mode::Visual | Mode::Operator(_)
                    ) && line_len > 0
                    {
                        line_len - 1
                    } else {
                        line_len
                    };
                    textarea[1].move_cursor(CursorMove::Jump(last_row as u16, last_col as u16));
                    vim[which].desired_column = Some(last_col);
                }
                activate(&mut textarea[1], vim_mode, false, None);
            }

            Transition::Quit => break,
        }
    }

    disable_raw_mode()?;
    crossterm::execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    term.show_cursor()?;

    println!("Single-line input: {:?}", single_line_textarea.lines()[0]);
    println!("Top textarea: {:?}", textarea[0].lines());
    println!("Bottom textarea: {:?}", textarea[1].lines());

    Ok(())
}
