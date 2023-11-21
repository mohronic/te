use std::io::{self, Error, Write};

use crossterm::{
    cursor,
    event::{read, Event, KeyEvent},
    style::{Color, SetBackgroundColor, SetForegroundColor, ResetColor},
    terminal::{self, size},
    ExecutableCommand,
};

use crate::editor::Position;

const EDITOR_BG_COLOR: Color = Color::Rgb {
    r: 16,
    g: 15,
    b: 15,
};
const TEXT_COLOR: Color = Color::Rgb {
    r: 206,
    g: 205,
    b: 195,
};

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
        Terminal::reset_bg_color();
        Terminal::set_fg_color(TEXT_COLOR);
        let (columns, rows) = size()?;
        Ok(Self {
            size: Size {
                width: columns,
                height: rows.saturating_sub(2),
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

    pub fn set_bg_color(color: Color) {
        io::stdout()
            .execute(SetBackgroundColor(color))
            .expect("failed to set bg color");
    }

    pub fn reset_bg_color() {
        io::stdout()
            .execute(SetBackgroundColor(EDITOR_BG_COLOR))
            .expect("failed to reset bg color");
    }

    pub fn set_fg_color(color: Color) {
        io::stdout()
            .execute(SetForegroundColor(color))
            .expect("failed to set fg color");
    }

    pub fn reset_color() {
        io::stdout()
            .execute(ResetColor)
            .expect("failed to reset color");
    }
}
