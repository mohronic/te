use std::io::{self, Error, Write};

use crossterm::{
    cursor,
    event::{read, Event, KeyEvent},
    terminal::{self, size},
    ExecutableCommand,
};

use crate::editor::Position;

pub struct Size {
    pub width: u16,
    pub height: u16,
}

pub struct Terminal {
    size: Size,
}

impl Terminal {
    pub fn default() -> Result<Self, Error> {
        terminal::enable_raw_mode()?;
        let (columns, rows) = size()?;
        Ok(Self {
            size: Size {
                width: columns,
                height: rows,
            },
        })
    }

    pub fn size(&self) -> &Size {
        &self.size
    }

    pub fn clear_screen() {
        io::stdout()
            .execute(terminal::Clear(terminal::ClearType::All))
            .expect("failed to clear screen");
    }

    pub fn clear_current_line() {
        io::stdout()
            .execute(terminal::Clear(terminal::ClearType::CurrentLine))
            .expect("failed to clear screen");
    }

    pub fn cursor_position(position: &Position) {
        let x = position.x as u16;
        let y = position.y as u16;
        io::stdout()
            .execute(cursor::MoveTo(x, y))
            .expect("failed to move cursor");
    }

    pub fn hide_cursor() {
        io::stdout()
            .execute(cursor::Hide)
            .expect("failed to hide cursor");
    }

    pub fn show_cursor() {
        io::stdout()
            .execute(cursor::Show)
            .expect("failed to show cursor");
    }

    pub fn flush() -> Result<(), Error> {
        io::stdout().flush()
    }

    pub fn read_key() -> Result<KeyEvent, Error> {
        loop {
            if let Ok(Event::Key(pressed_key)) = read() {
                return Ok(pressed_key);
            }
        }
    }
}
