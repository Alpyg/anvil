use proc_macro2::TokenStream;
use quote::quote;
use syn::{Error, Fields, ItemStruct, parse2};

pub(crate) fn app_state(_args: TokenStream, input: TokenStream) -> Result<TokenStream, Error> {
    let item = parse2::<ItemStruct>(input)?;
    let name = &item.ident;

    let fields = match &item.fields {
        Fields::Named(named) => &named.named,
        other => {
            return Err(Error::new_spanned(
                other,
                "#[app_state] requires a struct with named fields",
            ));
        }
    };

    let layers = fields.iter().map(|f| {
        let id = f.ident.as_ref().unwrap();
        quote! { .layer(::axum::Extension(self.#id.clone())) }
    });

    let inserts = fields.iter().map(|f| {
        let id = f.ident.as_ref().unwrap();
        let ty = &f.ty;
        quote! { services.insert::<#ty>(self.#id.clone()); }
    });

    Ok(quote! {
        #item

        impl #name {
            pub fn inject(&self, router: ::axum::Router) -> ::axum::Router {
                router #(#layers)*
            }

            pub fn services(&self) -> ::anvil_core::Services {
                let mut services = ::anvil_core::Services::new();
                #(#inserts)*
                services
            }
        }
    })
}

