// Copyright 2020 Liam Bloom
// SPDX-License-Identifier: Apache-2.0

//! # Graphic CLI
//! 
//! This library allows you to create a GUI in the command line.

// TODO: Move functions to Canvas if they can be
// TODO: Use mpsc to send data from layer to canvas

#![warn(missing_docs)]

use std::{
    io::{stdout, Read, Write, StdoutLock}, 
    iter::Iterator,
    sync::{
        atomic::{AtomicUsize, AtomicBool, Ordering},
        mpsc::{channel, Sender, TryRecvError},
        Once, Arc, Mutex, RwLock, Condvar
    },
    thread::{self, JoinHandle},
    marker::PhantomData,
    collections::HashSet,
    time::{Instant, Duration},
};
use crossterm::{
    tty::IsTty,
    style::{ContentStyle, StyledContent, PrintStyledContent, style},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, disable_raw_mode}, 
    cursor, event, execute, queue, ExecutableCommand, QueueableCommand
};
//use num_traits::{Zero, One};
use bmp;
use lazy_static::lazy_static;
#[cfg(unix)]
use libc::{winsize, ioctl, STDOUT_FILENO, TIOCGWINSZ};

pub use crossterm::style::Color;
pub use event::{Event, KeyEvent, KeyCode, KeyModifiers, MouseEvent, MouseEventKind, MouseButton};

pub mod error;
use error::*;

const DEFAULT_PX_SIZE: (f32, f32) = (1.0, 0.5);

#[cfg(windows)]
static PX_SIZE: (f32, f32) = DEFAULT_PX_SIZE;

#[cfg(unix)]
lazy_static! {
    static ref PX_SIZE: (f32, f32) = {
        match get_winsize() {
            Ok(w) => {
                let ar = (w.ws_xpixel as f64 / w.ws_col as f64) / (w.ws_ypixel as f64 / w.ws_row as f64);
                if ar < 0.75 {
                    (1.0, 0.5)
                }
                else if ar > 1.333 {
                    (0.5, 1.0)
                }
                else {
                    (1.0, 1.0)
                }
            },
            Err(_) => DEFAULT_PX_SIZE
        }
    };
}

//static CANVAS_COUNT: AtomicUsize = AtomicUsize::new(0);
// IDK if this works
/*lazy_static! {
    static ref CANVAS_LOCK: (AtomicBool, Condvar) = (AtomicBool::new(false), Condvar::new());
}*/

pub enum ResizeType {
    Manual(Box<dyn FnMut() -> ()>),
    Auto(ResizeAxis, ResizeAxis) // TODO: Add way to add cutoff function (hide edges, stop until resized)
}

#[derive(Copy, Clone, Debug)]
pub enum ResizeAxis {
    /// Either the top (vertical) or left (horizontal)
    Start,

    /// Center, rounds towards the start
    CenterRoundDown,

    /// Center, rounds towards the end
    CenterRoundUp,

    /// Either the bottom (vertical) or right (horizontal)
    End,
}

/*#[derive(Clone, Debug)]
enum Message {
    /// Tells the listener that something a modification is about to be
    /// made, and that it should now wait until updateLock is free
    DrawStarted,

    /// A new layer and its index
    NewLayer(Arc<Mutex<Vec<StyledContent<char>>>>, usize),

    /// Requests a full redraw
    FullRedraw,

    /// Terminates the thread
    End,
    // Redraw all characters
    //Redraw,
}*/

/// The main element of this crate, the `Canvas` element draws to the canvas
//#[derive(/*Clone, */Debug)]
pub struct Canvas {
    /// This holds the layers within the canvas.

