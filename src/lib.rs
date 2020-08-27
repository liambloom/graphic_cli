macro_rules! export_children {
    ($name:ident) => {
        mod $name;
        pub use $name::*;
    };
}

pub use paste::paste;

pub mod measurement;
pub mod colors;
pub mod elements;
mod macros;
mod element_traits; // Making this public would make elements::private public
//mod symbol;

export_children!(errors);

pub mod prelude {
    pub use crate::element_traits::{Element, Child, RemoveChild, Parent, OptionParent};
    pub use crate::elements::Document;
}

