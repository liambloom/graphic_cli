use std::{
    sync::atomic::AtomicUsize, 
    convert::From, 
    error::Error, 
    fmt, 
    io::{self, stdin, stdout, stderr, Write},
    ops::Deref, 
    sync::{
        PoisonError, TryLockError, 
        atomic::{AtomicBool, Ordering}, 
    },
    rc::Rc,
    cell::{RefCell, BorrowError},
};
use crossterm::{
    QueueableCommand, ExecutableCommand,
    tty::IsTty,
    style::{ContentStyle, StyledContent, PrintStyledContent},
    terminal, cursor,
};

static CANVAS_ONCE: AtomicBool = AtomicBool::new(true);

#[derive(Clone, Debug)]
pub struct Canvas {
    pub layers: Vec<Layer>,
    changed: Rc<RefCell<Vec<bool>>>,
}

impl Canvas {
    pub fn new() -> Self {
        let mut out = stdout();
        CANVAS_COUNT.fetch_add(1, Ordering::Release);
        if CANVAS_ONCE.compare_and_swap(true, false, Ordering::AcqRel) {
            if !out.is_tty() {
                eprintln!("Stdout is not a terminal");
            }
        }
        out.execute(cursor::Hide).expect("Failed to hide the cursor");
        Self {
            layers: Vec::new(),
            changed: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn add_layer(&mut self) {
        self.layers.push(self.new_layer());
    }

    fn new_layer(&self) -> Layer {
        Layer {
            buf: vec![StyledContent::new(ContentStyle::new(), ' '); Layer::size()],
            changed: Rc::clone(&self.changed),
        }
    }

    pub fn update(&self) -> Result<()> {
        self.update_inner(self.changed.borrow())
    }

    pub fn try_update(&self) -> Result<()> {
        self.update_inner(self.changed.try_borrow()?)
    }

    fn update_inner(&self, changed: impl Deref<Target = Vec<bool>>) -> Result<()> {
        let cols = terminal::size().expect("Unable to get terminal size").0;
        let mut out = stdout();
        for (i, e) in changed.iter().enumerate() {
            if *e {
                out
                    .queue(cursor::MoveTo((i % cols as usize) as u16, (i / cols as usize) as u16))?
                    .queue(PrintStyledContent(char_at(&self.layers, i)))?;
            }
        }
        out.flush()?;
        Ok(())
    }
}

static CANVAS_COUNT: AtomicUsize = AtomicUsize::new(0);

impl Drop for Canvas {
    fn drop(&mut self) {
        if CANVAS_COUNT.fetch_sub(1, Ordering::AcqRel) == 1 {
            stdout().execute(cursor::Show).expect("Failed to un-hide the cursor");
        }
    }
}

#[derive(Clone, Debug)]
pub struct Layer {
    buf: Vec<StyledContent<char>>,
    changed: Rc<RefCell<Vec<bool>>>,
    //sender: Sender<Message>
}

impl Layer {
    fn size() -> usize {
        let (cols, rows) = terminal::size().expect("Unable to get terminal size");
        rows as usize * cols as usize
    }

    #[cfg(windows)]
    fn resolution() -> (u16, u16) {
        let (cols, rows) = terminal::size().expect("Unable to get terminal size");
        (cols, rows * 2)
    }

    #[cfg(unix)]
    fn resolution() -> (u16, u16) {
        // https://hermanradtke.com/2015/01/12/terminal-window-size-with-rust-ffi.html
        // The above link is a blogpost on how to get the terminal size with rust FII.
        use libc::{c_int, c_ulong, c_ushort, winsize, STDOUT_FILENO};
        use libc::funcs::bsd44::ioctl;
        
    }

    fn fillRect(&self) {
        
    }
}


fn char_at(buf: &Vec<Layer>, i: usize) -> StyledContent<char> {
    let mut c = StyledContent::new(ContentStyle::new(), ' ');
    for layer in buf.iter().rev() {
        c = overlay(&c, &layer.buf[i]);
        if overlay_possible(c.content()) {
            break;
        }
    }
    c
}

fn overlay(_c1: &StyledContent<char>, c2: &StyledContent<char>) -> StyledContent<char> {
    *c2
}

fn overlay_possible(_c: &char) -> bool {
    false
}


#[derive(Debug)]
pub enum ErrorKind {
    PoisonError,
    WouldBlock,
    CrosstermError(crossterm::ErrorKind),
    IOError(io::Error),
    BorrowError,
}

impl Error for ErrorKind {}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ErrorKind::*;

        match self {
            PoisonError => write!(f, "poisoned lock: another task failed inside"),
            WouldBlock => write!(f, "try_lock failed because the operation would block"),
            CrosstermError(err) => write!(f, "{}", err),
            IOError(err) => write!(f, "{}", err),
            BorrowError => write!(f, "already mutably borrowed")
        }
    }
}

pub type Result<T> = std::result::Result<T, ErrorKind>;

impl<T> From<TryLockError<T>> for ErrorKind {
    fn from(err: TryLockError<T>) -> ErrorKind {
        use ErrorKind::*;

        match err {
            TryLockError::Poisoned(_) => PoisonError,
            TryLockError::WouldBlock => WouldBlock,
        }
    }
}

impl From<crossterm::ErrorKind> for ErrorKind {
    fn from(err: crossterm::ErrorKind) -> ErrorKind {
        ErrorKind::CrosstermError(err)
    }
}

impl<T> From<PoisonError<T>> for ErrorKind {
    fn from(_: PoisonError<T>) -> ErrorKind {
        ErrorKind::PoisonError
    }
}

impl From<io::Error> for ErrorKind {
    fn from(err: io::Error) -> ErrorKind {
        ErrorKind::IOError(err)
    }
}

impl From<BorrowError> for ErrorKind {
    fn from(_: BorrowError) -> ErrorKind {
        ErrorKind::BorrowError
    }
}