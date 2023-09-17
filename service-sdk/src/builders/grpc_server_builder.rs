use std::{
    convert::Infallible,
    net::{IpAddr, SocketAddr},
};

use hyper::Body;

use my_logger::LogEventCtx;
use tokio::task::JoinHandle;
use tonic::{
    body::BoxBody,
    codegen::{http::Request, Service},
    transport::{server::Router, Error, NamedService, Server},
};

const GRPC_PORT: u16 = 8888;
pub struct GrpcServerBuilder {
    server: Option<Router>,
    listen_address: Option<SocketAddr>,
}

impl GrpcServerBuilder {
    pub fn new() -> Self {
        Self {
            server: None,
            listen_address: None,
        }
    }

    pub fn update_listen_endpoint(&mut self, ip: IpAddr, port: u16) {
        self.listen_address = Some(SocketAddr::new(ip, port));
    }

    pub fn add_grpc_service<S>(&mut self, svc: S)
    where
        S: Service<Request<Body>, Response = hyper::Response<BoxBody>, Error = Infallible>
            + NamedService
            + Clone
            + Send
            + 'static,
        S::Future: Send + 'static,
    {
        if self.server.is_none() {
            self.server = Some(Server::builder().add_service(svc));
            return;
        }

        let server = self.server.take().unwrap();
        self.server = Some(server.add_service(svc));
    }

    pub fn build(&mut self) -> GrpcServer {
        let grpc_addr = if let Some(taken) = self.listen_address {
            taken
        } else {
            SocketAddr::new(crate::consts::get_default_ip_address(), GRPC_PORT)
        };
        let mut result = GrpcServer::new(self.server.take().unwrap());

        result.start(grpc_addr);
        result
    }
}

pub struct GrpcServer {
    server: Option<Router>,
    join_handle: Option<JoinHandle<()>>,
}

impl GrpcServer {
    pub fn new(server: Router) -> Self {
        Self {
            server: Some(server),
            join_handle: None,
        }
    }

    pub fn start(&mut self, grpc_addr: SocketAddr) {
        my_logger::LOGGER.write_info(
            "Starting GRPC Server".to_string(),
            format!("GRPC server starts at: {:?}", &grpc_addr),
            LogEventCtx::new(),
        );

        let server = self.server.take().unwrap();
        let result = tokio::spawn(async move {
            server.serve(grpc_addr).await.unwrap();
        });
        self.join_handle = Some(result);
    }
}
