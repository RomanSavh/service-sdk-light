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
       Arc<impl MyTelemetrySettings + ServiceInfo #(#traits)* + Send + Sync + 'static>
    };

    result.into()
}

#[proc_macro_derive(SdkSettingsTraits)]
pub fn generate_sdk_settings_traits(_input: TokenStream) -> TokenStream {
    quote::quote! {
    #[async_trait::async_trait]
    impl service_sdk::ServiceInfo for SettingsReader {
        fn get_service_name(&self) -> rust_extensions::StrOrString<'static> {
            env!("CARGO_PKG_NAME").into()
        }
        fn get_service_version(&self) -> rust_extensions::StrOrString<'static> {
            env!("CARGO_PKG_VERSION").into()
        }
    }

        }
    .into()
}
