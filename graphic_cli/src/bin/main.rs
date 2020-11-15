// Main is only used for testing (for now), so this is fine
#![allow(unused_imports, dead_code, unused_variables)]

use std::io::stdout;
use crossterm::{*, style::*};
use graphic_cli::elements::*;
//use terminal_size::{Width, Height, terminal_size};
use crossterm::terminal;
#[cfg(unix)]
use libc::winsize;

fn main() {
    //TTYDoc::new().unwrap().borrow_mut().draw();
    //std::thread::sleep(std::time::Duration::from_millis(5000))
    /*let (width, height) = terminal_size().unwrap();
    println!("width: {:?}, height: {:?}", width, height);*/
    let (width, height) = terminal::size().unwrap();
    println!("width: {}, height: {}", width, height);
    /*let mut stdout = stdout();
    stdout
        .execute(PrintStyledContent(style('f'))).unwrap()
        .execute(PrintStyledContent(style('o'))).unwrap()
        .execute(PrintStyledContent(style('o'))).unwrap();*/
}