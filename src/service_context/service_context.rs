use std::sync::Arc;

use my_http_server_controllers::controllers::{AuthErrorFactory, ControllersAuthorization};
#[cfg(feature = "no-sql")]
use my_no_sql_server_abstractions::MyNoSqlEntity;
#[cfg(feature = "no-sql")]
use my_no_sql_tcp_reader::{MyNoSqlDataReader, MyNoSqlTcpConnection, MyNoSqlTcpConnectionSettings};
#[cfg(feature = "service-bus")]
use my_service_bus_abstractions::{
    publisher::{MySbMessageSerializer, MyServiceBusPublisher},
    subscriber::{MySbMessageDeserializer, SubscriberCallback, TopicQueueType},
    GetMySbModelTopicId,
};
#[cfg(feature = "service-bus")]
use my_service_bus_tcp_client::{MyServiceBusClient, MyServiceBusSettings};
use my_telemetry_writer::{MyTelemetrySettings, MyTelemetryWriter};
use rust_extensions::AppStates;
#[cfg(feature = "no-sql")]
use serde::de::DeserializeOwned;

use crate::{ServiceHttpServer, SERVICE_APP_NAME, SERVICE_APP_VERSION};

pub struct ServiceContext {
    pub http_server: ServiceHttpServer,
    pub telemetry_writer: MyTelemetryWriter,
    pub app_states: Arc<AppStates>,
    #[cfg(feature = "no-sql")]
    pub my_no_sql_connection: Arc<MyNoSqlTcpConnection>,
    #[cfg(feature = "service-bus")]
    pub sb_client: Arc<MyServiceBusClient>,
}

impl ServiceContext {
    pub fn new(
        #[cfg(all(feature = "service-bus", feature = "no-sql"))] settings: Arc<
            impl MyTelemetrySettings
                + MyNoSqlTcpConnectionSettings
                + MyServiceBusSettings
                + Send
                + Sync
                + 'static,
        >,
        #[cfg(all(not(feature = "no-sql"), all(feature = "service-bus")))] settings: Arc<
            impl MyTelemetrySettings + MyServiceBusSettings + Send + Sync + 'static,
        >,
        #[cfg(all(not(feature = "service-bus"), all(feature = "no-sql")))] settings: Arc<
            impl MyTelemetrySettings + MyNoSqlTcpConnectionSettings + Send + Sync + 'static,
        >,
        #[cfg(all(not(feature = "service-bus"), not(feature = "no-sql")))] settings: Arc<
            impl MyTelemetrySettings + Send + Sync + 'static,
        >,
    ) -> Self {
        let app_states = Arc::new(AppStates::create_un_initialized());

        #[cfg(feature = "no-sql")]
        let my_no_sql_connection = Arc::new(MyNoSqlTcpConnection::new(
            SERVICE_APP_NAME.to_string(),
            settings.clone(),
        ));

        #[cfg(feature = "service-bus")]
        let sb_client = Arc::new(MyServiceBusClient::new(
            SERVICE_APP_NAME,
            SERVICE_APP_VERSION,
            settings.clone(),
            my_logger::LOGGER.clone(),
        ));

        let http_server = ServiceHttpServer::new(app_states.clone(), None, None);

        Self {
            http_server: http_server,
            telemetry_writer: MyTelemetryWriter::new(
                SERVICE_APP_NAME.to_string(),
                settings.clone(),
            ),
            app_states,
            #[cfg(feature = "no-sql")]
            my_no_sql_connection,
            #[cfg(feature = "service-bus")]
            sb_client,
        }
    }

    pub fn setup_http(
        &mut self,
        authorization: Option<ControllersAuthorization>,
        auth_error_factory: Option<Arc<dyn AuthErrorFactory + Send + Sync + 'static>>,
    ) -> &mut Self {
        self.http_server =
            ServiceHttpServer::new(self.app_states.clone(), authorization, auth_error_factory);

        self
    }

    pub fn register_http_routes(&mut self, config: fn(&mut ServiceHttpServer)) -> &mut Self {
        config(&mut self.http_server);
        self
    }

    pub async fn start_application(&mut self) {
        self.app_states.set_initialized();
        self.telemetry_writer
            .start(self.app_states.clone(), my_logger::LOGGER.clone());
        #[cfg(feature = "no-sql")]
        self.my_no_sql_connection
            .start(my_logger::LOGGER.clone())
            .await;
        #[cfg(feature = "service-bus")]
        self.sb_client.start().await;
        self.http_server.start_http_server();
        self.app_states.wait_until_shutdown().await;
    }

    //ns
    #[cfg(feature = "no-sql")]
    pub async fn get_ns_reader<
        TMyNoSqlEntity: MyNoSqlEntity + Sync + Send + DeserializeOwned + 'static,
    >(
        &self,
    ) -> Arc<MyNoSqlDataReader<TMyNoSqlEntity>> {
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
            .subscribe(SERVICE_APP_NAME.to_string(), queue_type, callback)
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
}