    // TODO: Make accessible from multiple threads (so I can update from a listener thread)
    // To avoid deadlock: If updating is the only thing that ever requires both "changed" and
    //      "layer", and there's only one thread on which updating can happen (make Canvas#update
    //      just call sender.send(Message::Update)), then they will never deadlock. Note: this 
    //      means I can not expose a public interface for changed or for waiting for an update
    // ^ I think the above solution is inconvenient for users and probably wouldn't work, since
    //      waiting layers to have no references might take a long time
    // NEW IDEA: Make the buffer inside a layer be Arc<Mutex<Vec<StyledContent<char>>>> or
    //      Arc<RwLock<Vec<StyledContent<char>>>>, and replace Canvas#layers with a clone of that
    //      Arc, instead of the Layer. This would allow me to access the layers' buffers without 
    //      accessing the layers themselves
    // ^ This idea doesn't solve the deadlock, although it does make the solution I mentioned
    //      previously a bit more palatable, as the user couldn't keep a layer until an update
    //      finished, so only I would have to be careful with a public "wait for update" method.
    // ^ Also, while it does work, it does involve the type Vec<Arc<Mutex<Vec<StyledContent<char>>>>>,
    //      which is just stupidly long (and also requires following 4 pointers to get to anything)
    // ^ Also also, Idk if making each layer have its own lock would slow down or speed up the program
    // ANOTHER NEW IDEA: Each layer's buffer is an Arc<Vec<Mutex<StyledContent<char>>>>>. Since Mutex 
    //      have interior mutability, both update and draw would only be required to borrow the content
    //      of the Arc immutably, which means they can both work at the same time. 
    // ^ Still doesn't solve
    //      deadlock (although makes it less likely)
    // ^ Is there any way to have the updater thread update
    // TODO: No. This is not an acceptable type
    //pub layers: Arc<Vec<Arc<Vec<Mutex<StyledContent<char>>>>>>,

    /// What to do when the terminal is resized. Defaults to `ResizeType::Auto(ResizeAxis::Start, ResizeAxis::Start)`
    pub resize_type: ResizeType,

    layer_count: Arc<AtomicUsize>,

    // TODO: Make accessible from multiple threads
    // TODO: Replace with more efficient data structure, like HashSet ~~or BTreeSet~~
    changed: Arc<Mutex<HashSet<usize>>>,
    // RwLock may not be the perfect solution (and how it works depends on the kernel),
    //      see https://www.reddit.com/r/rust/comments/f4zldz/i_audited_3_different_implementation_of_async/?utm_source=share&utm_medium=web2x&context=3
    //      but it is approximately what I want (behavior-wise, not intent-wise), so it's good
    //      enough for now.
    //update_lock: Arc<RwLock<()>>,
    //sender: Sender<Message>,
    //listener: Option<JoinHandle<()>>,
}

/*macro_rules! do_while {
    (do $block:block while $condition:expr;) => {
        let mut first = true;
        while first || ($condition) {
            first = false;
            $block
        }
    };
}*/

