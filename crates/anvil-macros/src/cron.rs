use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Error, FnArg, Ident, ItemFn, LitStr, Pat, Token,
    parse::{Parse, ParseStream},
    parse2,
};

struct CronArgs {
    schedule: LitStr,
    timezone: Option<LitStr>,
}

impl Parse for CronArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let schedule: LitStr = input.parse()?;
        let mut timezone = None;
        while input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "timezone" => timezone = Some(input.parse()?),
                other => {
                    return Err(Error::new(
                        key.span(),
                        format!("unknown cron option `{other}` (expected `timezone`)"),
                    ));
                }
            }
        }
        Ok(CronArgs { schedule, timezone })
    }
}

pub(crate) fn cron(args: TokenStream, input: TokenStream) -> Result<TokenStream, Error> {
    let CronArgs { schedule, timezone } = parse2::<CronArgs>(args)?;
    let func = parse2::<ItemFn>(input)?;
    let name = &func.sig.ident;

    let mut idents = Vec::new();
    let mut types = Vec::new();
    for arg in &func.sig.inputs {
        match arg {
            FnArg::Typed(pt) => {
                let ident = match &*pt.pat {
                    Pat::Ident(p) => p.ident.clone(),
                    other => {
                        return Err(Error::new_spanned(
                            other,
                            "#[cron] parameters must be plain identifiers",
                        ));
                    }
                };
                idents.push(ident);
                types.push((*pt.ty).clone());
            }
            FnArg::Receiver(r) => {
                return Err(Error::new_spanned(
                    r,
                    "#[cron] cannot be applied to methods",
                ));
            }
        }
    }

    let tz_setup = match &timezone {
        Some(lit) => quote! { let __tz = ::anvil_core::parse_timezone(#lit)?; },
        None => quote! { let __tz = ::anvil_core::chrono_tz::UTC; },
    };

    Ok(quote! {
        #func

        ::anvil_core::inventory::submit! {
            ::anvil_core::CronJob {
                build: |services| {
                    #( let #idents = <#types as ::anvil_core::FromServices>::from_services(services)?; )*
                    #tz_setup
                    let job = ::anvil_core::tokio_cron_scheduler::JobBuilder::new()
                        .with_timezone(__tz)
                        .with_cron_job_type()
                        .with_schedule(#schedule)?
                        .with_run_async(::std::boxed::Box::new(move |_uuid, _lock| {
                            #( let #idents = ::core::clone::Clone::clone(&#idents); )*
                            ::std::boxed::Box::pin(async move {
                                #name(#(#idents),*).await;
                            })
                        }))
                        .build()?;
                    ::core::result::Result::Ok(job)
                },
            }
        }
    })
}
