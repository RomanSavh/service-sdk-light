extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro]
pub fn generate_settings_signature(_item: TokenStream) -> TokenStream {
    let mut traits = vec![];

    if cfg!(feature = "service-bus") {
        traits.push(quote::quote!(+ MyServiceBusSettings));
    }

    if cfg!(feature = "no-sql-reader") {
        traits.push(quote::quote!(+ MyNoSqlTcpConnectionSettings));
    }

    if cfg!(feature = "no-sql-writer") {
        traits.push(quote::quote!(+ MyNoSqlWriterSettings));
    }

    let result = quote::quote! {
       Arc<impl MyTelemetrySettings + ServiceInfo + SeqSettings #(#traits)* + Send + Sync + 'static>
    };

    println!("Settings configuration: {}", result);

    result.into()
}
