use std::io::Error;

use crossterm::{
    event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    terminal,
};

use crate::{Document, Terminal, Row};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
    cursor_position: Position,
    document: Document,
}

impl Editor {
    pub fn default() -> Self {
        Self {
            should_quit: false,
            terminal: Terminal::default().expect("Failed to initialize terminal"),
            cursor_position: Position::default(),
            document: Document::open(),
        }
    }

    pub fn run(&mut self) {
        loop {
            if let Err(error) = self.refresh_screen() {
                die(&error)
            }
            if self.should_quit {
                break;
            }
            if let Err(error) = self.process_keypress() {
                die(&error);
            }
        }

        terminal::disable_raw_mode().unwrap();
    }

    fn refresh_screen(&self) -> Result<(), Error> {
        Terminal::hide_cursor();
        Terminal::cursor_position(&Position::default());
        if self.should_quit {
            Terminal::clear_screen();
            println!("Goodbye and thanks for all the fish!\r");
        } else {
            self.draw_rows();
            Terminal::cursor_position(&self.cursor_position);
        }
        Terminal::show_cursor();
        Terminal::flush()
    }

    fn draw_welcome_message(&self) {
        let mut welcome_message = format!("te editor -- version {}\r", VERSION);
        let width = self.terminal.size().width as usize;
        let len = welcome_message.len();
        let padding = width.saturating_sub(len) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));
        welcome_message = format!("~{}{}", spaces, welcome_message);
        welcome_message.truncate(width);
        println!("{}\r", welcome_message);
    }

    fn draw_rows(&self) {
        let height = self.terminal.size().height;
        for terminal_row in 0..height - 1 {
            Terminal::clear_current_line();

            if let Some(row) = self.document.row(terminal_row as usize) {
                self.draw_row(row);
            } else if terminal_row == height / 3 {
                self.draw_welcome_message();
            } else {
                println!("~\r");
            }
        }
    }

    fn draw_row(&self, row: &Row) {
        let start = 0;
        let end = self.terminal.size().width as usize;
        let row = row.render(start, end);
        println!("{}\r", row)
    }

    fn process_keypress(&mut self) -> Result<(), Error> {
        let pressed_key = Terminal::read_key()?;
        if pressed_key.kind == KeyEventKind::Release {
            return Ok(());
        }
        match (pressed_key.modifiers, pressed_key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('q')) => self.should_quit = true,
            (KeyModifiers::NONE, KeyCode::Up)
            | (KeyModifiers::NONE, KeyCode::Down)
            | (KeyModifiers::NONE, KeyCode::Left)
            | (KeyModifiers::NONE, KeyCode::Right)
            | (KeyModifiers::NONE, KeyCode::PageUp)
            | (KeyModifiers::NONE, KeyCode::PageDown)
            | (KeyModifiers::NONE, KeyCode::End)
            | (KeyModifiers::NONE, KeyCode::Home) => self.move_cursor(pressed_key),
            _ => (),
        }
        Ok(())
    }

    fn move_cursor(&mut self, key: KeyEvent) {
        let Position { mut x, mut y } = self.cursor_position;
        let height = self.terminal.size().height as usize;
        let width = self.terminal.size().width as usize;
        match key.code {
            KeyCode::Up => y = y.saturating_sub(1),
            KeyCode::Down => {
                if y < height {
                    y = y.saturating_add(1)
                }
            }
            KeyCode::Left => x = x.saturating_sub(1),
            KeyCode::Right => {
                if x < width {
                    x = x.saturating_add(1)
                }
            }
            KeyCode::PageUp => y = 0,
            KeyCode::PageDown => y = height,
            KeyCode::End => x = width,
            KeyCode::Home => x = 0,
            _ => (),
        };
        self.cursor_position = Position { x, y }
    }
}

fn die(err: &std::io::Error) {
    Terminal::clear_screen();
    terminal::disable_raw_mode().unwrap();
    panic!("{}", err)
}
