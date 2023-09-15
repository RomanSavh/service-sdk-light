extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro]
pub fn generate_settings_signature(_item: TokenStream) -> TokenStream {
    let mut signature = vec![
        "MyTelemetrySettings",
        "Send",
        "Sync",
        "ServiceInfo",
        "SeqSettings",
        "'static",
    ];

    if cfg!(feature = "service-bus") {
        signature.push("MyServiceBusSettings");
    }

    if cfg!(feature = "no-sql-reader") {
        signature.push("MyNoSqlTcpConnectionSettings");
    }

    if cfg!(feature = "no-sql-writer") {
        signature.push("MyNoSqlWriterSettings");
    }

    let result = format!("Arc<impl {}>", signature.join(" + ").to_string());

    return result.parse().unwrap();
}
