UNCHECKED

use std::{ops::Deref, cmp::{Eq, PartialEq}};

pub struct Symbol<T> {
    inner: T,
}
impl<T> Symbol<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
}
impl<T> PartialEq for Symbol<T> {
    fn eq(&self, o: &Self) -> bool {
        self as *const Self == o as *const Self
    }
}
impl<T> Eq for Symbol<T> {}
impl<T> Deref for Symbol<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}