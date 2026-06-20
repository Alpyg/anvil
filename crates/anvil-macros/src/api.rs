use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Error, Ident, ItemFn, LitStr, parenthesized,
    parse::{Parse, ParseStream},
    parse2,
};

struct ApiArgs {
    method: Ident,
    path: LitStr,
}

impl Parse for ApiArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let method = input.parse()?;
        let content;
        parenthesized!(content in input);
        let path = content.parse()?;
        Ok(ApiArgs { method, path })
    }
}

pub(crate) fn api(args: TokenStream, input: TokenStream) -> Result<TokenStream, Error> {
    let ApiArgs { method, path } = parse2::<ApiArgs>(args)?;
    let func = parse2::<ItemFn>(input)?;
    let name = &func.sig.ident;

    Ok(quote! {
        #func
        ::anvil_core::inventory::submit! {
            ::anvil_core::ApiEndpoint {
                register: |router| router.route(
                    #path,
                    ::axum::routing::#method(#name),
                ),
            }
        }
    })
}
