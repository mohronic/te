use std::{
    env,
    io::Error,
    time::{Duration, Instant},
};

use crossterm::{
    event::{KeyCode, KeyEventKind, KeyModifiers},
    style::Color,
    terminal,
};

use crate::{Document, Row, Terminal};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const QUIT_TIMES: u8 = 1;
const STATUS_BG_COLOR: Color = Color::Rgb {
    r: 40,
    g: 39,
    b: 38,
};
const STATUS_FG_COLOR: Color = Color::Rgb {
    r: 135,
    g: 133,
    b: 128,
};
const MESSAGE_BG_COLOR: Color = Color::Rgb {
    r: 64,
    g: 62,
    b: 60,
};

#[derive(Default)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

struct StatusMessage {
    text: String,
    time: Instant,
}

impl StatusMessage {
    fn from(message: String) -> Self {
        Self {
            text: message,
            time: Instant::now(),
        }
    }
}

pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
    cursor_position: Position,
    offset: Position,
    document: Document,
    status_message: StatusMessage,
    quit_times: u8,
}

impl Editor {
    pub fn default() -> Self {
        let args: Vec<String> = env::args().collect();
        let mut initial_status = String::from("HELP: Ctrl-S = save | Ctrl-Q = quit");
        let document = if args.len() > 1 {
            let file_name = &args[1];
            let doc = Document::open(file_name);
            if doc.is_ok() {
                doc.unwrap()
            } else {
                initial_status = format!("ERR: Could not open file: {}", file_name);
                Document::default()
            }
        } else {
            Document::default()
        };

        Self {
            should_quit: false,
            terminal: Terminal::default().expect("Failed to initialize terminal"),
            cursor_position: Position::default(),
            offset: Position::default(),
            document,
            status_message: StatusMessage::from(initial_status),
            quit_times: QUIT_TIMES,
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
            Terminal::reset_color();
            Terminal::clear_screen();
            println!("Goodbye and thanks for all the fish!\r");
        } else {
            self.draw_rows();
            self.draw_status_bar();
            self.draw_message_bar();
            Terminal::cursor_position(&Position {
                x: self.cursor_position.x.saturating_sub(self.offset.x),
                y: self.cursor_position.y.saturating_sub(self.offset.y),
            });
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
        for terminal_row in 0..height {
            Terminal::clear_current_line();

            if let Some(row) = self.document.row(terminal_row as usize + self.offset.y) {
                self.draw_row(row);
            } else if self.document.is_empty() && terminal_row == height / 3 {
                self.draw_welcome_message();
            } else {
                println!("~\r");
            }
        }
    }

    fn draw_row(&self, row: &Row) {
        let start = self.offset.x;
        let width = self.terminal.size().width as usize;
        let end = self.offset.x + width;
        let row = row.render(start, end);
        println!("{}\r", row)
    }

    fn process_keypress(&mut self) -> Result<(), Error> {
        let pressed_key = Terminal::read_key()?;
        if pressed_key.kind == KeyEventKind::Release {
            return Ok(());
        }
        match (pressed_key.modifiers, pressed_key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('q')) => {
                if self.quit_times > 0 && self.document.is_dirty() {
                    self.status_message = StatusMessage::from(format!(
                        "WARNING! File has unsaved changes. Press Ctrl-Q {} more times to quit",
                        self.quit_times
                    ));
                    self.quit_times -= 1;
                    return Ok(());
                }
                self.should_quit = true
            }
            (KeyModifiers::CONTROL, KeyCode::Char('s')) => self.save(),
            (_, KeyCode::Char(c)) => {
                self.document.insert(&self.cursor_position, c);
                self.move_cursor(KeyCode::Right);
            }
            (KeyModifiers::NONE, KeyCode::Delete) => self.document.delete(&self.cursor_position),
            (KeyModifiers::NONE, KeyCode::Backspace) => {
                if self.cursor_position.x > 0 || self.cursor_position.y > 0 {
                    self.move_cursor(KeyCode::Left);
                    self.document.delete(&self.cursor_position);
                }
            }
            (KeyModifiers::NONE, KeyCode::Enter) => {
                self.document.insert_newline(&self.cursor_position);
                self.move_cursor(KeyCode::Down);
                self.move_cursor(KeyCode::Home);
            }
            (KeyModifiers::NONE, KeyCode::Up)
            | (KeyModifiers::NONE, KeyCode::Down)
            | (KeyModifiers::NONE, KeyCode::Left)
            | (KeyModifiers::NONE, KeyCode::Right)
            | (KeyModifiers::NONE, KeyCode::PageUp)
            | (KeyModifiers::NONE, KeyCode::PageDown)
            | (KeyModifiers::NONE, KeyCode::End)
            | (KeyModifiers::NONE, KeyCode::Home) => self.move_cursor(pressed_key.code),
            _ => (),
        }
        self.scroll();
        if self.quit_times < QUIT_TIMES {
            self.quit_times = QUIT_TIMES;
            self.status_message = StatusMessage::from(String::new());
        }
        Ok(())
    }

