macro_rules! export_children {
    ($name:ident) => {
        mod $name;
        pub use $name::*;
    };
}

pub mod measurement;
pub mod colors;
pub mod elements;
mod element_traits; // Making this public would make elements::private public

export_children!(errors);

pub mod prelude {
    pub use crate::element_traits::{Element, Child, RemoveChild};
    pub use crate::elements::Document;
}