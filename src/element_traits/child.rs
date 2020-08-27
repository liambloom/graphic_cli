use super::*;

pub trait RemoveChild {
    fn remove(self);
}

pub trait Child: Element {}

impl<T> RemoveChild for T // This looks nice, and it doesn't cause any compilation errors, but there's no way it actually works
    where T: 'static + Element // I'm not completely certain what 'static does, and I hope that it doesn't break the program
{
    fn remove(self) {
        match self.parent_mut().unwrap().remove_child(self.id().expect("You cannot remove a top level element")) {
            Ok(()) => (),
            Err(_) => unreachable!(),
        };
    }
}