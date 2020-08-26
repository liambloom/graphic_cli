#![allow(unused_variables)]

use crate::element_traits::*;
use crate::measurement::Unit;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub struct UnimplementedChild;
impl UnimplementedChild {
    pub fn new(config: UnimplementedChildConfig) -> Self {
        Self
    }
}
impl Child for UnimplementedChild {
    fn parent(&self) -> &dyn Element {
        unimplemented!()
    }
    fn parent_mut(&self) -> &mut dyn Element {
        unimplemented!()
    }
    fn set_width(&mut self, v: Unit) {
        unimplemented!()
    }
    fn set_height(&mut self, v: Unit) {
        unimplemented!()
    }
    fn id(&self) -> &str {
        unimplemented!()
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
}
impl OptionParent for UnimplementedChild {}
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