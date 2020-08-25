use super::*;
use crate::measurement::Unit;

pub trait Child: Element + RemoveChild {
    fn parent(&self) -> &dyn Element;
    fn parent_mut(&self) -> &mut dyn Element;
    fn set_width(&mut self, v: Unit);
    fn set_height(&mut self, v: Unit);
    fn id(&self) -> &str;
    /*fn remove(&mut self) {
        match self.parent_mut().remove_child(self.id()) {
            Ok(()) => (),
            Err(_) => unreachable!(),
        };
    }*/
}
pub trait RemoveChild {
    fn remove(self);
}

impl<T> RemoveChild for T // This looks nice, and it doesn't cause any compilation errors, but there's no way it actually works
    where T: 'static + Child // I'm not completely certain what 'static does, and I hope that it doesn't break the program
{
    fn remove(self) {
        match self.parent_mut().remove_child(self.id()) {
            Ok(()) => (),
            Err(_) => unreachable!(),
        };
    }
}