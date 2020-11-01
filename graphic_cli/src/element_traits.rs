use std::{
    rc::Rc,
    cell::{Ref, RefMut, RefCell},
    any::TypeId,
    sync::atomic::AtomicUsize,
};
use crate::{
    elements::*,
};
use as_any_min::AsAny;

fn type_id_of_value<T: 'static + ?Sized>(_value: &T) -> TypeId {
    TypeId::of::<T>()
}

pub trait Element: AsElement + AsParent + AsChild + AsAny + 'static {
    fn id(&self) -> usize;
}

impl dyn Element {
    const ELEMENT_COUNT: AtomicUsize = AtomicUsize::new(0);

    fn downcast_ref<'a, T: Element>(el_ref: Ref<'a, dyn Element>) -> Result<Ref<'a, T>, Ref<'a, dyn Element>> {
        if (*el_ref).as_any().is::<T>() {
            // SAFETY: I checked that it is T, so it's safe
            // The unsafe conversion is copied strait from the source code of the Any struct
            Ok(Ref::map(el_ref, |inner| unsafe { &*(inner as *const dyn Element as *const T) } ))
        }
        else {
            Err(el_ref)
        }
    }

    fn downcast_mut<'a, T: Element>(el_ref: RefMut<'a, dyn Element>) -> Result<RefMut<'a, T>, RefMut<'a, dyn Element>> {
        if (*el_ref).as_any().is::<T>() {
            // SAFETY: I checked that it is T, so it's safe
            // The unsafe conversion is copied strait from the source code of the Any struct
            Ok(RefMut::map(el_ref, |inner| unsafe { &mut *(inner as *mut dyn Element as *mut T) }))
        }
        else {
            Err(el_ref)
        }
    }
}

pub trait AsElement {
    fn as_element(&self) -> &dyn Element;
    fn as_element_mut(&mut self) -> &mut dyn Element;
}

impl<T> AsElement for T
    where T: Element
{
    fn as_element(&self) -> &dyn Element {
        self
    }
    fn as_element_mut(&mut self) -> &mut dyn Element {
        self
    }
}

pub trait AsParent {
    fn is_parent(&self) -> bool;
    fn as_parent(&self) -> Option<&dyn Parent>;
    fn as_parent_mut(&mut self) -> Option<&mut dyn Parent>;
    // fn as_parent_boxed(&self) -> Box<dyn Parent>;
}

pub trait Parent: Element {
    fn children(&self) -> &Vec<Rc<RefCell<dyn Child>>>;
}

impl dyn Parent {
    fn ref_as_parent<'a>(el_ref: Ref<'a, dyn Element>) -> Option<Ref<'a, dyn Parent>> {
        if el_ref.is_parent() {
            Some(Ref::map(el_ref, |orig| orig.as_parent().unwrap()))
        }
        else {
            None
        }
    }

    fn ref_mut_as_parent<'a>(el_ref: RefMut<'a, dyn Element>) -> Option<RefMut<'a, dyn Parent>> {
        if el_ref.is_parent() {
            Some(RefMut::map(el_ref, |orig| orig.as_parent_mut().unwrap()))
        }
        else {
            None
        }
    }

    pub fn get_child<'a, T: Child>(&'a self, id: usize) -> Option<Ref<'a, T>> {
        for child in self.children().iter() {
            let child = (&**child).borrow();
            match Element::downcast_ref::<T>(Ref::map(child, |child| child.as_element())) {
                Ok(child) => return Some(child),
                Err(child) => {
                    if let Some(child) = Parent::ref_as_parent(child) {
                        if let Some(sub_child) = child.get_child::<T>(id) {
                            return Some(sub_child);
                        }
                    }
                }
            }
        }
        None
    }

    // I don't *need* to make this &mut self, but should I?
    pub fn get_child_mut<T: Element + 'static>(&self, id: usize) -> Option<RefMut<'_, T>> {
        for child in self.children().iter() {
            let child = (&**child).borrow_mut();
            match Element::downcast_mut::<T>(RefMut::map(child, |child| child.as_element_mut())) {
                Ok(child) => return Some(child),
                Err(child) => {
                    if let Some(child) = Parent::ref_mut_as_parent(child) {
                        if let Some(sub_child) = child.get_child_mut::<T>(id) {
                            return Some(sub_child);
                        }
                    }
                }
            }
        }
        None
    }
}

pub trait AsChild {
    fn is_child(&self) -> bool;
    fn as_child(&self) -> Option<&dyn Child>;
    fn as_child_mut(&mut self) -> Option<&mut dyn Child>;
}

pub trait Child: Element {
    fn doc(&self) -> Rc<Doc>;
    fn parent(&self) -> Rc<dyn Parent>;
}