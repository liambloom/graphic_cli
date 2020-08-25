use std::fmt::{self, Display};
use crate::errors::ErrorKind;
use super::*;

pub trait Element: fmt::Debug + Display + PrivElement {
    fn doc(&self) -> &dyn Parent; // In order to return the type Document<R, W> it must know R and W, so Element would have to have 2 generic types
    fn get_width(&self) -> u16;
    fn get_height(&self) -> u16;
}

pub trait PrivElement {
    fn draw(&self);
    fn children_owned(&mut self) -> &mut Vec<Box<dyn Child>>;
    fn width_exact(&self) -> f32;
    fn height_exact(&self) -> f32;
}