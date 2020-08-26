use super::*;
use crate::errors::ErrorKind;

pub trait OptionParent {
    fn add_child(&mut self, _child: Box<dyn Child>) -> Result<(), ErrorKind> {
        Err(ErrorKind::NoChildrenAllowed)
    }
    fn remove_child(&mut self, _id: &str) -> Result<(), ErrorKind> {
        Err(ErrorKind::NoChildrenAllowed)
    }
    fn add_child_after(&mut self, _child: Box<dyn Child>, _relative: &dyn Child) -> Result<(), ErrorKind> {
        Err(ErrorKind::NoChildrenAllowed)
    }
    fn add_child_before(&mut self, _child: Box<dyn Child>, _relative: &dyn Child) -> Result<(), ErrorKind> {
        Err(ErrorKind::NoChildrenAllowed)
    }
    fn contains(&self, child: &dyn Child) -> bool {
        self.contains_id(child.id())
    }
    fn contains_id(&self, _id: &str) -> bool {
        false
    }
    fn child_count(&self) -> usize {
        0
    }
    fn get_child(&self, _id: &str) -> Option<&dyn Child> {
        None
    }
    fn get_child_mut(&mut self, _id: &str) -> Option<&mut dyn Child> {
        None
    }
}

pub trait Parent: Element {}

impl<T> OptionParent for T
    where T: Parent
{
    fn add_child(&mut self, child: Box<dyn Child>) -> Result<(), ErrorKind> {
        if self.doc().contains_id(child.id()) { Err(ErrorKind::IdAlreadyExists(child.id().to_string())) }
        else {
            self.children_owned().push(child);
            Ok(())
        }
    }
    fn remove_child(&mut self, id: &str) -> Result<(), ErrorKind> {
        let children = self.children_owned();
        children.remove(children.iter().position(|e| e.id() == id).ok_or(ErrorKind::ChildNotFound(id.to_string()))?);
        Ok(())
    }
    fn add_child_after(&mut self, _child: Box<dyn Child>, _relative: &dyn Child) -> Result<(), ErrorKind> {
        unimplemented!();
    }
    fn add_child_before(&mut self, _child: Box<dyn Child>, _relative: &dyn Child) -> Result<(), ErrorKind> {
        unimplemented!();
    }
    fn contains_id(&self, id: &str) -> bool {
        self.children().iter().any(|e| e.id() == id || e.contains_id(id))
    }
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