#![allow(dead_code, unused_variables, unused_imports)]

use graphic_cli::elements::*;
use graphic_cli::*;
use graphic_cli::prelude::*;
use std::io::*;
use paste::paste;

/*/*macro_rules! SGML {
    (<$tag_name:ident $($attr_name:ident=($attr_value:expr))*/>) => {
        $tag_name {
            $($attr_name: $attr_value,)*
            children: Vec::new(),
        }
    };
    (<$tag_name:ident $($attr_name:ident=($attr_value:expr))*> { $($child:tt)* } </$end_tag_name:ident>) => {
        if (stringify!($tag_name) == stringify!($end_tag_name)) {
            $tag_name {
                $($attr_name: $attr_value,)*
                children: vec![$(SGML!($child),)*],
            }
        }
    };
}*/

fn main() -> () {
    /*match 2 {
        0 => (),
        1 => {
            match true {
                true => return 2,
                false => 3,
            };
        },
        _ => (),
    }*/
    /*let y: *mut Element = &mut Document::default().unwrap();
    let x: Box<dyn Element> = unsafe { Box::from_raw(Document::default().unwrap() *mut Element) };*/
    println!("{:?}", markup!(
        {<document read=(stdin()) write=(SeekStdout::new())> {
            {<unimplementedChild/>}
        } </document>}
    ).unwrap());
    //SGML!(<document/>);
    
}

fn bar<T: Read>(x: T) {

}