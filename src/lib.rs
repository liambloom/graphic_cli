mod element_traits; // Making this public would make elements::private public
pub mod errors;
pub mod measurement;
pub mod colors;
pub mod elements;

pub mod prelude {
    pub use crate::element_traits::{Element, Child, RemoveChild};
    pub use crate::elements::Document;
}