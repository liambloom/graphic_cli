use std::{
    rc::Rc,
    cell::RefCell,
    sync::atomic::AtomicUsize,
};
use crate::{
    elements::*,
    casters::*,
};

pub trait Element: AsElement + AsParent + AsChild + AsAny + 'static {
    fn id(&self) -> usize;
}

impl dyn Element {
    const ELEMENT_COUNT: AtomicUsize = AtomicUsize::new(0);
}

pub trait Parent: Element {
    fn children(&self) -> &Vec<Rc<RefCell<dyn Child>>>;
}

impl dyn Parent {
    pub fn get_child(&self, id: usize) -> Option<Rc<RefCell<dyn Child>>> {
        for child in self.children() {
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

    pub fn safe_to_drop(&self) -> bool {
        for child in self.children() {
            if Rc::strong_count(child) > 1 {
                return false;
            }
            if let Some(child) = child.borrow().as_parent() {
                if child.safe_to_drop() {
                    return false;
                }
            }
        }
        true
    }
}

pub trait Child: Element {
    fn doc(&self) -> Rc<Doc>;
    fn parent(&self) -> Rc<dyn Parent>;
}