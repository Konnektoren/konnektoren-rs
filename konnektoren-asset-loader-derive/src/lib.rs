use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, LitStr, parse_macro_input};

#[proc_macro_derive(AssetCollection, attributes(asset))]
pub fn derive_asset_collection(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let mut folder = None;
    for attribute in input.attrs {
        if attribute.path().is_ident("asset")
            && let Err(error) = attribute.parse_nested_meta(|meta| {
                if meta.path.is_ident("folder") {
                    folder = Some(meta.value()?.parse::<LitStr>()?);
                    Ok(())
                } else {
                    Err(meta.error("expected `folder = \"...\"`"))
                }
            })
        {
            return error.into_compile_error().into();
        }
    }
    let Some(folder) = folder else {
        return syn::Error::new_spanned(name, "missing `#[asset(folder = \"...\")]`")
            .into_compile_error()
            .into();
    };
    quote! {
        impl ::konnektoren_asset_loader::AssetCollection for #name {
            const FOLDER: &'static str = #folder;
        }
    }
    .into()
}
