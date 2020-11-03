use std::{
    rc::Rc,
    cell::{RefCell/*, Ref, RefMut*/},
    sync::atomic::{AtomicUsize, Ordering},
};
use crate::{
    // elements::*,
    casters::*,
};

pub trait Element: AsElement + AsParent + AsChild + AsAny + 'static {
    fn id(&self) -> usize;
}

impl dyn Element {
    const ELEMENT_COUNT: AtomicUsize = AtomicUsize::new(0);

    pub fn get_id() -> usize {
        Element::ELEMENT_COUNT.fetch_add(1, Ordering::Relaxed)
    }
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
    /*fn doc(&self) -> Ref<dyn Parent> {
        (*self.doc_rc()).borrow()
    }
    fn doc_mut(&self) -> RefMut<dyn Parent> {
        self.doc_rc().borrow_mut()
    }*/
    fn doc_rc(&self) -> Rc<RefCell<dyn Parent>>;

    /*fn parent(&self) -> Ref<dyn Parent> {
        self.parent_rc().borrow()
    }
    fn parent_mut(&self) -> RefMut<dyn Parent> {
        self.parent_rc().borrow_mut()
    }*/
    fn parent_rc(&self) -> Rc<RefCell<dyn Parent>>;
}