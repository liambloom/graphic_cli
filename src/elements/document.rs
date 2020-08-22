use std::{
    io,
    fmt::{self, Display, Formatter},
};
use crossterm::terminal::size;
use crate::element_traits::*;

static mut CONSTRUCTED: bool = false;

unsafe fn enforce_once() {
    if CONSTRUCTED {
        panic!("You may not construct a graphic_cli::elements::Document more than once");
    }
    else {
        CONSTRUCTED = true;
    }
}

pub struct Document {
    read: Box<dyn io::Read>,
    write: Box<dyn io::Write>,
    bmp: Vec<Vec<todo!("Make color enum")>>,
    children: Vec<Box<dyn Child>>,
    id: String,
    width: u16, // (1) if self.write is io::Stdout, this should use crossterm::terminal::size() (or maybe crossterm::terminal::SetSize)
    height: u16, // (2) if it's not io::Stdout,
}

impl Document {
    pub unsafe fn new() {
        enforce_once();
    }
}

impl Element for Document {
    fn id(&self) -> &str {
        &self.id
    }
    fn children(&self) -> Vec<&dyn Child> {
        self.children.iter().map(|&child| &*child).collect()
    }
    fn children_mut(&mut self) -> Vec<&mut dyn Child> {
        self.children.iter().map(|child| &mut **child).collect() // Is this allowed? Is box supposed to have interior mutability?
    }
    fn child_count(&self) -> usize {
        self.children.len()
    }
    fn get_width(&self) -> u16 {
        self.width
    }
    fn get_height(&self) -> u16 {
        self.height
    }
}

impl PrivElement for Document {

}

impl Display for Document {

}