impl Canvas {
    /// Creates a new canvas.
    /// 
    /// Warning: Do not create more than one `Canvas` at a time
    pub fn new() -> Result<Self> {
        let mut out = stdout();
        //let out = out.lock();
        //let stdout_lock_ptr = &stdout_lock as *const StdoutLock<'_>;
        //if CANVAS_COUNT.fetch_add(1, Ordering::AcqRel) == 0 { 
            execute!(out, EnterAlternateScreen, cursor::Hide)?;
            enable_raw_mode()?;
        //}
        static ONCE: Once = Once::new();
        ONCE.call_once(|| {
            if !out.is_tty() {
                eprintln!("Stdout is not a terminal");
            }
        });
        let changed = Arc::new(Mutex::new(HashSet::new()));
        let update_lock = Arc::new(RwLock::new(()));
        let layer_count = Arc::new(AtomicUsize::new(0));
        //let (sender, receiver) = channel();
        //let out_wrap = Arc::new(Mutex::new(out));
        Ok(Self {
            //layers: Arc::clone(&layers),
            resize_type: ResizeType::Auto(ResizeAxis::Start, ResizeAxis::Start),
            changed: Arc::clone(&changed),
            layer_count: Arc::clone(&layer_count),
            /*sender,
            update_lock: Arc::clone(&update_lock),
            // TODO: Add resize listener
            listener: Some(thread::spawn(move || {
                use Message::*;

                /*let stdout = stdout();
                let stdout = stdout.lock();*/
                //let out = out_wrap.into_inner().unwrap();
                let mut layers = Vec::new();

                // TODO: Do peek/poll in a loop to check multiple things (receiver.try_recv() || events::poll())
                loop {
                    let mut _lock = None;
                    let mut msg = if event::poll(Duration::from_secs(0)).unwrap() {
                            match event::read().unwrap() {
                                Event::Resize(cols, rows) => {
                                    todo!()
                                },
                                other => {
                                    todo!();
                                }
                            }
                        }
                        else {
                            None
                        };
                    do_while! {
                        do {
                            loop {
                                match receiver.try_recv() {
                                    Ok(DrawStarted) => {
                                        msg = msg.or(Some(DrawStarted));
                                        _lock = _lock.or_else(|| Some(update_lock.write().unwrap()));
                                    },
                                    Ok(FullRedraw) => {
                                        msg = Some(FullRedraw);
                                        _lock = _lock.or_else(|| Some(update_lock.write().unwrap()));
                                    },
                                    Ok(NewLayer(layer, index)) => layers.insert(index, layer),
                                    Err(TryRecvError::Empty) => break,
                                    Ok(End) | Err(TryRecvError::Disconnected) => return, 
                                }
                            }
                        }
                        while layer_count.load(Ordering::Acquire) != layers.len();
                    }

                    let msg = match msg {
                        Some(msg) => msg,
                        None => continue,
                    };

                    if layers.len() == 0 {
                        continue;
                    }

                    let mut layer_refs = Vec::with_capacity(layers.len());
                    for layer in layers.iter().rev() {
                        layer_refs.push(layer.lock().unwrap());
                    }

                    let cols = terminal::size().expect("Unable to get terminal size").0;
                    //let mut out = stdout();
                    let mut update_px = |i: usize| {
                        let mut c = StyledContent::new(ContentStyle::new(), ' ');
                        for layer in layer_refs.iter() {
                            underlay(&mut c, &layer[i]);
                            if !underlay_possible(&c) {
                                break;
                            }
                        }
                        queue!(out,
                            cursor::MoveTo((i % cols as usize) as u16, (i / cols as usize) as u16),
                            PrintStyledContent(c)
                        ).unwrap();
                    };
                    if let DrawStarted = msg {
                        for &i in changed.lock().unwrap().iter() {
                            update_px(i);
                        }
                    }
                    else {
                        for i in 0..layer_refs[0].len() {
                            update_px(i);
                        }
                    }

                    out.flush().unwrap();
                }
            })),*/
        })
    }

    pub fn new_layer<'a>(&'a self) -> Layer<'a> {
        self.new_layer_at(self.layer_count.load(Ordering::Acquire))
    }

    pub fn new_layer_at<'a>(&'a self, i: usize) -> Layer<'a> {
        let _lock = self.update_lock.read();
        let buf = Arc::new(Mutex::new(vec![StyledContent::new(ContentStyle::new(), ' '); Layer::size()]));
        let len = self.layer_count.fetch_add(1, Ordering::Acquire);
        if i > len {
            panic!("Index {} is out of bounds for length {}", i, len);
        }
        let _ = self.sender.send(Message::NewLayer(Arc::clone(&buf), i));
        // TODO: Add method to add layers in different places
        Layer {
            buf,
            changed: Arc::clone(&self.changed),
            phantom: PhantomData
        }
    }

    /// Gets the resolution of a canvas
    pub fn resolution() -> (u16, u16) {
        let (cols, rows) = terminal::size().expect("Unable to get terminal size");
        ((cols as f32 / PX_SIZE.0) as u16, (rows as f32 * PX_SIZE.1) as u16)
    }

    pub fn request_animation_frame(&self, f: impl FnOnce(Instant) + Send) -> i64 {
        todo!()
    }
}

impl Drop for Canvas {
    fn drop(&mut self) {
        // TODO: Maybe log error caused by listener thread panicking? They don't effect shutdown though, so don't unwrap them

        // "_" doesn't bind
        let _ = self.sender.send(Message::End);
        if CANVAS_COUNT.fetch_sub(1, Ordering::AcqRel) == 1 {
            let mut out = stdout();
            //let outRef = &mut out;
            disable_raw_mode()
                .and(out.queue(cursor::Show).map(|_| ()))
                .and(out.queue(LeaveAlternateScreen).map(|_| ()))
                .and(out.flush().map_err(|e| e.into() ))
                .expect("Error de-initializing canvas");
        }
        // This is known as the option dance
        // https://users.rust-lang.org/t/spawn-threads-and-join-in-destructor/1613/2
        let _ = self.listener.take().unwrap().join();
    }
}

/// The layer holds image data within a canvas
#[derive(/*Clone, */Debug)]
pub struct Layer<'a> {
    buf: Arc<Mutex<Vec<StyledContent<char>>>>,
    changed: Arc<Mutex<HashSet<usize>>>,
    phantom: PhantomData<&'a ()>,
}

impl<'a> Layer<'a> {
    fn size() -> usize {
        let (cols, rows) = terminal::size().expect("Unable to get terminal size");
        rows as usize * cols as usize
    }
}

