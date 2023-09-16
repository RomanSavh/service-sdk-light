use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

use is_alive_middleware::IsAliveMiddleware;
use my_http_server::controllers::{
    swagger::SwaggerMiddleware,
    {
        actions::{
            DeleteAction, GetAction, GetDescription, HandleHttpRequest, PostAction, PutAction,
        },
        AuthErrorFactory, ControllersAuthorization, ControllersMiddleware,
    },
};
use my_http_server::{HttpServerMiddleware, MyHttpServer};
use rust_extensions::StrOrString;

pub struct HttpServerBuilder {
    listen_address: SocketAddr,

    middlewares: Vec<Arc<dyn HttpServerMiddleware + Send + Sync + 'static>>,

    app_name: String,
    app_version: String,
    controllers: Option<ControllersMiddleware>,
}
impl HttpServerBuilder {
    pub fn new(app_name: StrOrString<'static>, app_version: StrOrString<'static>) -> Self {
        Self {
            listen_address: SocketAddr::new(IpAddr::from([0, 0, 0, 0]), 8000),
            middlewares: vec![],
            controllers: None,
            app_name: app_name.to_string(),
            app_version: app_version.to_string(),
        }
    }

    pub fn set_authorization(&mut self, authorization: ControllersAuthorization) {
        if self.controllers.is_none() {
            self.controllers = Some(ControllersMiddleware::new(Some(authorization), None));
        } else {
            self.controllers
                .as_mut()
                .unwrap()
                .update_authorization_map(authorization);
        }
    }

    pub fn set_auth_error_factory(&mut self, value: impl AuthErrorFactory + Send + Sync + 'static) {
        if self.controllers.is_none() {
            self.controllers = Some(ControllersMiddleware::new(None, Some(Arc::new(value))));
        } else {
            self.controllers
                .as_mut()
                .unwrap()
                .update_auth_error_factory(Arc::new(value));
        }
    }

    pub fn update_listen_endpoint(&mut self, ip: IpAddr, port: u16) {
        self.listen_address = SocketAddr::new(ip, port);
    }

    pub fn add_middleware(
        &mut self,
        middleware: Arc<dyn HttpServerMiddleware + Send + Sync + 'static>,
    ) -> &mut Self {
        self.middlewares.push(middleware);

        return self;
    }

    pub fn register_get(
        &mut self,
        action: impl GetAction + HandleHttpRequest + GetDescription + Send + Sync + 'static,
    ) -> &mut Self {
        if self.controllers.is_none() {
            self.controllers = Some(ControllersMiddleware::new(None, None));
        }
        self.controllers
            .as_mut()
            .unwrap()
            .register_get_action(Arc::new(action));
        return self;
    }

    pub fn register_post(
        &mut self,
        action: impl PostAction + HandleHttpRequest + GetDescription + Send + Sync + 'static,
    ) -> &mut Self {
        if self.controllers.is_none() {
            self.controllers = Some(ControllersMiddleware::new(None, None));
        }
        self.controllers
            .as_mut()
            .unwrap()
            .register_post_action(Arc::new(action));
        return self;
    }

    pub fn register_put(
        &mut self,
        action: impl PutAction + HandleHttpRequest + GetDescription + Send + Sync + 'static,
    ) -> &mut Self {
        if self.controllers.is_none() {
            self.controllers = Some(ControllersMiddleware::new(None, None));
        }
        self.controllers
            .as_mut()
            .unwrap()
            .register_put_action(Arc::new(action));
        return self;
    }

    pub fn register_delete(
        &mut self,
        action: impl DeleteAction + HandleHttpRequest + GetDescription + Send + Sync + 'static,
    ) -> &mut Self {
        if self.controllers.is_none() {
            self.controllers = Some(ControllersMiddleware::new(None, None));
        }
        self.controllers
            .as_mut()
            .unwrap()
            .register_delete_action(Arc::new(action));
        return self;
    }

    pub fn build(&mut self) -> MyHttpServer {
        let mut my_http_server = MyHttpServer::new(self.listen_address);

        if let Some(controllers) = self.controllers.take() {
            let controllers = Arc::new(controllers);
            let swagger_middleware = SwaggerMiddleware::new(
                controllers.clone(),
                self.app_name.clone(),
                self.app_version.clone(),
            );

            my_http_server.add_middleware(Arc::new(swagger_middleware));
            my_http_server.add_middleware(controllers.clone());
        }

        let is_alive = IsAliveMiddleware::new(self.app_name.clone(), self.app_version.clone());

        my_http_server.add_middleware(Arc::new(is_alive));

        for middleware in self.middlewares.drain(..) {
            my_http_server.add_middleware(middleware);
        }

        my_http_server
    }
}
