//use std::any::TypeId;
//use crossterm::ErrorKind;

#[derive(Debug)]
pub enum ErrorKind {
    ChildNotFound(String),
    IdAlreadyExists(String),
    NoChildrenAllowed,
    AlreadyExistsFor(std::any::TypeId),
    CrosstermError(crossterm::ErrorKind),
}

impl From<crossterm::ErrorKind> for ErrorKind {
    fn from(error: crossterm::ErrorKind) -> Self {
        Self::CrosstermError(error) // This does contain other errors
    }
}