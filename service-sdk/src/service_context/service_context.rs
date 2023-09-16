use my_http_server::MyHttpServer;
use my_logger::my_seq_logger::{SeqLogger, SeqSettings};
use my_telemetry::my_telemetry_writer::{MyTelemetrySettings, MyTelemetryWriter};
use rust_extensions::{AppStates, MyTimer, MyTimerTick, StrOrString};

#[cfg(feature = "no-sql-writer")]
use my_no_sql_sdk::data_writer::MyNoSqlWriterSettings;

#[cfg(any(feature = "no-sql-reader", feature = "no-sql-writer"))]
use my_no_sql_sdk::abstractions::MyNoSqlEntity;

#[cfg(feature = "no-sql-reader")]
use my_no_sql_sdk::reader::{
    MyNoSqlDataReader, MyNoSqlTcpConnection, MyNoSqlTcpConnectionSettings,
};

#[cfg(feature = "no-sql-reader")]
use serde::de::DeserializeOwned;

#[cfg(feature = "service-bus")]
use my_service_bus::{
    abstractions::{
        publisher::{MySbMessageSerializer, MyServiceBusPublisher},
        subscriber::{MySbMessageDeserializer, SubscriberCallback, TopicQueueType},
        GetMySbModelTopicId,
    },
    client::{MyServiceBusClient, MyServiceBusSettings},
};

use std::{sync::Arc, time::Duration};

use crate::{GrpcServer, GrpcServerBuilder, HttpServerBuilder, ServiceInfo};

pub struct ServiceContext {
    pub http_server_builder: HttpServerBuilder,
    pub http_server: Option<MyHttpServer>,

    pub telemetry_writer: MyTelemetryWriter,
    pub app_states: Arc<AppStates>,
    pub app_name: StrOrString<'static>,
    pub app_version: StrOrString<'static>,
    pub background_timers: Vec<MyTimer>,
    #[cfg(feature = "no-sql-reader")]
    pub my_no_sql_connection: Arc<MyNoSqlTcpConnection>,
    #[cfg(feature = "service-bus")]
    pub sb_client: Arc<MyServiceBusClient>,
    #[cfg(feature = "grpc")]
    pub grpc_server_builder: Option<GrpcServerBuilder>,
    #[cfg(feature = "grpc")]
    pub grpc_server: Option<GrpcServer>,
}

impl ServiceContext {
    pub fn new(settings: service_sdk_macros::generate_settings_signature!()) -> Self {
        let app_states = Arc::new(AppStates::create_un_initialized());
        let app_name = settings.get_service_name();
        let app_version = settings.get_service_version();

        #[cfg(feature = "no-sql-reader")]
        let my_no_sql_connection = Arc::new(MyNoSqlTcpConnection::new(
            app_name.clone(),
            settings.clone(),
        ));

        #[cfg(feature = "service-bus")]
        let sb_client = Arc::new(MyServiceBusClient::new(
            app_name.clone(),
            app_version.clone(),
            settings.clone(),
            my_logger::LOGGER.clone(),
        ));

        println!("Initialized service context");

        SeqLogger::enable_from_connection_string(settings.clone());

        Self {
            http_server_builder: HttpServerBuilder::new(app_name.clone(), app_version.clone()),
            http_server: None,
            telemetry_writer: MyTelemetryWriter::new(app_name.clone(), settings.clone()),
            app_states,
            #[cfg(feature = "no-sql-reader")]
            my_no_sql_connection,
            #[cfg(feature = "service-bus")]
            sb_client,
            app_name,
            app_version,
            #[cfg(feature = "grpc")]
            grpc_server_builder: None,
            background_timers: vec![],
            grpc_server: None,
        }
    }

    pub fn register_background_job(
        &mut self,
        duration: Duration,
        name: &str,
        job: Arc<dyn MyTimerTick + Send + Sync + 'static>,
    ) {
        let mut timer = MyTimer::new(duration);
        timer.register_timer(name, job);

        self.background_timers.push(timer);
    }

    pub fn configure_http_server(&mut self, config: impl Fn(&mut HttpServerBuilder)) -> &mut Self {
        config(&mut self.http_server_builder);
        self
    }

    pub async fn start_application(&mut self) {
        self.app_states.set_initialized();
        self.telemetry_writer
            .start(self.app_states.clone(), my_logger::LOGGER.clone());
        for timer in self.background_timers.iter() {
            timer.start(self.app_states.clone(), my_logger::LOGGER.clone());
        }
        #[cfg(feature = "no-sql-reader")]
        self.my_no_sql_connection
            .start(my_logger::LOGGER.clone())
            .await;
        #[cfg(feature = "service-bus")]
        self.sb_client.start().await;

        let mut http_server = self.http_server_builder.build();
        http_server.start(self.app_states.clone(), my_logger::LOGGER.clone());
        self.http_server = Some(http_server);

        #[cfg(feature = "grpc")]
        if let Some(mut grpc_server_builder) = self.grpc_server_builder.take() {
            self.grpc_server = Some(grpc_server_builder.build());
        }

        println!("Application is stated");
        self.app_states.wait_until_shutdown().await;
    }

    //ns
    #[cfg(feature = "no-sql-reader")]
    pub async fn get_ns_reader<
        TMyNoSqlEntity: MyNoSqlEntity + Sync + Send + DeserializeOwned + 'static,
    >(
        &self,
    ) -> Arc<dyn MyNoSqlDataReader<TMyNoSqlEntity> + Sync + Send + 'static> {
        return self.my_no_sql_connection.get_reader().await;
    }

    //sb
    #[cfg(feature = "service-bus")]
    pub async fn register_sb_subscribe<
        TModel: GetMySbModelTopicId + MySbMessageDeserializer<Item = TModel> + Send + Sync + 'static,
    >(
        &self,
        callback: Arc<dyn SubscriberCallback<TModel> + Send + Sync + 'static>,
        queue_type: TopicQueueType,
    ) -> &Self {
        self.sb_client
            .subscribe(self.app_name.clone(), queue_type, callback)
            .await;

        self
    }

    #[cfg(feature = "service-bus")]
    pub async fn get_sb_publisher<TModel: MySbMessageSerializer + GetMySbModelTopicId>(
        &self,
        do_retries: bool,
    ) -> MyServiceBusPublisher<TModel> {
        return self.sb_client.get_publisher(do_retries).await;
    }

    #[cfg(feature = "grpc")]
    pub fn configure_grpc_server(&mut self, config: impl Fn(&mut GrpcServerBuilder)) {
        let mut grpc_server_builder = GrpcServerBuilder::new();
        config(&mut grpc_server_builder);
        self.grpc_server_builder = Some(grpc_server_builder);
    }
}
