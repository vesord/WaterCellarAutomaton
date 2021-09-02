#![recursion_limit="128"]

use proc_macro::TokenStream;
use quote::{quote};
use syn;
use syn::__private::TokenStream2;

#[proc_macro_derive(VertexAttribPointers, attributes(location))]
pub fn vertex_attrib_pointers_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_vertex_attrib_pointer(&ast)
}

fn impl_vertex_attrib_pointer(ast: &syn::DeriveInput) -> TokenStream {
    let ident = &ast.ident;
    let generics = &ast.generics;
    let where_clause = &ast.generics.where_clause;

    let fields_vertex_attrib_pointer
        = generate_vertex_attrib_pointer_calls(&ast.data);

    let gen =  quote!{
        impl #ident #generics #where_clause {
            pub fn vertex_attrib_pointers(gl: &::gl::Gl) {
                let stride = ::std::mem::size_of::<Self>();
                let offset = 0;
                #(#fields_vertex_attrib_pointer)*
            }
        }
    };
    // panic!("code = {:#?}",  gen.to_string());
    gen.into()
}

fn generate_vertex_attrib_pointer_calls(data: &syn::Data) -> Vec<TokenStream2> {
    match data {
        &syn::Data::Enum(_)
        => panic!("VertexAttribPointers can not be implemented for enums"),
        &syn::Data::Struct(ref s) => {
            s.fields.iter()
                .map(generate_struct_field_vertex_attrib_pointer_call)
                .collect()
        },
        _ => panic!("VertexAttribPointers can not be implemented for this type"),
    }
}

fn generate_struct_field_vertex_attrib_pointer_call(field: &syn::Field) -> TokenStream2 {
    let field_name = match field.ident {
        Some(ref i) => format!("{}", i),
        None => String::from(""),
    };
    let location_attr = field.attrs
        .iter()
        .filter(|a|
            a.path.get_ident().unwrap() == "location" && !a.tokens.is_empty())
        .next()
        .unwrap_or_else(|| panic!(
            "Field {:?} is missing #[location = ?] attribute", field_name
        ));

    let attr_meta: syn::Meta = location_attr.parse_meta()
        .unwrap_or_else(|e| panic!("{:?}", e));
    let meta_name = match attr_meta {
        syn::Meta::NameValue(meta_name) => meta_name,
        _ => panic!("Field {:?} does not match #[location = ?] pattern", field_name),
    };
    let location_value = match meta_name.lit {
        syn::Lit::Int(val) => val.base10_parse::<usize>().unwrap_or_else(
            |e| panic!("Unable to parse 'location' param in field {:?} (usize param needed). Message: {:?}", field_name, e)
        ),
        _ => panic!("In field {:?} 'location' attribute has not int parameter", field_name),
    };

    let field_ty = &field.ty;

    let gen = quote! {
        let location = #location_value;
        unsafe {
            #field_ty::vertex_attrib_pointer(gl, stride, location, offset);
        }
        let offset = offset + ::std::mem::size_of::<#field_ty>();
    };

    gen
}