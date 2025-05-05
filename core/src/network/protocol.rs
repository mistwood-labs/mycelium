use crate::proto;
use async_trait::async_trait;
use futures::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use libp2p::request_response;
use prost::Message;
use tokio::io;

#[derive(Clone)]
pub struct ReactionProtocol;

impl AsRef<str> for ReactionProtocol {
    fn as_ref(&self) -> &str {
        "/mycelium/reaction/1.0.0"
    }
}

#[derive(Clone)]
pub struct ReactionCodec;

#[async_trait]
impl request_response::Codec for ReactionCodec {
    type Protocol = ReactionProtocol;
    type Request = proto::SignedReaction;
    type Response = proto::SignedAck;

    async fn read_request<T>(
        &mut self,
        _protocol: &ReactionProtocol,
        io: &mut T,
    ) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        io.read_to_end(&mut buf).await?;
        proto::SignedReaction::decode(buf.as_slice())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    async fn read_response<T>(
        &mut self,
        _protocol: &ReactionProtocol,
        io: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        io.read_to_end(&mut buf).await?;
        proto::SignedAck::decode(buf.as_slice())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    async fn write_request<T>(
        &mut self,
        _protocol: &ReactionProtocol,
        io: &mut T,
        req: Self::Request,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let mut buf = Vec::new();
        req.encode(&mut buf)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        io.write_all(&buf).await?;
        io.close().await?;
        Ok(())
    }

    async fn write_response<T>(
        &mut self,
        _protocol: &ReactionProtocol,
        io: &mut T,
        res: Self::Response,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let mut buf = Vec::new();
        res.encode(&mut buf)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        io.write_all(&buf).await?;
        io.close().await?;
        Ok(())
    }
}
