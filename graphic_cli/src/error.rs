use std::any::TypeId;

#[derive(Debug)]
pub enum ErrorKind {
    IncorrectType(TypeId),
    /*ChildNotFound(String),
    IdAlreadyExists(String),
    NoChildrenAllowed,
    AlreadyExistsFor(std::any::TypeId),
    CrosstermError(crossterm::ErrorKind),*/
}

/*impl From<crossterm::ErrorKind> for ErrorKind {
    fn from(error: crossterm::ErrorKind) -> Self {
        Self::CrosstermError(error) // This does contain other errors
    }
}*/

pub type Result<T> = std::result::Result<T, ErrorKind>;