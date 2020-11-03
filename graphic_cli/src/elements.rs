use std::{
    rc::{Rc, Weak},
    cell::RefCell,
    sync::atomic::AtomicBool,
    io::{Read, Write, stdin, stdout},
    any::TypeId,
};
use crate::{
    traits::*,
    casters::*,
    error::*,
};
use graphic_cli_derive::*;
use crossterm::{
    tty::IsTty,
};

/*
for each element
pub fn new(parent: Rc<dyn Parent>, other args) -> Rc<Self> {
convert the parent to a weak reference
if parent is a child, set doc to parent.doc()
else set doc to Rc::clone(parent).downgrade()
}
*/

// There can be multiple top level elements
// This will be the one for stdin and stdout
#[derive(Parent, NotChild)]
pub struct TTYDoc {
    children: Vec<Rc<RefCell<dyn Child>>>,
    id: usize,
    _ondrop: TTYDocOnDrop,
}

impl TTYDoc {
    const EXISTS: AtomicBool = AtomicBool::new(false);

    pub fn new() -> Result<Rc<RefCell<Self>>> {
        let stdin = stdin();
        let stdout = stdout();
        if !stdin.is_tty() || !stdout.is_tty() {
            Err(ErrorKind::DefaultIONotTTY)
        }
        else {
            Self::new_using(stdin, stdout)
        }
    }

    pub fn new_using<R, W>(input: R, output: W) -> Result<Rc<RefCell<Self>>>
        where R: Read + IsTty,
              W: Write + IsTty
    {
        if *Self::EXISTS.get_mut() {
            return Err(ErrorKind::AlreadyExists(TypeId::of::<Self>()));
        }
        todo!();
    }
}

impl Element for TTYDoc {
    fn id(&self) -> usize {
        self.id
    }
}

impl Parent for TTYDoc {
    fn children(&self) -> &Vec<Rc<RefCell<dyn Child>>> {
        &self.children
    }
}

struct TTYDocOnDrop;

impl Drop for TTYDocOnDrop {
    fn drop(&mut self) {
        *TTYDoc::EXISTS.get_mut() = true;
    }
}

#[derive(NotParent, Child)]
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