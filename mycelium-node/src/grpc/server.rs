use crate::grpc::proto::mycelium_grpc::mycelium_service_server::{
    MyceliumService, MyceliumServiceServer,
};
use anyhow::Result;
use std::net::SocketAddr;
use tonic::transport::Server;

#[derive(Debug, Default)]
pub struct MyceliumServiceImpl;

#[tonic::async_trait]
impl MyceliumService for MyceliumServiceImpl {
    async fn copy_post(
        &self,
        _request: tonic::Request<crate::grpc::proto::mycelium_grpc::CopyRequest>,
    ) -> Result<tonic::Response<crate::grpc::proto::mycelium_grpc::CopyResponse>, tonic::Status>
    {
        Ok(tonic::Response::new(
            crate::grpc::proto::mycelium_grpc::CopyResponse {
                success: true,
                message: "Copy successful".to_string(),
            },
        ))
    }

    async fn echo_post(
        &self,
        _request: tonic::Request<crate::grpc::proto::mycelium_grpc::EchoRequest>,
    ) -> Result<tonic::Response<crate::grpc::proto::mycelium_grpc::EchoResponse>, tonic::Status>
    {
        Ok(tonic::Response::new(
            crate::grpc::proto::mycelium_grpc::EchoResponse {
                success: true,
                message: "Echo successful".to_string(),
            },
        ))
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
