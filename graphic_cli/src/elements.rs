

/*
for each element
pub fn new(parent: Rc<dyn Parent>, other args) -> Rc<Self> {
convert the parent to a weak reference
if parent is a child, set doc to parent.doc()
else set doc to Rc::clone(parent).downgrade()
}
*/

reexport!(test_child);
//#[cfg(feature = "tty")]
reexport!(tty_doc);