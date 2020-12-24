// Copyright 2020 Liam Bloom
// SPDX-License-Identifier: Apache-2.0

use std::{
    sync::atomic::AtomicUsize, 
    convert::From, 
    error::Error, 
    fmt, 
    io::{self, stdout, Write},
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
    style::{ContentStyle, StyledContent, PrintStyledContent, style},
    terminal, cursor,
};
pub use crossterm::style::Color;
#[cfg(unix)]
use libc::{winsize, ioctl, STDOUT_FILENO, TIOCGWINSZ};

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
            changed: Rc::new(RefCell::new(vec![true; Layer::size()])),
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
    pub buf: Vec<StyledContent<char>>,
    changed: Rc<RefCell<Vec<bool>>>,
    //sender: Sender<Message>
}

impl Layer {
    fn size() -> usize {
        let (cols, rows) = terminal::size().expect("Unable to get terminal size");
        rows as usize * cols as usize
    }

    #[cfg(windows)]
    pub fn resolution() -> (u16, u16) {
        let (cols, rows) = terminal::size().expect("Unable to get terminal size");
        (cols, rows * 2)
    }

    
    #[cfg(unix)]
    pub fn resolution() -> (u16, u16) {
        let w = get_winsize();

    }

    pub fn fill_rect(&mut self, x: u16, y: u16, width: u16, height: u16, color: Color) {
        for i in y..(y+height) {
            for j in x..(x+width) {
                self.set_px(i, j, color);
            }
        }
    }

    fn px_size() -> (f32, f32) {
        let r = Layer::resolution();
        let (cols, rows) = terminal::size().expect("Unable to get terminal size");
        let ar = (r.0 as f64 / cols as f64) / (r.1 as f64 / rows as f64);
        if ar < 0.75 {
            (1.0, 0.5)
        }
        else if ar > 1.333 {
            (0.5, 1.0)
        }
        else {
            (1.0, 1.0)
        }
    }

    fn coords_to_index(x: u16, y: u16) -> usize {
        let r = Layer::resolution();
        if x > r.0 || y > r.1 {
            panic!("Cannot draw outside of bounds");
        }
        let (px_width, px_height) = Layer::px_size();
        let x = x as f32 * px_width;
        let y = y as f32 * px_height;
        y as usize * terminal::size().expect("Unable to get terminal size").0 as usize + x as usize
    }

    fn set_px(&mut self, x: u16, y: u16, color: Color) {
        let i = Layer::coords_to_index(x, y);
        self.changed.borrow_mut()[i] = true;
        let (px_width, px_height) = Layer::px_size();
        let x = x as f32 * px_width;
        let y = y as f32 * px_height;
        overlay(&mut self.buf[i], &style(
        if px_width == 0.5 {
                if x % 1.0 == 0.0 {
                    '▌'
                }
                else {
                    '▐'
                }
            }
            else if px_height == 0.5 {
                if y % 1.0 == 0.0 {
                    '▀'
                }
                else {
                    '▄'
                }
            }
            else {
                '█'
            }
        )
            .with(color));
        
    }
}


fn char_at(buf: &Vec<Layer>, i: usize) -> StyledContent<char> {
    let mut c = StyledContent::new(ContentStyle::new(), ' ');
    for layer in buf.iter().rev() {
        underlay(&mut c, &layer.buf[i]);
        if !underlay_possible(&c) {
            break;
        }
    }
    c
}

/// Overlays c2 over c1, storing the result in c1
fn overlay(c1: &mut StyledContent<char>, c2: &StyledContent<char>) {
    if !underlay_possible(c2) {
        *c1 = *c2
    }
    else if c1.content() == c2.content() {
        let s1 = c1.style_mut();
        let s2 = c2.style();
        if s2.foreground_color.is_some() {
            s1.foreground_color = s2.foreground_color;
        }
        if s2.background_color.is_some() {
            s1.background_color = s2.background_color;
        }
    }
    else {
        match (c1.content(), c2.content()) {
            ('▌', '▐') | ('▐', '▌') | ('▀', '▄') | ('▄', '▀') => (*c1.style_mut()).background_color = c2.style().foreground_color,
            (_, ' ') | (_, '█') => (),
            _ => *c1 = *c2
        }
    }
}

/// Overlays c1 over c2, storing the result in c1
fn underlay(c1: &mut StyledContent<char>, c2: &StyledContent<char>) {
    let mut c2 = *c2;
    overlay(&mut c2, c1);
    *c1 = c2;
}

fn underlay_possible(c: &StyledContent<char>) -> bool {
    if ['▌', '▐', '▀', '▄'].contains(c.content()) {
        let style = c.style();
        style.foreground_color.is_none() || style.background_color.is_none()
    }
    else {
        match c.content() {
            ' ' => c.style().background_color.is_none(),
            '█' => c.style().foreground_color.is_none(),
            _ => false
        }
    }
}

// Parts of the following function were taken from code written by Herman J. Radtke III
// It can be found at https://hermanradtke.com/2015/01/12/terminal-window-size-with-rust-ffi.html
// The original code is licensed under CC BY 4.0 (https://creativecommons.org/licenses/by/4.0/)
// Changes have been made to the code
#[cfg(unix)]
fn get_winsize() -> winsize {
    let w = winsize { ws_row: 0, ws_col: 0, ws_xpixel: 0, ws_ypixel: 0 };
    let r = unsafe { ioctl(STDOUT_FILENO, TIOCGWINSZ, &w) };

    debug_assert_eq!((w.ws_col, w.ws_row), terminal::size().unwrap());

    w
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