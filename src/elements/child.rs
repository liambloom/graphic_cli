#![allow(unused_variables)]

use std::fmt::{self, Display, Formatter};
use crate::{
    element_traits::*,
    measurement::Unit,
    errors::Result,
};

#[derive(Debug)]
pub struct UnimplementedChild;
impl UnimplementedChild {
    pub fn new(config: UnimplementedChildConfig) -> Self {
        Self
    }
}
impl Element for UnimplementedChild {
    fn doc(&self) -> &dyn Parent {
        unimplemented!()
    }
    fn children(&self) -> Vec<&dyn Child> {
        unimplemented!()
    }
    fn children_mut(&mut self) -> Vec<&mut dyn Child> {
        unimplemented!()
    }
    fn get_width(&self) -> u16 {
        unimplemented!()
    }
    fn get_height(&self) -> u16 {
        unimplemented!()
    }
    fn parent(&self) -> Option<&dyn Parent> {
        unimplemented!()
    }
    fn parent_mut(&self) -> Option<&mut dyn Parent> {
        unimplemented!()
    }
    fn set_width(&mut self, v: Unit) -> Result<()> {
        unimplemented!()
    }
    fn set_height(&mut self, v: Unit) -> Result<()> {
        unimplemented!()
    }
    fn id(&self) -> Option<&str> {
        unimplemented!()
    }
}
impl OptionParent for UnimplementedChild {}
impl Child for UnimplementedChild {}
impl PrivElement for UnimplementedChild {
    fn draw(&self) {
        unimplemented!()
    }
    fn children_owned(&mut self) -> &mut Vec<Box<dyn Child>> {
        unimplemented!()
    }
    fn width_exact(&self) -> f32 {
        unimplemented!()
    }
    fn height_exact(&self) -> f32 {
        unimplemented!()
    }
}
impl Display for UnimplementedChild
{
    fn fmt<'a>(&self, f: &mut Formatter<'a>) -> fmt::Result {
        unimplemented!()
    }
}

#[derive(Default)]
pub struct UnimplementedChildConfig {
    pub children: Vec<Box<dyn Child>>,
}