use super::*;
use crate::errors::ErrorKind;

pub trait Parent: Element {
    fn children(&self) -> Vec<&dyn Child>; // Rc or Box?
    fn children_mut(&mut self) -> Vec<&mut dyn Child>;
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
    /* {
        self.add_child_at(child, self.child_count())
    }*/
    //fn add_child_at(&mut self, child: Box<dyn Child>, i: usize);
    fn add_child_after(&mut self, _child: Box<dyn Child>, _relative: &dyn Child) -> Result<(), ErrorKind> {
        unimplemented!();
        /*match self.index_of(&*child) {
            Some(i) => {
                self.add_child_at(child, i + 1);
                Ok(())
            }
            None => Err(ChildNotFound::InsertBefore(relative))
        }*/
    }
    fn add_child_before(&mut self, _child: Box<dyn Child>, _relative: &dyn Child) -> Result<(), ErrorKind> {
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
            else if e {
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