use std::{
    sync::{Arc, RwLock, Once, TryLockError, PoisonError, RwLockWriteGuard as WLock, atomic::{AtomicBool, Ordering}},
    io::{self, stdin, stdout, stderr, Write},
    convert::From,
    ops::Deref,
    error::Error,
    fmt,
};
use crossterm::{
    QueueableCommand, ExecutableCommand,
    tty::IsTty,
    style::{ContentStyle, StyledContent, PrintStyledContent},
    terminal, cursor,
};
use lazy_static::lazy_static;

type Av<T> = Arc<RwLock<Vec<T>>>;

static AUTO_UPDATE: AtomicBool = AtomicBool::new(true);

#[derive(Copy, Clone, Debug)]
enum TTYWrite {
    Out,
    Err,
}

impl TTYWrite {
    pub fn get_write(&self) -> Box<dyn Write> {
        match self {
            TTYWrite::Out => Box::new(stdout()),
            TTYWrite::Err => Box::new(stderr())
        }
    }
}

lazy_static! {
    static ref WRITE_CONSTRUCTOR: TTYWrite = {
        if stdout().is_tty() {
            TTYWrite::Out
        }
        else if stderr().is_tty() {
           TTYWrite::Err
        }
        else {
            panic!("Neither stdout nor stderr are ttys");
        }
    };
}

fn ttyout() -> Box<dyn Write> {
    WRITE_CONSTRUCTOR.get_write()
}

struct CursorHider;

impl Drop for CursorHider {
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        ttyout().execute(cursor::Hide);
    }
}

lazy_static! {
    static ref LAYERS: Av<Layer> = Arc::new(RwLock::new(Vec::new()));
    static ref CHANGED: Av<bool> = Arc::new(RwLock::new(vec![true; Layer::size()]));
}

pub fn canvas() -> Canvas {
    static ONCE: Once = Once::new();
    static _CURSOR_HIDER: CursorHider = CursorHider;

    #[allow(unused_must_use)]
    ONCE.call_once(|| {
        ttyout().execute(cursor::Show);

        if !stdin().is_tty() {
            panic!("Stdin is not tty");
        }

        ttyout();

        Canvas {
            layers: Arc::clone(&LAYERS),
            changed: Arc::clone(&CHANGED),
        }.draw();
    });

    Canvas {
        layers: Arc::clone(&LAYERS),
        changed: Arc::clone(&CHANGED),
    }
}

pub struct Canvas {
    pub layers: Av<Layer>,
    changed: Av<bool>,
}

impl Canvas {

    pub fn add_layer(&mut self) -> Result<()> {
        let mut layers = self.layers.write()?;
        layers.push(self.new_layer());
        Ok(())
    }

    pub fn try_add_layer(&mut self) -> Result<()> {
        let mut layers = self.layers.try_write()?;
        layers.push(self.new_layer());
        Ok(())
    }

    fn new_layer(&self) -> Layer {
        Layer {
            buf: vec![StyledContent::new(ContentStyle::new(), ' '); Layer::size()],
        }
    }

    pub fn draw(&self) -> Result<()> {
        self.draw_inner(
            self.changed.write()?,
            self.layers.write()?
        )
    }

    pub fn try_draw(&self) -> Result<()> {
        self.draw_inner(
            self.changed.try_write()?,
            self.layers.try_write()?
        )
    }

    fn draw_inner(&self, changed: WLock<Vec<bool>>, layers: WLock<Vec<Layer>>) -> Result<()> {
        let cols = terminal::size().expect("Unable to get terminal size").0;
        let mut out = ttyout();
        for (i, e) in changed.iter().enumerate() {
            if *e {
                out
                    .queue(cursor::MoveTo((i % cols as usize) as u16, (i / cols as usize) as u16))?
                    .queue(PrintStyledContent(char_at(&layers, i)))?;
            }
        }
        out.flush()?;
        Ok(())
    }
}

pub struct Layer {
    buf: Vec<StyledContent<char>>,
}

impl Layer {
    fn size() -> usize {
        let (cols, rows) = terminal::size().expect("Unable to get terminal size");
        rows as usize * cols as usize
    }
}


fn char_at<T: Deref<Target = Vec<Layer>>>(buf: &T, i: usize) -> StyledContent<char> {
    let mut c = StyledContent::new(ContentStyle::new(), ' ');
    for j in (0..buf.len()).rev() {
        c = overlay(&c, &buf[j].buf[i]);
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
}

impl Error for ErrorKind {}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ErrorKind::*;

        match self {
            PoisonError => write!(f, "Poison"),
            WouldBlock => write!(f, "WouldBlock"),
            CrosstermError(err) => write!(f, "{}", err),
            IOError(err) => write!(f, "{}", err),
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