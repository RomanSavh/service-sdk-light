use std::{net::SocketAddr, sync::Arc};

use is_alive_middleware::IsAliveMiddleware;
use my_http_server::{HttpServerMiddleware, MyHttpServer};
use my_http_server_controllers::{
    controllers::{
        actions::{
            DeleteAction, GetAction, GetDescription, HandleHttpRequest, PostAction, PutAction,
        },
        AuthErrorFactory, ControllersAuthorization, ControllersMiddleware,
    },
    swagger::SwaggerMiddleware,
};
use rust_extensions::AppStates;

use crate::{SERVICE_APP_NAME, SERVICE_APP_VERSION};

pub struct ServiceHttpServer {
    server: MyHttpServer,
    controllers: Option<ControllersMiddleware>,
    middlewares: Vec<Arc<dyn HttpServerMiddleware + Send + Sync + 'static>>,
    app_states: Arc<AppStates>,
}

impl ServiceHttpServer {
    pub fn new(
        app_states: Arc<AppStates>,
        authorization: Option<ControllersAuthorization>,
        auth_error_factory: Option<Arc<dyn AuthErrorFactory + Send + Sync + 'static>>,
    ) -> Self {
        Self {
            server: MyHttpServer::new(SocketAddr::from(([0, 0, 0, 0], 8000))),
            middlewares: vec![],
            controllers: Some(ControllersMiddleware::new(
                authorization,
                auth_error_factory,
            )),
            app_states,
        }
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
            SERVICE_APP_NAME.to_string(),
            SERVICE_APP_VERSION.to_string(),
        );

        let is_alive = IsAliveMiddleware::new(
            SERVICE_APP_NAME.to_string(),
            SERVICE_APP_VERSION.to_string(),
        );

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
