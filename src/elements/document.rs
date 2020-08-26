use std::{
    io::{self, Read, Write, Seek, SeekFrom, Stdout, Stdin},
    sync::atomic::AtomicBool,
    convert::TryInto,
    any::{Any, TypeId},
    ops::Drop,
    fmt::{self, Display, Formatter}
};
use crossterm::{
    terminal::size,
    cursor::{position, MoveTo},
    ExecutableCommand,
};
use crate::{
    element_traits::{*},
    ErrorKind,
};

static mut STDOUT_DOC_EXISTS: AtomicBool = AtomicBool::new(false);
//static mut STDERR_DOC_EXISTS: AtomicBool = AtomicBool::new(false);

fn enforce_once<T: 'static>() -> Result<()/*TypeId*/, ErrorKind> {
    let t = TypeId::of::<T>();
    let exists = unsafe { 
        if t == TypeId::of::<SeekStdout>() {
            STDOUT_DOC_EXISTS.get_mut()
        }
        /*else if t == TypeId::of::<Stdout>() {
            STDERR_DOC_EXISTS.get_mut()
        }*/
        else {
            return Ok(())
        }
    };
    if &mut true == exists {
        Err(ErrorKind::AlreadyExistsFor(t))
    }
    else {
        *exists = true;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Document<R, W>
    where R: Read + fmt::Debug,
          W: Write + Seek + Any + fmt::Debug
{
    read: R,
    write: W,
    bmp: Vec<Vec<crate::colors::RGB>>,
    children: Vec<Box<dyn Child>>,
    width: u16, // (1) if self.write is io::Stdout, this should use crossterm::terminal::size() (or maybe crossterm::terminal::SetSize)
    height: u16, // (2) if it's not io::Stdout,
}

impl<R, W> Document<R, W>
    where R: Read + fmt::Debug,
          W: Write + Seek + Any + fmt::Debug
{
    pub fn default() -> Result<Document<Stdin, SeekStdout>, ErrorKind> {
        enforce_once::<W>()?;
        let size = size()?;
        Ok(Document {
            read: io::stdin(),
            write: SeekStdout::new(),
            bmp: Vec::new(), // This won't work
            children: Vec::new(),
            width: size.0,
            height: size.1,
        })
    }
    pub fn new(config: DocumentConfig<R, W>) -> Result<Self, ErrorKind> {
        enforce_once::<W>()?;
        Ok(Self {
            read: config.read,
            write: config.write,
            bmp: config.bmp, // This won't work
            children: config.children,
            width: config.width,
            height: config.height,
        })
    }
}

impl<R, W> Drop for Document<R, W>
    where R: Read + fmt::Debug,
          W: Write + Seek + Any + fmt::Debug
{
    fn drop(&mut self) {
        if TypeId::of::<W>() == TypeId::of::<SeekStdout>() {
            unsafe {
                *STDOUT_DOC_EXISTS.get_mut() = false;
            }
        }
    }
}

#[derive(Default)]
pub struct DocumentConfig<R, W>
    where R: Read + fmt::Debug,
          W: Write + Seek + Any + fmt::Debug
{
    pub read: R,
    pub write: W,
    pub bmp: Vec<Vec<crate::colors::RGB>>,
    pub children: Vec<Box<dyn Child>>,
    pub width: u16, // (1) if self.write is io::Stdout, this should use crossterm::terminal::size() (or maybe crossterm::terminal::SetSize)
    pub height: u16, // (2) if it's not io::Stdout,
}

impl Default for DocumentConfig<Stdin, SeekStdout>
{
    fn default() -> Self {
        let size = size().unwrap();
        Self {
            read: io::stdin(),
            write: SeekStdout::new(),
            bmp: Vec::new(), // This won't work
            children: Vec::new(),
            width: size.0,
            height: size.1,
        }
    }
}

#[derive(Debug)]
pub struct SeekStdout {
    stdout: Stdout,
}
impl SeekStdout {
    pub fn new() -> Self {
        Self { stdout: io::stdout() }
    }
    pub fn from(stdout: Stdout) -> Self {
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
        let size = unwrap_io(size())?;
        match pos {
            SeekFrom::Start(p) => move_to(&mut self.stdout, to_u16(p)?, size.0),
            SeekFrom::End(p) => move_to(&mut self.stdout, to_u16(p)? + size.0 * size.1, size.0),
            SeekFrom::Current(p) => {
                let current = unwrap_io(position())?;
                move_to(&mut self.stdout, to_u16(p)? + size.0 * (current.1 - 1) + current.0, size.0)
            }
        }
    }
}

fn to_u16<T, E>(n: T) -> Result<u16, io::Error> 
    where T: TryInto<u16, Error = E>,
          E: ToString
{
    match n.try_into() {
        Ok(n) => Ok(n),
        Err(err) => Err(io::Error::new(io::ErrorKind::InvalidInput, err.to_string())),
    }
}

fn unwrap_io<T, E: ToString>(r: Result<T, E>) -> io::Result<T> {
    match r {
        Ok(val) => Ok(val),
        Err(err) => Err(io::Error::new(io::ErrorKind::Other, err.to_string())),
    }
}

fn move_to(stdout: &mut Stdout, to: u16, cols: u16) -> Result<u64, io::Error> {
    match stdout.execute(MoveTo(to % cols, to / cols)) {
        Ok(_) => Ok(to.into()),
        Err(err) => Err(io::Error::new(io::ErrorKind::Other, err.to_string())),
    }
}

impl<R, W> Element for Document<R, W>
    where R: Read + fmt::Debug,
          W: Write + Seek + Any + fmt::Debug
{
    fn doc(&self) -> &dyn Parent {
        self
    }
    fn children(&self) -> Vec<&dyn Child> {
        todo!();
        //self.children.iter().map(|&child| &*child).collect()
    }
    fn children_mut(&mut self) -> Vec<&mut dyn Child> {
        todo!();
        //self.children.iter().map(|child| &mut **child).collect() // Is this allowed? Is box supposed to have interior mutability?
    }
    fn get_width(&self) -> u16 {
        self.width
    }
    fn get_height(&self) -> u16 {
        self.height
    }
}

impl<R, W> Parent for Document<R, W>
    where R: Read + fmt::Debug,
          W: Write + Seek + Any + fmt::Debug {}

impl<R, W> PrivElement for Document<R, W>
    where R: Read + fmt::Debug,
          W: Write + Seek + Any + fmt::Debug
{
    fn draw(&self) {
        todo!();
    }
    fn children_owned(&mut self) -> &mut Vec<Box<dyn Child>> {
        todo!();
    }
    fn width_exact(&self) -> f32 {
        todo!();
    }
    fn height_exact(&self) -> f32 {
        todo!();
    }
}

impl<R, W> Display for Document<R, W>
    where R: Read + fmt::Debug,
          W: Write + Seek + Any + fmt::Debug
{
    fn fmt<'a>(&self, _f: &mut Formatter<'a>) -> fmt::Result {
        todo!()
    }
}