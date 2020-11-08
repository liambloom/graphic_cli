// Main is only used for testing (for now), so this is fine
#![allow(unused_imports, dead_code, unused_variables)]

use std::io::stdout;
use crossterm::{*, style::*};
use graphic_cli::elements::*;

fn main() {
    TTYDoc::new().unwrap().borrow_mut().draw();
    
    /*let mut stdout = stdout();
    stdout
        .execute(PrintStyledContent(style('f'))).unwrap()
        .execute(PrintStyledContent(style('o'))).unwrap()
        .execute(PrintStyledContent(style('o'))).unwrap();*/
}