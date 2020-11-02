use std::{
    rc::{Rc, Weak},
    cell::RefCell,
};
use crate::{
    traits::*,
    casters::*,
    internal::InnerDrop,
};
use graphic_cli_derive::*;

/*
for each element
pub fn new(parent: Rc<dyn Parent>, other args) -> Rc<Self> {
convert the parent to a weak reference
if parent is a child, set doc to parent.doc()
else set doc to Rc::clone(parent).downgrade()
}
*/

#[derive(Parent, NotChild)]
pub struct Doc {
    children: Vec<Rc<RefCell<dyn Child>>>,
    id: usize,
    ondrop: InnerDrop,
    // If I need to implement Drop, just add a private unit struct that implements drop
}

/*
impl Doc {
    pub fn new_element<T: Element>(&mut self) -> Rc<T> {

    }
}*/

impl Element for Doc {
    fn id(&self) -> usize {
        self.id
    }
}

impl Parent for Doc {
    fn children(&self) -> &Vec<Rc<RefCell<dyn Child>>> {
        &self.children
    }
}

#[derive(NotParent, Child)]
pub struct TestChild {
    doc: Weak<Doc>,
    parent: Weak<dyn Parent>,
    id: usize,
}

impl Element for TestChild {
    fn id(&self) -> usize {
        self.id
    }
}

impl Child for TestChild {
    fn doc(&self) -> Rc<Doc> {
        self.doc.upgrade().unwrap()
    }
    fn parent(&self) -> Rc<dyn Parent> {
        self.parent.upgrade().unwrap()
    }
}