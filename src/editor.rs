use std::io::{Error, self, Write};

use crossterm::{
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal, ExecutableCommand, execute, cursor,
};

pub struct Editor {
    should_quit: bool,
}

impl Editor {
    pub fn default() -> Self {
        Self { should_quit: false }
    }

    pub fn run(&mut self) {
        terminal::enable_raw_mode().unwrap();

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
        let mut stdout = io::stdout();
        execute!(stdout, terminal::Clear(terminal::ClearType::All), cursor::MoveTo(0, 0))?;
        if self.should_quit {
            println!("Goodbye and thanks for all the fish!\r");
        } else {
            self.draw_rows();
            execute!(stdout, cursor::MoveTo(0, 0))?;
        }
        stdout.flush()
    }

    fn draw_rows(&self) {
        for _ in 0..24 {
            println!("~\r");
        }
    }

    fn process_keypress(&mut self) -> Result<(), Error> {
        let pressed_key = read_key()?;
        match (pressed_key.modifiers, pressed_key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('q')) => self.should_quit = true,
            _ => (),
        }
        Ok(())
    }
}

fn read_key() -> Result<KeyEvent, Error> {
    loop {
        if let Ok(Event::Key(pressed_key)) = read() {
            return Ok(pressed_key);
        }
    }
}

fn die(err: &std::io::Error) {
    panic!("{}", err)
}
