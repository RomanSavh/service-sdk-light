extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro]
pub fn generate_settings_signature(_item: TokenStream) -> TokenStream {
    let mut traits = vec![];

    traits.push(quote::quote!(+ SeqSettings));

    #[cfg(feature = "my-service-bus")]
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
    #[async_trait]
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
        use service_sdk::my_grpc_extensions::client::generate_grpc_client;
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
    let mut uses = vec![];

    uses.push(quote::quote!(
        use service_sdk::flurl;
    ));

    #[cfg(feature = "my-service-bus")]
    uses.push(quote::quote!(
        service_sdk::my_service_bus::client::MyServiceBusSettings
    ));

    #[cfg(feature = "postgres")]
    uses.push(quote::quote!(service_sdk::my_postgres::PostgresSettings));

    #[cfg(feature = "no-sql-writer")]
    uses.push(quote::quote!(
        service_sdk::my_no_sql_sdk::data_writer::MyNoSqlWriterSettings
    ));

    #[cfg(feature = "no-sql-reader")]
    uses.push(quote::quote!(
        service_sdk::my_no_sql_sdk::reader::MyNoSqlTcpConnectionSettings
    ));

    #[cfg(feature = "grpc")]
    uses.push(quote::quote!(
        service_sdk::my_grpc_extensions::GrpcClientSettings
    ));

    quote::quote! {
        use service_sdk::async_trait::async_trait;
        use service_sdk::serde_yaml;
        use service_sdk::my_settings_reader;
        use service_sdk::macros::SdkSettingsTraits;
        use service_sdk::rust_extensions;
        use service_sdk::my_logger::my_seq_logger::SeqSettings;
        use service_sdk::my_telemetry::my_telemetry_writer::MyTelemetrySettings;
        #(#uses)*
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

#[proc_macro]
pub fn use_my_postgres(_input: TokenStream) -> TokenStream {
    quote::quote! {
        use service_sdk::my_postgres;
        use service_sdk::my_postgres::macros::*;
        use service_sdk::my_telemetry::MyTelemetryContext;
        use service_sdk::my_logger;
    }
    .into()
}

#[proc_macro]
pub fn use_my_no_sql_entity(_input: TokenStream) -> TokenStream {
    quote::quote! {
        use service_sdk::my_no_sql_sdk;
        use service_sdk::my_no_sql_sdk::macros::my_no_sql_entity;
        use service_sdk::rust_extensions;
    }
    .into()
}

#[proc_macro]
pub fn use_my_sb_entity_protobuf_model(_input: TokenStream) -> TokenStream {
    quote::quote! {
        use service_sdk::my_service_bus;
        use service_sdk::my_service_bus::macros::my_sb_entity_protobuf_model;
        use service_sdk::rust_extensions;
    }
    .into()
}

#[proc_macro]
pub fn use_my_sb_subscriber(_input: TokenStream) -> TokenStream {
    quote::quote! {
        use service_sdk::my_service_bus;
        use service_sdk::rust_extensions;
        use service_sdk::my_logger;
        use service_sdk::my_logger::LogEventCtx;
        use service_sdk::my_telemetry::MyTelemetryContext;
        use service_sdk::my_service_bus::abstractions::subscriber::*;

    }
    .into()
}
