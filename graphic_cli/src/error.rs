use std::{
    //any::type_name,
    error::Error,
    fmt,
    //marker::PhantomData,
};
//use crate::traits::Element;

/*#[derive(Debug)]
struct AlreadyExistsError<T>
    where T: Element
{
    data: PhantomData<T>,
}

impl<T> fmt::Display for AlreadyExistsError<T>
    where T: Element
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "There can only be one element of type {}", type_name::<T>())
    }
}

impl<T> Error for AlreadyExistsError<T>
    where T: Element {}

#[cfg(feature = "tty")]
mod tty_errors {
    use crossterm::tty::IsTty;

    #[derive(Debug)]
    struct NotTTYError<T>
        where T: IsTty
    {
        stream: T
    }

    impl<T> Display for NotTTYError<T>
        where T: IsTty
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{} is not a TTY", stream)
        }
    }

    impl<T> Error for NotTTYError<T>
        where T: IsTty {}
}
#[cfg(feature = "tty")]
pub use tty_errors::*;*/

#[derive(Debug)]
pub enum ErrorKind {
    AlreadyExists(&'static str),
    #[cfg(feature = "tty")]
    NotATTY,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ErrorKind::*;

        write!(f, "Graphic CLI Error")?;
        match self {
            AlreadyExists(type_name) => 
                write!(f, "An attempt was made to create a second instance of {}, but only one is allowed to exist at a given time", type_name),
            #[cfg(feature = "tty")]
            NotATTY => write!(f, "An attempt was made to create a TTY doc using a stream that is not a TTY"),
        }
    }
}

impl Error for ErrorKind {}

pub type Result<T> = std::result::Result<T, ErrorKind>;