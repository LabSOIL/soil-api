// In your proc-macro crate (e.g. my_macros/src/lib.rs):

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields, Lit, Meta};

/// This derive macro generates a "create" model struct
/// with the same fields (and types) as the original struct,
/// skipping fields annotated with #[update = false].
#[proc_macro_derive(ToCreateModel, attributes(update))]
pub fn to_create_model(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    // Name the generated struct by appending "Create" to the original name.
    let create_name = format_ident!("{}Create", name);

    // Only support structs with named fields.
    let fields = if let Data::Struct(data) = input.data {
        if let Fields::Named(named) = data.fields {
            named.named
        } else {
            panic!("ToCreateModel only supports structs with named fields");
        }
    } else {
        panic!("ToCreateModel can only be derived for structs");
    };

    // Filter out fields that are marked with #[update = false].
    let filtered_fields = fields
        .into_iter()
        .filter(|field| {
            // Check if any attribute named `update` is set to false.
            !field.attrs.iter().any(|attr| {
                if attr.path.is_ident("update") {
                    if let Ok(Meta::NameValue(nv)) = attr.parse_meta() {
                        if let Lit::Bool(lit_bool) = nv.lit {
                            return !lit_bool.value; // skip if update is false
                        }
                    }
                }
                false
            })
        })
        .map(|field| {
            let ident = field.ident;
            let ty = field.ty;
            quote! {
                pub #ident: #ty
            }
        });

    // Generate the create model struct with extra derives if desired.
    let expanded = quote! {
        #[derive(Serialize, Deserialize, ToSchema)]
        pub struct #create_name {
            #(#filtered_fields),*
        }
    };

    TokenStream::from(expanded)
}

/// This derive macro generates an "update" model struct.
/// Each field is wrapped in Option<Option<T>> (using serde_with’s double option)
/// so that you can differentiate between “not provided” and “provided as null.”
#[proc_macro_derive(ToUpdateModel, attributes(update))]
pub fn to_update_model(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree.
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    // Name the generated struct by appending "Update" to the original name.
    let update_name = format_ident!("{}Update", name);

    // Only support structs with named fields.
    let fields = if let Data::Struct(data) = input.data {
        if let Fields::Named(named) = data.fields {
            named.named
        } else {
            panic!("ToUpdateModel only supports structs with named fields");
        }
    } else {
        panic!("ToUpdateModel can only be derived for structs");
    };

    // Filter out fields that are marked with #[update = false].
    let filtered_fields = fields
        .into_iter()
        .filter(|field| {
            !field.attrs.iter().any(|attr| {
                if attr.path.is_ident("update") {
                    if let Ok(Meta::NameValue(nv)) = attr.parse_meta() {
                        if let Lit::Bool(lit_bool) = nv.lit {
                            return !lit_bool.value;
                        }
                    }
                }
                false
            })
        })
        .map(|field| {
            let ident = field.ident;
            let ty = field.ty;
            // Wrap the type in Option<Option<...>> and add serde attributes.
            quote! {
                #[serde(
                    default,
                    skip_serializing_if = "Option::is_none",
                    with = "::serde_with::rust::double_option"
                )]
                pub #ident: Option<Option<#ty>>
            }
        });

    // Generate the update model struct with extra derives if desired.
    let expanded = quote! {
        #[derive(Serialize, Deserialize, ToSchema)]
        pub struct #update_name {
            #(#filtered_fields),*
        }
    };

    TokenStream::from(expanded)
}
