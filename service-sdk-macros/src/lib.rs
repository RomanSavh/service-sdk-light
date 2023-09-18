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

#[proc_macro]
pub fn use_grpc_client(_input: TokenStream) -> TokenStream {
    quote::quote! {
        use service_sdk::my_grpc_extensions;
        use service_sdk::my_telemetry;
        use service_sdk::async_trait;
    }
    .into()
}

#[proc_macro]
pub fn use_grpc_server(_input: TokenStream) -> TokenStream {
    quote::quote! {
        use service_sdk::my_grpc_extensions;
        use service_sdk::my_telemetry;
        use service_sdk::futures_core;
        use service_sdk::async_trait::async_trait;
        use service_sdk::my_grpc_extensions::server::with_telemetry;
        use service_sdk::my_grpc_extensions::server::generate_server_stream;
    }
    .into()
}

#[proc_macro]
pub fn use_settings(_input: TokenStream) -> TokenStream {
    quote::quote! {
        use service_sdk::flurl;
        use service_sdk::async_trait::async_trait;
        use service_sdk::serde_yaml;
        use service_sdk::my_settings_reader;
        use service_sdk::macros::SdkSettingsTraits;
        use service_sdk::rust_extensions;
    }
    .into()
}

#[proc_macro]
pub fn use_my_http_server(_input: TokenStream) -> TokenStream {
    quote::quote! {
        use service_sdk::async_trait;
        use service_sdk::my_http_server;
    }
    .into()
}
