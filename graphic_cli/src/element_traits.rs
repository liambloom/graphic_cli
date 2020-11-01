use std::{
    rc::Rc,
    cell::RefCell,
    sync::atomic::AtomicUsize,
};
use crate::{
    elements::*,
};
use as_any_min::AsAny;

pub trait Element: AsElement + AsParent + AsChild + AsAny + 'static {
    fn id(&self) -> usize;
}

impl dyn Element {
    const ELEMENT_COUNT: AtomicUsize = AtomicUsize::new(0);
}

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

pub trait Parent: Element {
    fn children(&self) -> &Vec<Rc<RefCell<dyn Child>>>;
}

impl dyn Parent {
    pub fn get_child(&self, id: usize) -> Option<Rc<RefCell<dyn Child>>> {
        for child in self.children().iter() {
            let child_ref = child.borrow();
            if child_ref.id() == id {
                return Some(Rc::clone(child))
            }
            else {
                if let Some(child) = child_ref.as_parent() {
                    let sub_child = child.get_child(id);
                    if sub_child.is_some() {
                        return sub_child;
                    }
                }
            }
        }
        None
    }
}

pub trait AsChild {
    fn is_child(&self) -> bool;
    fn as_child(&self) -> Option<&dyn Child>;
    fn as_child_mut(&mut self) -> Option<&mut dyn Child>;
}

pub trait Child: Element {
    fn doc(&self) -> Rc<Doc>;
    fn parent(&self) -> Rc<dyn Parent>;
}