use std::{
    //any::type_name,
    error::Error,
    fmt,
    convert::From,
    //marker::PhantomData,
    io,
};

#[derive(Debug)]
pub enum ErrorKind {
    AlreadyExists(&'static str),
    //#[cfg(feature = "tty")]
    NotATTY,
    CrosstermError(crossterm::ErrorKind),
    IOError(io::Error),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ErrorKind::*;

        write!(f, "Graphic CLI Error")?;
        match self {
            AlreadyExists(type_name) => 
                write!(f, "An attempt was made to create a second instance of {}, but only one is allowed to exist at a given time", type_name),
            //#[cfg(feature = "tty")]
            NotATTY => write!(f, "An attempt was made to create a TTY doc using a stream that is not a TTY"),
            CrosstermError(err) => err.fmt(f),
            IOError(err) => err.fmt(f),
        }
    }
}

impl From<crossterm::ErrorKind> for ErrorKind {
    fn from(err: crossterm::ErrorKind) -> Self {
        Self::CrosstermError(err)
    }
}

impl From<io::Error> for ErrorKind {
    fn from(err: io::Error) -> Self {
        Self::IOError(err)
    }
}

impl Error for ErrorKind {}

pub type Result<T> = std::result::Result<T, ErrorKind>;