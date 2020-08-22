macro_rules! export_element {
    ($name:ident) => {
        mod $name;
        pub use $name::*;
    };
}

export_element!(document);