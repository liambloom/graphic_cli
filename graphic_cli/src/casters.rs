use crate::traits::*;

pub use as_any_min::AsAny;

pub trait AsElement {
    fn as_element(&self) -> &dyn Element;
    fn as_element_mut(&mut self) -> &mut dyn Element;
}

impl<T> AsElement for T
    where T: Element
{
    fn as_element(&self) -> &dyn Element {
        self
    }
    fn as_element_mut(&mut self) -> &mut dyn Element {
        self
    }
}


pub trait AsParent {
    fn is_parent(&self) -> bool;
    fn as_parent(&self) -> Option<&dyn Parent>;
    fn as_parent_mut(&mut self) -> Option<&mut dyn Parent>;
}

pub trait AsChild {
    fn is_child(&self) -> bool;
    fn as_child(&self) -> Option<&dyn Child>;
    fn as_child_mut(&mut self) -> Option<&mut dyn Child>;
}