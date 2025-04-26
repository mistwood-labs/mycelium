use crate::grpc::proto::mycelium_grpc::{
    mycelium_service_server::{MyceliumService, MyceliumServiceServer},
    CopyRequest, CopyResponse, EchoRequest, EchoResponse,
};
use tonic::{Request, Response, Status};

use anyhow::Result;
use std::net::SocketAddr;
use tonic::transport::Server;

#[derive(Debug, Default)]
pub struct MyceliumServiceImpl;

#[tonic::async_trait]
impl MyceliumService for MyceliumServiceImpl {
    async fn copy_post(
        &self,
        request: Request<CopyRequest>,
    ) -> Result<Response<CopyResponse>, Status> {
        println!("Received CopyPost request: {:?}", request);

        // TODO

        Ok(Response::new(CopyResponse {
            success: true,
            message: "Copy successful".to_string(),
        }))
    }

    async fn echo_post(
        &self,
        request: Request<EchoRequest>,
    ) -> Result<Response<EchoResponse>, Status> {
        println!("Received EchoPost request: {:?}", request);

        // TODO

        Ok(Response::new(EchoResponse {
            success: true,
            message: "Echo successful".to_string(),
        }))
    }
}

pub async fn start_server(port: u16) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let service = MyceliumServiceImpl::default();

    println!("gRPC server listening on {}", addr);

    Server::builder()
        .add_service(MyceliumServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
