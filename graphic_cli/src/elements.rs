use std::{
    rc::{Rc, Weak},
    cell::RefCell,
};
use crate::{
    traits::*,
    casters::*,
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

#[cfg(feature = "tty")]
reexport!(tty_doc);

#[derive(Debug, NotParent, Child)]
pub struct TestChild {
    doc: Weak<RefCell<dyn Parent>>,
    parent: Weak<RefCell<dyn Parent>>,
    id: usize,
}

impl TestChild {
    pub fn new(parent: Rc<RefCell<dyn Parent>>) -> Rc<RefCell<Self>> {
        let this = Rc::new(RefCell::new(Self {
            doc: if parent.borrow().is_child() { 
                    Rc::downgrade(&parent.borrow().as_child().unwrap().doc_rc())
                } 
                else { 
                    Rc::downgrade(&parent)
                },
            parent: Rc::downgrade(&parent),
            id: Element::get_id()//*Element::ELEMENT_COUNT.get_mut()
        }));
        this
    }
}

impl Element for TestChild {
    fn id(&self) -> usize {
        self.id
    }
}

impl Child for TestChild {
    fn doc_rc(&self) -> Rc<RefCell<dyn Parent>> {
        self.doc.upgrade().expect(
            "A child attempted to get its top level parent, which no longer existed. This should be impossible"
        )
    }
    fn parent_rc(&self) -> Rc<RefCell<dyn Parent>> {
        self.parent.upgrade().expect(
            "A child attempted to get its parent, which no longer existed. This should be impossible"
        )
    }
}