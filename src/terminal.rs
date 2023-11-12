use std::io::Error;

use crossterm::terminal::size;

pub struct Size {
    pub width: u16,
    pub height: u16,
}

pub struct Terminal {
    size: Size,
}

impl Terminal {
    pub fn default() -> Result<Self, Error> {
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
}