impl<'a> Layer<'a> {
    /// Sets the color of one pixel of the layer
    pub fn plot(&mut self, p: IPoint, color: Color) -> Result<()> {
        todo!();
        //p = (p.0.floor(), p.1.floor());
        /*self.validate_iPoints(&[p])?;
        let r = Canvas::resolution();
        if p.0 > r.0 || p.1 > r.1 {
            panic!("Cannot draw outside of bounds");
        }
        let i = (p.1 as f32 * PX_SIZE.1) as usize * terminal::size().expect("Unable to get terminal size").0 as usize + (p.0 as f32 * PX_SIZE.0) as usize;
        //let i = iPoint_to_index(&p);
        self.changed.borrow_mut()[i] = true;
        let x = p.0 as f32 * PX_SIZE.0;
        let y = p.1 as f32 * PX_SIZE.1;
        overlay(&mut self.buf[i], &style(
            if PX_SIZE.0 == 0.5 {
                if x % 1.0 == 0.0 {
                    '▌'
                }
                else {
                    '▐'
                }
            }
            else if PX_SIZE.1 == 0.5 {
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
        Ok(())*/
    }

    fn resolution(&self) -> (usize, usize) {
        let res = Canvas::resolution();
        (res.0 as usize, res.0 as usize)
    }

    /// Draws a line connecting points `p0` and `p1`
    pub fn line(&mut self, p0: FPoint, p1: FPoint, color: Color) -> Result<()> {
        // Bresenham's Line algorithm with sub-pixel precision is here
        // https://stackoverflow.com/questions/41195973/how-to-use-bresenhams-line-drawing-algorithm-with-sub-pixel-bias
        // I'm not using it because having a high precision (a high `scale`), the loop
        // runs like a bazillion times, so it doesn't matter how efficient the algorithm
        // is, it's not that much faster than DDA. DDA is a slightly slower algorithm, 
        // but it's easier, and loops fewer times (far fewer with a high precision).
        // Maybe I could even modify DDA to keep track of how far off it is to be self
        // correcting (because DDA becomes offset from correct after a long distance)
        self.validate_fPoints(&[p0, p1])?;
        let dx = p1.0 - p0.0;
        let dy = p1.1 - p0.1;
        let steps;
        let mut x;
        let mut y;
        if dx.abs() > dy.abs() {
            x = p0.0.floor() + 0.5;
            y = p0.1 + (p0.0 - x) * (dy / dx);
            steps = dx.abs();
        } 
        else {
            y = p0.1.floor() + 0.5;
            x = p0.0 + (p0.1 - y) * (dx / dy);
            steps = dy.abs();
        }
        let x_step = dx / steps;
        let y_step = dy / steps;
        for _ in 0..=steps.round() as i32 {
            self.plot((x as u16, y as u16), color)?;
            x += x_step;
            y += y_step;
        }
        Ok(())
    }

    /// Draws and fills a rectangle to the layer
    pub fn fill_rect(&mut self, x: u16, y: u16, width: u16, height: u16, color: Color) -> Result<()> {
        self.validate_iPoints(&[(x, y), (x + width, y + height)])?;
        for y in y..(y + height) {
            for x in x..(x + width) {
                self.plot((x, y), color)?;
            }
        }
        Ok(())
    }

    pub fn draw_img(&mut self, x: u16, y: u16, src: &mut impl Read) -> Result<()> {
        let img = bmp::from_reader(src)?;
        for px in img.coordinates() {
            let px_color = img.get_pixel(px.0, px.1);
            self.plot((x + px.0 as u16, y + px.1 as u16), Color::Rgb { r: px_color.r, g: px_color.g, b: px_color.b })?;
        }
        Ok(())
    }

    fn validate_fPoints(&self, points: &[FPoint]) -> Result<()> {
        let resolution = self.resolution();
        let resolution = (resolution.0 as f32, resolution.1 as f32);
        for point in points {
            if point.0 < 0.0 || point.0.round() >= resolution.0 || point.1 < 0.0 || point.1.round() >= resolution.1 {
                return Err(ErrorKind::InvalidPoint(point.0, point.1))
            }
        }
        Ok(())
    }

    fn validate_iPoints(&self, points: &[IPoint]) -> Result<()> {
        let resolution = self.resolution();
        let resolution = (resolution.0 as u16, resolution.1 as u16);
        for point in points {
            if point.0 < 0 || point.0 >= resolution.0 || point.1 < 0 || point.1 >= resolution.1 {
                return Err(ErrorKind::InvalidPoint(point.0 as f32, point.1 as f32))
            }
        }
        Ok(())
    }
}

