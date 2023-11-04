use crossterm::terminal;
use std::io::{self, Read};

fn main() -> io::Result<()> {
    terminal::enable_raw_mode()?;

    println!("Hello, world!");

    for b in io::stdin().bytes() {
        match b {
            Ok(b) => {
                let c = b as char;
                if c.is_control() {
                    println!("{:#b} \r", b);
                } else {
                    println!("{:#b} ({}) \r", b, c);
                }
                if b == to_ctrl_byte(c) {
                    break;
                }
            }
            Err(e) => die(e),
        }
    }

    return Ok(());
}

fn to_ctrl_byte(c: char) -> u8 {
    let byte = c as u8;
    byte & 0b0001_1111
}

fn die(err: std::io::Error) {
    panic!("{}", err)
}
