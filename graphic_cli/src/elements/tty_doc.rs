// There can be multiple top level elements
// This will be the one for stdin and stdout

use std::{
    rc::Rc,
    cell::RefCell,
    sync::{self, atomic::{AtomicBool, Ordering}},
    io::{Read, Write, stdin, stdout, Stdin, Stdout},
    any::type_name,
    fmt,
};
use crate::{
    traits::*,
    casters::*,
    error::*,
    draw::Canvas,
};
use graphic_cli_derive::*;
use crossterm::{
    terminal,
    QueueableCommand,
    ExecutableCommand,
    cursor,
    tty::IsTty,
    style,
};

static TTY_DOC_EXISTS: AtomicBool = AtomicBool::new(false);
// static mut TTY_DOC: sync::Weak<Box<dyn Parent>> = sync::Weak::new();

#[derive(Debug, NotChild)]
pub struct TTYDoc<R, W>
    where R: Read + IsTty + fmt::Debug + 'static,
          W: Write + IsTty + fmt::Debug + 'static 
{
    rows: u16,
    cols: u16,
    children: Vec<Rc<RefCell<dyn Child>>>,
    id: usize,
    input: R,
    output: W,
    canvas: Canvas,
}

impl<R, W> TTYDoc<R, W>
    where R: Read + IsTty + fmt::Debug + 'static,
          W: Write + IsTty + fmt::Debug + 'static 
{
    pub fn new_using(input: R, mut output: W) -> Result<Rc<RefCell<Self>>> {
        if TTY_DOC_EXISTS.compare_and_swap(false, true, Ordering::AcqRel) {
            return Err(ErrorKind::AlreadyExists(type_name::<Self>()));
        }

        if !input.is_tty() || !output.is_tty() {
            return Err(ErrorKind::NotATTY);
        }
        
        let (cols, rows) = terminal::size()?;

        output.execute(cursor::Hide)?;

        Ok(Rc::new(RefCell::new(Self {
            rows,
            cols,
            children: Vec::new(),
            id: Element::get_id(),
            input,
            output,
            canvas: Canvas::new(rows.into(), cols.into()),
        })))
    }

    pub fn draw(&mut self) -> Result<()> {
        self.output.queue(cursor::MoveTo(0, 0))?;
        for row in self.canvas.a.iter() {
            self.output
                .queue(cursor::MoveDown(1))?
                .queue(cursor::MoveToColumn(0))?;
            for cell in row {
                self.output.queue(style::PrintStyledContent(*cell))?;
            }
        }
        self.output.flush()?;
        Ok(())
    }
}

impl<R, W> AsParent for TTYDoc<R, W>
    where R: Read + IsTty + fmt::Debug + 'static,
          W: Write + IsTty + fmt::Debug + 'static 
{
    fn is_parent(&self) -> bool {
        true
    }
    fn as_parent(&self) -> Option<&dyn Parent> {
        Some(self)
    }
    fn as_parent_mut(&mut self) -> Option<&mut dyn Parent> {
        Some(self)
    }
}
impl<R, W> Drop for TTYDoc<R, W>
    where R: Read + IsTty + fmt::Debug + 'static,
        W: Write + IsTty + fmt::Debug + 'static  
{
    fn drop(&mut self) {
        if !self.as_parent().unwrap().safe_to_drop() {
            panic!("Cannot drop parent element {}#{}, one of its children still has multiple strong references", stringify!(#name), self.id());
        }
        
        assert!(TTY_DOC_EXISTS.compare_and_swap(true, false, Ordering::AcqRel));
        self.output.execute(cursor::Show).expect("Failed to re-show cursor");
    }
}

impl TTYDoc<Stdin, Stdout> { 
    pub fn new() -> Result<Rc<RefCell<Self>>> {
        Self::new_using(stdin(), stdout())
    }
}

impl<R, W> Element for TTYDoc<R, W>
    where R: Read + IsTty + fmt::Debug + 'static,
          W: Write + IsTty + fmt::Debug + 'static 
{
    fn id(&self) -> usize {
        self.id
    }
}

impl<R, W> Parent for TTYDoc<R, W>
    where R: Read + IsTty + fmt::Debug + 'static,
          W: Write + IsTty + fmt::Debug + 'static 
{
    fn children(&self) -> &Vec<Rc<RefCell<dyn Child>>> {
        &self.children
    }
}

/*#[derive(Clone, Debug)]
struct TTYDocOnDrop;

impl Drop for TTYDocOnDrop {
    fn drop(&mut self) {
        assert!(TTY_DOC_EXISTS.compare_and_swap(true, false, Ordering::SeqCst));
        let doc = &unsafe { TTY_DOC };
        (**doc.upgrade().unwrap()).as_any().downcast_ref::<TTYDoc>().unwrap().execute(cursor::Show);
    }
}*/