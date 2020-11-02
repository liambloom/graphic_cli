pub struct InnerDrop {
    f: Box<dyn Fn() -> () + 'static>,
}

impl InnerDrop {
    pub fn new<F>(f: F) -> Self
        where F: Fn() -> () + 'static
    {
        Self {
            f: Box::new(f)
        }
    }
}

impl Drop for InnerDrop {
    fn drop(&mut self) {
        (self.f)();
    }
}