    fn move_cursor(&mut self, key_code: KeyCode) {
        let Position { mut x, mut y } = self.cursor_position;
        let terminal_height = self.terminal.size().height as usize;
        let height = self.document.len() as usize;
        let mut width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };
        match key_code {
            KeyCode::Up => y = y.saturating_sub(1),
            KeyCode::Down => {
                if y < height {
                    y = y.saturating_add(1)
                }
            }
            KeyCode::Left => {
                if x > 0 {
                    x -= 1
                } else if y > 0 {
                    y -= 1;
                    if let Some(row) = self.document.row(y) {
                        x = row.len();
                    } else {
                        x = 0;
                    }
                }
            }
            KeyCode::Right => {
                if x < width {
                    x += 1;
                } else if y < height {
                    y += 1;
                    x = 0;
                }
            }
            KeyCode::PageUp => {
                y = if y > terminal_height {
                    y - terminal_height
                } else {
                    0
                }
            }
            KeyCode::PageDown => {
                y = if y.saturating_add(terminal_height) < height {
                    y + terminal_height as usize
                } else {
                    height
                }
            }
            KeyCode::End => x = width,
            KeyCode::Home => x = 0,
            _ => (),
        };

        width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };
        if x > width {
            x = width;
        }

        self.cursor_position = Position { x, y }
    }

    fn scroll(&mut self) {
        let Position { x, y } = self.cursor_position;
        let width = self.terminal.size().width as usize;
        let height = self.terminal.size().height as usize;
        let offset = &mut self.offset;

        if y < offset.y {
            offset.y = y;
        } else if y >= offset.y.saturating_add(height) {
            offset.y = y.saturating_sub(height).saturating_add(1);
        }

        if x < offset.x {
            offset.x = x;
        } else if x >= offset.x.saturating_add(width) {
            offset.x = x.saturating_sub(width).saturating_add(1);
        }
    }

    fn draw_status_bar(&self) {
        let mut status;
        let width = self.terminal.size().width as usize;
        let modified_indicator = if self.document.is_dirty() {
            " (modified)"
        } else {
            ""
        };
        let mut file_name = "[No Name]".to_string();
        if let Some(name) = &self.document.file_name {
            file_name = name.clone();
            file_name.truncate(20);
        }
        status = format!(
            "{} - {} lines{}",
            file_name,
            self.document.len(),
            modified_indicator
        );
        let line_indicator = format!(
            "{}/{}",
            self.cursor_position.y.saturating_add(1),
            self.document.len()
        );
        let len = status.len() + line_indicator.len();
        if width > len {
            status.push_str(&" ".repeat(width - len));
        }
        status = format!("{}{}", status, line_indicator);
        status.truncate(width);

        Terminal::set_bg_color(STATUS_BG_COLOR);
        Terminal::set_fg_color(STATUS_FG_COLOR);
        println!("{}\r", status);
        Terminal::reset_bg_color();
        Terminal::reset_fg_color();
    }

    fn draw_message_bar(&self) {
        Terminal::set_bg_color(MESSAGE_BG_COLOR);
        Terminal::clear_current_line();
        let message = &self.status_message;
        if Instant::now() - message.time < Duration::new(5, 0) {
            let mut text = message.text.clone();
            text.truncate(self.terminal.size().width as usize);
            print!("{}", text);
        }
        Terminal::reset_bg_color();
    }

    fn save(&mut self) {
        if self.document.file_name.is_none() {
            let new_name = self.promt("Save as: ").unwrap_or(None);
            if new_name.is_none() {
                self.status_message = StatusMessage::from("Save aborted.".to_string());
                return;
            }
            self.document.file_name = new_name;
        }

        if self.document.save().is_ok() {
            self.status_message = StatusMessage::from("File saved successfully".to_string());
        } else {
            self.status_message = StatusMessage::from("Error writing file".to_string());
        }
    }

    fn promt(&mut self, prompt: &str) -> Result<Option<String>, Error> {
        let mut result = String::new();

        loop {
            self.status_message = StatusMessage::from(format!("{}{}", prompt, result));
            self.refresh_screen()?;
            let pressed_key = Terminal::read_key()?;
            if pressed_key.kind == KeyEventKind::Release {
                continue;
            }
            match pressed_key.code {
                KeyCode::Enter => break,
                KeyCode::Backspace => {
                    if !result.is_empty() {
                        result.truncate(result.len() - 1);
                    }
                }
                KeyCode::Esc => {
                    result.truncate(0);
                    break;
                }
                KeyCode::Char(c) => {
                    if !c.is_control() {
                        result.push(c);
                    }
                }
                _ => (),
            }
        }
        self.status_message = StatusMessage::from(String::new());
        if result.is_empty() {
            return Ok(None);
        }
        Ok(Some(result))
    }
}

fn die(err: &std::io::Error) {
    Terminal::clear_screen();
    terminal::disable_raw_mode().unwrap();
    panic!("{}", err)
}
