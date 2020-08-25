#[macro_export]
macro_rules! Markup {
    (<$tag_name:ident $($attr_name:ident=($attr_value:expr))*/>) => {
        paste! {
            [<$tag_name:camel>]::new(
                [<$tag_name:camel Config>] {
                    $($attr_name: $attr_value,)*
                    ..[<$tag_name:camel Config>]::default()
                }
            )
        }
    };
    (<$tag_name:ident $($attr_name:ident=($attr_value:expr))*> {$($child:tt)*} </$end_tag_name:ident>) => {{
        if (stringify!($tag_name) == stringify!($end_tag_name)) {
            //println!("{}", stringify!(SGML!($child)));
            paste! {
                [<$tag_name:camel>]::new(
                    [<$tag_name:camel Config>] {
                        $($attr_name: $attr_value,)*
                        children: vec![$(Box::new(Markup!($child)),)*],
                        ..[<$tag_name:camel Config>]::default()
                    }
                )
            }
        }
        else {
            panic!("The begin and end tags must match");
        }
    }};
    ({<$tag_name:ident $($attr_name:ident=($attr_value:expr))*> {$child:tt} </$end_tag_name:ident>}) => {
        Markup!(<$tag_name $($attr_name=($attr_value))*> {$child} </$end_tag_name>)
    };
    ({<$tag_name:ident $($attr_name:ident=($attr_value:expr))*/>}) => {
        Markup!(<$tag_name $($attr_name=($attr_value))*/>)
    };
}

/*#[macro_export]
macro_rules! remove_delimiter {
    (($($t:tt)*)) => {$($t)*};
    ([$($t:tt)*]) => {$($t)*};
    ({$($t:tt)*}) => {$($t)*};
    ($($t:tt)*) => {$($t)*};
}*/