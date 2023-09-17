extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro]
pub fn generate_settings_signature(_item: TokenStream) -> TokenStream {
    let mut traits = vec![];

    traits.push(quote::quote!(+ SeqSettings));

    #[cfg(feature = "service-bus")]
    traits.push(quote::quote!(+ MyServiceBusSettings));

    #[cfg(feature = "no-sql-reader")]
    traits.push(quote::quote!(+ MyNoSqlTcpConnectionSettings));

    #[cfg(feature = "no-sql-writer")]
    traits.push(quote::quote!(+ MyNoSqlWriterSettings));

    let result = quote::quote! {
       Arc<impl MyTelemetrySettings + ServiceInfo + SeqSettings #(#traits)* + Send + Sync + 'static>
    };

    result.into()
}
