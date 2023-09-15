use my_http_server::controllers::{AuthErrorFactory, ControllersAuthorization};
use my_logger::my_seq_logger::{SeqLogger, SeqSettings};
use my_telemetry::my_telemetry_writer::{MyTelemetrySettings, MyTelemetryWriter};
use rust_extensions::{AppStates, MyTimer, MyTimerTick};

#[cfg(feature = "grpc-server")]
use hyper::Body;

#[cfg(feature = "no-sql-writer")]
use my_no_sql::data_writer::MyNoSqlWriterSettings;

#[cfg(any(feature = "no-sql-reader", feature = "no-sql-writer"))]
use my_no_sql::server_abstractions::MyNoSqlEntity;

#[cfg(feature = "no-sql-reader")]
use my_no_sql::reader::{MyNoSqlDataReader, MyNoSqlTcpConnection, MyNoSqlTcpConnectionSettings};

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

#[cfg(feature = "grpc-server")]
use std::convert::Infallible;
use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
    time::Duration,
};
#[cfg(feature = "grpc-server")]
use tonic::transport::server::Router;

#[cfg(feature = "grpc-server")]
use tonic::{
    body::BoxBody,
    codegen::{http::Request, Service},
    transport::{NamedService, Server},
};

use crate::{ServiceInfo, ServiceHttpServerBuilder};

pub struct ServiceContext {
    pub http_server: ServiceHttpServerBuilder,
    pub telemetry_writer: MyTelemetryWriter,
    pub app_states: Arc<AppStates>,
    pub app_name: String,
    pub app_version: String,
    pub default_ip: IpAddr,
    pub background_timers: Vec<MyTimer>,
    #[cfg(feature = "no-sql-reader")]
    pub my_no_sql_connection: Arc<MyNoSqlTcpConnection>,
    #[cfg(feature = "service-bus")]
    pub sb_client: Arc<MyServiceBusClient>,
    #[cfg(feature = "grpc-server")]
    pub grpc_router: Option<Router>,
}

impl ServiceContext {
    pub fn new(settings: service_sdk_macros::generate_settings_signature!()) -> Self {
        let app_states = Arc::new(AppStates::create_un_initialized());
        let app_name = settings.get_service_name();
        let app_version = settings.get_service_version();
        let default_ip: IpAddr = IpAddr::from([0, 0, 0, 0]);

        #[cfg(feature = "no-sql-reader")]
        let my_no_sql_connection = Arc::new(MyNoSqlTcpConnection::new(
            app_name.clone(),
            settings.clone(),
        ));

        #[cfg(feature = "service-bus")]
        let sb_client = Arc::new(MyServiceBusClient::new(
            app_name.to_string(),
            app_version.clone(),
            settings.clone(),
            my_logger::LOGGER.clone(),
        ));

        let http_server = ServiceHttpServerBuilder::new(
            app_states.clone(),
            &app_name,
            &app_version,
            None,
            None,
            default_ip.clone(),
        );
        println!("Initialized service context");

        SeqLogger::enable_from_connection_string(settings.clone());

        Self {
            http_server: http_server,
            telemetry_writer: MyTelemetryWriter::new(app_name.clone(), settings.clone()),
            app_states,
            #[cfg(feature = "no-sql-reader")]
            my_no_sql_connection,
            #[cfg(feature = "service-bus")]
            sb_client,
            app_name,
            app_version,
            default_ip,
            #[cfg(feature = "grpc-server")]
            grpc_router: None,
            background_timers: vec![],
        }
    }

    pub fn setup_http(
        &mut self,
        authorization: Option<ControllersAuthorization>,
        auth_error_factory: Option<Arc<dyn AuthErrorFactory + Send + Sync + 'static>>,
    ) -> &mut Self {
        self.http_server = ServiceHttpServerBuilder::new(
            self.app_states.clone(),
            &self.app_name,
            &self.app_version,
            authorization,
            auth_error_factory,
            self.default_ip,
        );
        self
    }

    pub fn update_default_ip(&mut self, ip: IpAddr) -> &mut Self {
        self.default_ip = ip;
        self.http_server.update_ip(ip);
        self
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

    pub fn configurate_http_server(&mut self, config: impl Fn(&mut ServiceHttpServerBuilder)) -> &mut Self {
        config(&mut self.http_server);
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
        self.http_server.start_http_server();

        #[cfg(feature = "grpc-server")]
        {
            let grpc_addr = SocketAddr::new(self.default_ip, 8888);
            self.grpc_router
                .take()
                .expect("Grpc service is not defined. Cannot start grpc server")
                .serve(grpc_addr)
                .await
                .unwrap();
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
    ) -> Arc<dyn MyNoSqlDataReader<TMyNoSqlEntity>> {
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

    #[cfg(feature = "grpc-server")]
    pub fn add_grpc_service<S>(&mut self, svc: S)
    where
        S: Service<Request<Body>, Response = hyper::Response<BoxBody>, Error = Infallible>
            + NamedService
            + Clone
            + Send
            + 'static,
        S::Future: Send + 'static,
    {
        self.grpc_router = Some(Server::builder().add_service(svc));
    }
}