use std::fmt::{self, Display};
use super::*;
use crate::{
    measurement::Unit,
    errors::Result,
};

pub trait Element: fmt::Debug + Display + PrivElement + OptionParent {
    fn doc(&self) -> &dyn Parent; // In order to return the type Document<R, W> it must know R and W, so Element would have to have 2 generic types
    fn get_width(&self) -> u16;
    fn get_height(&self) -> u16;
    // These are not in the OptionParent trait because they can't have an implementation on the Parent trait because they need access to fields
    fn children(&self) -> Vec<&dyn Child>;
    fn children_mut(&mut self) -> Vec<&mut dyn Child>;
    fn set_width(&mut self, v: Unit) -> Result<()>;
    fn set_height(&mut self, v: Unit) -> Result<()>;
    fn parent(&self) -> Option<&dyn Parent> {
        None
    }
    fn parent_mut(&self) -> Option<&mut dyn Parent> {
        None
    }
    fn id(&self) -> Option<&str> {
        None
    }
}

pub trait PrivElement {
    fn draw(&self);
    fn children_owned(&mut self) -> &mut Vec<Box<dyn Child>>;
    fn width_exact(&self) -> f32;
    fn height_exact(&self) -> f32;
}