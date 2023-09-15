use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

use my_http::controllers::{
    controllers::{
        actions::{
            DeleteAction, GetAction, GetDescription, HandleHttpRequest, PostAction, PutAction,
        },
        AuthErrorFactory, ControllersAuthorization, ControllersMiddleware,
    },
    swagger::SwaggerMiddleware,
};
use my_http::core::{HttpServerMiddleware, MyHttpServer};
use my_http::is_alive::IsAliveMiddleware;
use rust_extensions::AppStates;

pub struct ServiceHttpServerBuilder {
    server: MyHttpServer,
    controllers: Option<ControllersMiddleware>,
    middlewares: Vec<Arc<dyn HttpServerMiddleware + Send + Sync + 'static>>,
    app_states: Arc<AppStates>,
    app_name: String,
    app_version: String,
}

impl ServiceHttpServerBuilder {
    pub fn new(
        app_states: Arc<AppStates>,
        app_name: &str,
        app_version: &str,
        authorization: Option<ControllersAuthorization>,
        auth_error_factory: Option<Arc<dyn AuthErrorFactory + Send + Sync + 'static>>,
        ip: IpAddr,
    ) -> Self {
        Self {
            server: MyHttpServer::new(SocketAddr::new(ip, 8000)),
            middlewares: vec![],
            controllers: Some(ControllersMiddleware::new(
                authorization,
                auth_error_factory,
            )),
            app_states,
            app_name: app_name.to_string(),
            app_version: app_version.to_string(),
        }
    }

    pub fn update_ip(&mut self, ip: IpAddr) {
        self.server = MyHttpServer::new(SocketAddr::new(ip, 8000));
    }

    pub fn add_middleware(
        &mut self,
        middleware: Arc<dyn HttpServerMiddleware + Send + Sync + 'static>,
    ) -> &mut Self {
        self.middlewares.push(middleware);

        return self;
    }

    pub fn register_get<
        TGetAction: GetAction + HandleHttpRequest + GetDescription + Send + Sync + 'static,
    >(
        &mut self,
        action: TGetAction,
    ) -> &mut Self {
        self.controllers
            .as_mut()
            .unwrap()
            .register_get_action(Arc::new(action));
        return self;
    }

    pub fn register_post<
        TGetAction: PostAction + HandleHttpRequest + GetDescription + Send + Sync + 'static,
    >(
        &mut self,
        action: TGetAction,
    ) -> &mut Self {
        self.controllers
            .as_mut()
            .unwrap()
            .register_post_action(Arc::new(action));
        return self;
    }

    pub fn register_put<
        TGetAction: PutAction + HandleHttpRequest + GetDescription + Send + Sync + 'static,
    >(
        &mut self,
        action: TGetAction,
    ) -> &mut Self {
        self.controllers
            .as_mut()
            .unwrap()
            .register_put_action(Arc::new(action));
        return self;
    }

    pub fn register_delete<
        TGetAction: DeleteAction + HandleHttpRequest + GetDescription + Send + Sync + 'static,
    >(
        &mut self,
        action: TGetAction,
    ) -> &mut Self {
        self.controllers
            .as_mut()
            .unwrap()
            .register_delete_action(Arc::new(action));
        return self;
    }

    pub fn start_http_server(&mut self) {
        let controllers = Arc::new(self.controllers.take().unwrap());
        let swagger_middleware = SwaggerMiddleware::new(
            controllers.clone(),
            self.app_name.clone(),
            self.app_version.clone(),
        );

        let is_alive = IsAliveMiddleware::new(self.app_name.clone(), self.app_version.clone());

        self.server.add_middleware(Arc::new(is_alive));
        self.server.add_middleware(Arc::new(swagger_middleware));
        self.server.add_middleware(controllers.clone());

        for middleware in self.middlewares.iter() {
            self.server.add_middleware(middleware.clone());
        }

        self.server
            .start(self.app_states.clone(), my_logger::LOGGER.clone());
    }
}
