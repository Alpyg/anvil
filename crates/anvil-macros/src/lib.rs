use proc_macro::TokenStream;

mod api;
mod app_state;
mod cron;

#[proc_macro_attribute]
pub fn app_state(args: TokenStream, input: TokenStream) -> TokenStream {
    match app_state::app_state(args.into(), input.into()) {
        Ok(t) => t.into(),
        Err(e) => e.into_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn api(args: TokenStream, input: TokenStream) -> TokenStream {
    match api::api(args.into(), input.into()) {
        Ok(t) => t.into(),
        Err(e) => e.into_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn cron(args: TokenStream, input: TokenStream) -> TokenStream {
    match cron::cron(args.into(), input.into()) {
        Ok(t) => t.into(),
        Err(e) => e.into_compile_error().into(),
    }
}
