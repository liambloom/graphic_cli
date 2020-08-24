use std::fmt::Display;
use crate::{
    Error, 
    measurement::*,
};

pub trait Element: Display + PrivElement {
    fn doc(&self) -> &dyn Element; // In order to return the type Document<R, W> it must know R and W, so Element would have to have 2 generic types
    fn children(&self) -> Vec<&dyn Child>; // Rc or Box?
    fn children_mut(&mut self) -> Vec<&mut dyn Child>;
    fn add_child(&mut self, child: Box<dyn Child>) -> Result<(), Error> {
        if self.doc().contains_id(child.id()) { Err(Error::IdAlreadyExists(child.id().to_string())) }
        else {
            self.children_owned().push(child);
            Ok(())
        }
    }
    fn remove_child(&mut self, id: &str) -> Result<(), Error> {
        let children = self.children_owned();
        children.remove(children.iter().position(|e| e.id() == id).ok_or(Error::ChildNotFound(id.to_string()))?);
        Ok(())
    }
    /* {
        self.add_child_at(child, self.child_count())
    }*/
    //fn add_child_at(&mut self, child: Box<dyn Child>, i: usize);
    fn add_child_after(&mut self, child: Box<dyn Child>, relative: &dyn Child) -> Result<(), Error> {
        unimplemented!();
        /*match self.index_of(&*child) {
            Some(i) => {
                self.add_child_at(child, i + 1);
                Ok(())
            }
            None => Err(ChildNotFound::InsertBefore(relative))
        }*/
    }
    fn add_child_before(&mut self, child: Box<dyn Child>, relative: &dyn Child) -> Result<(), Error> {
        unimplemented!();
        /*match self.index_of(&*child) {
            Some(i) => {
                self.add_child_at(child, i);
                Ok(())
            }
            None => Err(ChildNotFound::InsertAfter(relative))
        }*/
    }
    //fn remove_child_at(&mut self, i: usize);
    /*fn remove_child(&mut self, child: Box<dyn Child>) -> Result<(), ChildNotFound> {
        match self.index_of(&*child) {
            Some(i) => {
                self.remove_child_at(i);
                Ok(())
            }
            None => Err(ChildNotFound::Remove(child))
        }
    }*/
    fn contains(&self, child: &dyn Child) -> bool {
        self.contains_id(child.id())
    }
    fn contains_id(&self, id: &str) -> bool {
        self.children().iter().any(|e| e.id() == id || e.contains_id(id))
    }
    //fn index_of(&self, child: &dyn Child) -> Option<usize>;
    fn child_count(&self) -> usize;
    fn get_child(&self, id: &str) -> Option<&dyn Child> {
        for e in self.children() {
            if e.id() == id {
                return Some(e);
            }
            else {
                let el = e.get_child(id);
                if el.is_some() {
                    return el;
                }
            }
        }
        None
    }
    fn get_child_mut(&mut self, id: &str) -> Option<&mut dyn Child> {
        for e in self.children_mut() {
            if e.id() == id {
                return Some(e);
            }
            else {
                let el = e.get_child_mut(id);
                if el.is_some() {
                    return el;
                }
            }
        }
        None
    }
    fn get_width(&self) -> u16;
    fn get_height(&self) -> u16;
}

pub trait PrivElement {
    fn draw(&self);
    fn children_owned(&mut self) -> &mut Vec<Box<dyn Child>>;
    fn width_exact(&self) -> f32;
    fn height_exact(&self) -> f32;
}

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