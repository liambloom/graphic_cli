#![macro_use]

/*macro_rules! export_element {
    ($mod_name:ident, $element_name:ident, $config_name:ident) => {
        pub mod $mod_name;
        pub use $mod_name::Element as $element_name;
        pub use $mod_name::Config as $config_name;
    };
}*/

export_children!(document);
export_children!(child);
//pub mod document;
//export_element!(document, Document, DocumentConfig);