// There can be multiple top level elements
// This will be the one for stdin and stdout

use std::{
    rc::Rc,
    cell::RefCell,
    sync::atomic::{AtomicBool, Ordering},
    io::{Read, Write, stdin, stdout},
    any::type_name,
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

static TTY_DOC_EXISTS: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Parent, NotChild)]
pub struct TTYDoc {
    children: Vec<Rc<RefCell<dyn Child>>>,
    id: usize,
    _ondrop: TTYDocOnDrop,
}

impl TTYDoc {
    pub fn new() -> Result<Rc<RefCell<Self>>> {
        Self::new_using(stdin(), stdout())
    }

    pub fn new_using<R, W>(input: R, output: W) -> Result<Rc<RefCell<Self>>>
        where R: Read + IsTty + 'static,
              W: Write + IsTty + 'static
    {
        if TTY_DOC_EXISTS.compare_and_swap(false, true, Ordering::AcqRel) {
            return Err(ErrorKind::AlreadyExists(type_name::<Self>()));
        }

        if !input.is_tty() || !output.is_tty() {
            return Err(ErrorKind::NotATTY);
        }
        
        Ok(Rc::new(RefCell::new(Self {
            children: Vec::new(),
            id: Element::get_id(),
            _ondrop: TTYDocOnDrop,
        })))
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

#[derive(Clone, Debug)]
struct TTYDocOnDrop;

impl Drop for TTYDocOnDrop {
    fn drop(&mut self) {
        assert!(TTY_DOC_EXISTS.compare_and_swap(true, false, Ordering::AcqRel));
    }
}