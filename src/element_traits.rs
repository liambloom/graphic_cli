use std::fmt::{self, Display};
use crate::errors;

pub mod private {
    use super::Child;

    pub trait PrivElement {
        fn draw(&self);
        fn children_owned(&mut self) -> &mut Vec<Box<dyn Child>>;
    }
}

use private::*;

pub trait Element: fmt::Debug + Display + PrivElement {
    fn children(&self) -> Vec<&dyn Child>; // Rc or Box?
    fn children_mut(&self) -> Vec<&mut dyn Child>;
    fn add_child(&mut self, child: Box<dyn Child>) -> Result<(), errors::IdAlreadyExists> {
        if self.contains_id(child.id()) { Err(errors::IdAlreadyExists) }
        else { 
            self.children_owned().push(child);
            Ok(())
        }
    }
    fn remove_child(&mut self, id: &str) -> Result<(), errors::ChildNotFound> {
        let children = self.children_owned();
        children.remove(children.iter().position(|e| e.id() == id).ok_or(errors::ChildNotFound)?);
        Ok(())
    }
    /* {
        self.add_child_at(child, self.child_count())
    }*/
    //fn add_child_at(&mut self, child: Box<dyn Element>, i: usize);
    fn add_child_after<'a>(&mut self, child: Box<dyn Element>, relative: &'a dyn Element) -> Result<(), errors::ChildNotFound> {
        unimplemented!();
        /*match self.index_of(&*child) {
            Some(i) => {
                self.add_child_at(child, i + 1);
                Ok(())
            }
            None => Err(ChildNotFound::InsertBefore(relative))
        }*/
    }
    fn add_child_before<'a>(&mut self, child: Box<dyn Element>, relative: &'a dyn Element) -> Result<(), errors::ChildNotFound> {
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
    /*fn remove_child(&mut self, child: Box<dyn Element>) -> Result<(), ChildNotFound> {
        match self.index_of(&*child) {
            Some(i) => {
                self.remove_child_at(i);
                Ok(())
            }
            None => Err(ChildNotFound::Remove(child))
        }
    }*/
    fn contains(&self, child: &dyn Element) -> bool {
        self.contains_id(child.id())
    }
    fn contains_id(&self, id: &str) -> bool {
        self.children().iter().any(|e| e.id() == id || e.contains_id(id))
    }
    fn index_of(&self, child: &dyn Child) -> Option<usize>;
    fn child_count(&self) -> usize;
    fn id(&self) -> &str;
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
}

pub trait Child: Element + RemoveChild {
    fn parent(&self) -> &dyn Element;
    fn parent_mut(&self) -> &mut dyn Element;
}
pub trait RemoveChild {
    fn remove(self); // This returns self. I'm pretty sure this causes every compilation error on this page
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