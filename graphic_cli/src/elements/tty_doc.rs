// There can be multiple top level elements
// This will be the one for stdin and stdout

use std::{
    rc::Rc,
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