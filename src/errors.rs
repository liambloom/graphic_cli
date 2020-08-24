//use std::any::TypeId;
//use crossterm::ErrorKind;

pub enum Error {
    ChildNotFound(String),
    IdAlreadyExists(String),
    AlreadyExistsFor(std::any::TypeId),
    CrosstermError(crossterm::ErrorKind),
}

impl From<crossterm::ErrorKind> for Error {
    fn from(error: crossterm::ErrorKind) -> Self {
        Self::CrosstermError(error) // This does contain other errors
    }
}