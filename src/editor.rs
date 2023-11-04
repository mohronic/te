use crossterm::{terminal, event::{read, Event, KeyModifiers, KeyCode}};

pub struct Editor {}

impl Editor {
    pub fn run(&self) {
        terminal::enable_raw_mode().unwrap();

        loop {
            let event = read();
            match event {
                Ok(Event::Key(pressed_key)) => match (pressed_key.modifiers, pressed_key.code) {
                    (KeyModifiers::CONTROL, KeyCode::Char('q')) => break,
                    (_, KeyCode::Char(c)) => {
                        if c.is_control() {
                            println!("{:#b} \r", c as u8);
                        } else {
                            println!("{:#b} ({}) \r", c as u8, c);
                        }
                    }
                    _ => continue,
                },
                Ok(_) => continue,
                Err(e) => die(&e),
            }
        }

        terminal::disable_raw_mode().unwrap();
    }

    pub fn default() -> Self {
        Self {  }
    }

}

fn die(err: &std::io::Error) {
    panic!("{}", err)
}
