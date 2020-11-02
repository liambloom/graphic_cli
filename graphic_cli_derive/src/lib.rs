use proc_macro::TokenStream;
use quote::quote;
use syn;

#[proc_macro_derive(Parent)]
pub fn parent_derive(input: TokenStream) -> TokenStream {
    // In production code you should use panic! or expect, not unwrap
    let ast = syn::parse(input).unwrap();

    // Having these 
    impl_parent(&ast)
}

fn impl_parent(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl AsParent for #name {
            fn is_parent(&self) -> bool {
                true
            }
            fn as_parent(&self) -> Option<&dyn Parent> {
                Some(self)
            }
            fn as_parent_mut(&mut self) -> Option<&mut dyn Parent> {
                Some(self)
            }
        }
        impl Drop for #name {
            fn drop(&mut self) {
                if !self.as_parent().unwrap().safe_to_drop() {
                    panic!("Cannot drop parent element {}#{}, one of its children still has multiple strong references", stringify!(#name), self.id())
                }
            }
        }
    };
    gen.into()
}

#[proc_macro_derive(NotParent)]
pub fn not_parent_derive(input: TokenStream) -> TokenStream {
    // In production code you should use panic! or expect, not unwrap
    let ast = syn::parse(input).unwrap();

    // Having these 
    impl_not_parent(&ast)
}

fn impl_not_parent(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl AsParent for #name {
            fn is_parent(&self) -> bool {
                false
            }
            fn as_parent(&self) -> Option<&dyn Parent> {
                None
            }
            fn as_parent_mut(&mut self) -> Option<&mut dyn Parent> {
                None
            }
        }
    };
    gen.into()
}

#[proc_macro_derive(Child)]
pub fn child_derive(input: TokenStream) -> TokenStream {
    // In production code you should use panic! or expect, not unwrap
    let ast = syn::parse(input).unwrap();

    // Having these 
    impl_child(&ast)
}

fn impl_child(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl AsChild for #name {
            fn is_child(&self) -> bool {
                true
            }
            fn as_child(&self) -> Option<&dyn Child> {
                Some(self)
            }
            fn as_child_mut(&mut self) -> Option<&mut dyn Child> {
                Some(self)
            }
        }
    };
    gen.into()
}

#[proc_macro_derive(NotChild)]
pub fn not_child_derive(input: TokenStream) -> TokenStream {
    // In production code you should use panic! or expect, not unwrap
    let ast = syn::parse(input).unwrap();

    // Having these 
    impl_not_child(&ast)
}

fn impl_not_child(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl AsChild for #name {
            fn is_child(&self) -> bool {
                false
            }
            fn as_child(&self) -> Option<&dyn Child> {
                None
            }
            fn as_child_mut(&mut self) -> Option<&mut dyn Child> {
                None
            }
        }
    };
    gen.into()
}