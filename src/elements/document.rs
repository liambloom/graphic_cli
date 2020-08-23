use std::{
    io::{self, Read, Write, Seek, SeekFrom, Stdout},
    sync::atomic::AtomicBool,
    convert::TryInto,
};
use crossterm::{
    execute,
    terminal::size,
    cursor::MoveTo,
    ExecutableCommand,
};
use crate::element_traits::*;

static mut STDOUT_DOC_EXISTS: AtomicBool = AtomicBool::new(false);
static mut STDERR_DOC_EXISTS: AtomicBool = AtomicBool::new(false);

fn enforce_once()/* -> Result<(), >*/ {
    unsafe {
        if *STDOUT_DOC_EXISTS.get_mut() {
            panic!("You may not construct a graphic_cli::elements::Document more than once");
        }
        else {
            *STDOUT_DOC_EXISTS.get_mut() = true;
        }
    }
}

pub struct Document<R, W>
    where R: Read,
          W: Write + Seek
{
    read: R,
    write: W,
    bmp: Vec<Vec<crate::colors::RGB>>,
    children: Vec<Box<dyn Child>>,
    id: String,
    width: u16, // (1) if self.write is io::Stdout, this should use crossterm::terminal::size() (or maybe crossterm::terminal::SetSize)
    height: u16, // (2) if it's not io::Stdout,
}

impl<R, W> Document<R, W>
    where R: Read,
          W: Write + Seek
{
    pub unsafe fn new() {
        enforce_once();
    }
}

struct SeekStdout {
    stdout: Stdout,
}
impl SeekStdout {
    fn new() -> Self {
        Self { stdout: io::stdout() }
    }
    fn from(stdout: Stdout) -> Self {
        Self { stdout }
    }
}
impl Write for SeekStdout {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stdout.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.stdout.flush()
    }
}
impl Seek for SeekStdout {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let columns: u16 = match size() {
            Ok(size) => size.0,
            Err(err) => return Err(io::Error::new(io::ErrorKind::Other, err.to_string())),
        };
        match pos {
            SeekFrom::Start(p) => {
                let p_u16: u16 = match p.try_into() {
                    Ok(n) => n,
                    Err(err) => return Err(io::Error::new(io::ErrorKind::InvalidInput, err.to_string())),
                };
                match self.stdout.execute(MoveTo(p_u16 / columns, p_u16 / columns)) {
                    Ok(_) => Ok(p),
                    Err(err) => Err(io::Error::new(io::ErrorKind::Other, err.to_string())),
                }
            }
        }
    }
}

/*impl<R, W> Element for Document<R, W>
    where R: io::Read,
          W: io::Write
{
    fn id(&self) -> &str {
        &self.id
    }
    fn children(&self) -> Vec<&dyn Child> {
        self.children.iter().map(|&child| &*child).collect()
    }
    fn children_mut(&mut self) -> Vec<&mut dyn Child> {
        self.children.iter().map(|child| &mut **child).collect() // Is this allowed? Is box supposed to have interior mutability?
    }
    fn child_count(&self) -> usize {
        self.children.len()
    }
    fn get_width(&self) -> u16 {
        self.width
    }
    fn get_height(&self) -> u16 {
        self.height
    }
}

impl PrivElement for Document {

}

impl Display for Document {

}*/