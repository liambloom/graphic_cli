#[macro_export]
macro_rules! markup {
    // Normal tag, optionally with children, which must be encased in curly braces ({})
    (<$tag_name:ident $($attr_name:ident=($attr_value:expr))*> $({$($child:tt)*})? </$end_tag_name:ident>) => {{
        if (stringify!($tag_name) == stringify!($end_tag_name)) {
            paste! {
                [<$tag_name:camel>]::new(
                    [<$tag_name:camel Config>] {
                        $($attr_name: $attr_value,)*
                        $(children: vec![$(Box::new(markup!($child)),)*],)?
                        ..[<$tag_name:camel Config>]::default()
                    }
                )
            }
        }
        else {
            panic!("The begin and end tags must match");
        }
    }};
    // Self closing tag
    (<$tag_name:ident $($attr_name:ident=($attr_value:expr))*/>) => {
        markup!(<$tag_name $($attr_name=($attr_value))*></$tag_name>)
    };
    // Tag with delimiter
    ({<$tag_name:ident $($attr_name:ident=($attr_value:expr))*> $({$($child:tt)*})? </$end_tag_name:ident>}) => {
        markup!(<$tag_name $($attr_name=($attr_value))*> $({$($child)*})? </$end_tag_name>)
    };
    // Self closing tag with delimiter
    ({<$tag_name:ident $($attr_name:ident=($attr_value:expr))*/>}) => {
        markup!(<$tag_name $($attr_name=($attr_value))*/>)
    };
}