// NOTE: Methods that need integer points can take arguments of type Point<impl Integer>
// Should I add Copy trait bound?
/*pub trait Point<T> {
    fn x(&self) -> T;
    fn y(&self) -> T;
}

impl<T: Copy> Point<T> for (T, T) {
    fn x(&self) -> T {
        self.0
    }

    fn y(&self) -> T {
        self.1
    }
}

trait ToU16: Zero + One + PartialEq + Copy {
    fn to_u16(self) -> u16 {
        if self == Self::zero() {
            0
        }
        else {
            let mut r = 0;
            let two = Self::one() + Self::one();
            let n = Self::one();

        }
    }
}*/

/// Point type
pub type FPoint = (f32, f32);
pub type IPoint = (u16, u16);

/*fn fPoint_to_index(p: &FPoint) -> usize {
    let r = Canvas::resolution();
    if p.0 > r.0.into() || p.1 > r.1.into() {
        panic!("Cannot draw outside of bounds");
    }
    (p.1 * PX_SIZE.1 * terminal::size().expect("Unable to get terminal size").0 as f32) as usize + (p.0 * PX_SIZE.0) as usize
}

fn iPoint_to_index(p: &IPoint) -> usize {
    let r = Canvas::resolution();
    if p.0 > r.0 || p.1 > r.1 {
        panic!("Cannot draw outside of bounds");
    }
    (p.1 as f32 * PX_SIZE.1 * terminal::size().expect("Unable to get terminal size").0 as f32) as usize + (p.0 as f32 * PX_SIZE.0) as usize
}*/


/// Overlays c2 over c1, storing the result in c1
fn overlay(c1: &mut StyledContent<char>, c2: &StyledContent<char>) {
    if !underlay_possible(c2) {
        *c1 = *c2
    }
    else if c1.content() == c2.content() || ('─'..='╬').contains(c1.content()) {
        let s1 = c1.style_mut();
        let s2 = c2.style();
        if s2.foreground_color.is_some() {
            s1.foreground_color = s2.foreground_color;
        }
        if s2.background_color.is_some() {
            s1.background_color = s2.background_color;
        }
        //let lines = [['─', '━', '═'], ]
        if ![' ', '█'].contains(c2.content()) {
            todo!()
        }
    }
    // if c2 is ' ' or '', and it can be underlayed, c2 is entirely transparent,
    // so c1 stays the same
    else if ![' ', '█'].contains(c2.content()) {
        match if c1.content() < c2.content() { (c1.content(), c2.content()) } else { (c2.content(), c1.content()) } {
            ('▌', '▐') | ('▀', '▄') => (*c1.style_mut()).background_color = c2.style().foreground_color,
            //('─', '━') => *c1 = StyledContent::new(c1.style()),
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
    match c.content() {
        '█' => c.style().foreground_color.is_none(),
        '▌' | '▐' | '▀' | '▄' => {
            let style = c.style();
            style.foreground_color.is_none() || style.background_color.is_none()
        },
        _ => c.style().background_color.is_none(),
    }
}

// Parts of the following function were taken from code written by Herman J. Radtke III
// It can be found at https://hermanradtke.com/2015/01/12/terminal-window-size-with-rust-ffi.html
// The original code is licensed under CC BY 4.0 (https://creativecommons.org/licenses/by/4.0/)
// Changes have been made to the code
//
// It is changed enough that I feel it would be acceptable to use, as I don't feel that two lines
// of code are enough to constitute a "creative work," especially since this is the only way to 
// use the ioctl function that I am aware of, but it took way to long to find this, so I'm putting
// a shoutout to hjr3 for posting the code.
#[cfg(unix)]
fn get_winsize() -> Result<winsize> {
    let w = winsize { ws_row: 0, ws_col: 0, ws_xpixel: 0, ws_ypixel: 0 };
    let r = unsafe { ioctl(STDOUT_FILENO, TIOCGWINSZ, &w) };

    debug_assert_eq!((w.ws_col, w.ws_row), terminal::size().unwrap());

    if r == 0 && w.ws_xpixel > 0 && w.ws_ypixel > 0 && w.ws_col > 0 && w.ws_row > 0 {
        Ok(w)
    } 
    else {
        Err(io::Error::from(io::ErrorKind::Other).into())
    }
}
