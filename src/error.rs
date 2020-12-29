// Copyright 2020 Liam Bloom
// SPDX-License-Identifier: Apache-2.0

//! This module is used for error handling.

use std::{
    convert::From, 
    error::Error,
    cell::BorrowError,
    fmt,
    io,
};

/// The kinds of errors possible in this crate
#[derive(Debug)]
pub enum ErrorKind {
    /// An error originating in the `crossterm` crate
    CrosstermError(crossterm::ErrorKind),

    /// An error originating in `std::io`
    IOError(io::Error),

    /// An error originating from borrowing a `RefCell`
    BorrowError,
}

impl Error for ErrorKind {}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ErrorKind::*;

        match self {
            CrosstermError(err) => write!(f, "{}", err),
            IOError(err) => write!(f, "{}", err),
            BorrowError => write!(f, "already mutably borrowed")
        }
    }
}

/// The result type for this crate
pub type Result<T> = std::result::Result<T, ErrorKind>;

impl From<crossterm::ErrorKind> for ErrorKind {
    fn from(err: crossterm::ErrorKind) -> ErrorKind {
        ErrorKind::CrosstermError(err)
    }
}

impl From<io::Error> for ErrorKind {
    fn from(err: io::Error) -> ErrorKind {
        ErrorKind::IOError(err)
    }
}

impl From<BorrowError> for ErrorKind {
    fn from(_: BorrowError) -> ErrorKind {
        ErrorKind::BorrowError
    }
}