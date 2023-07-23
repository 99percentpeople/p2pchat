use super::message::{FileResponse, Request, Response};
use async_trait::async_trait;
use derive_more::From;
use futures::{AsyncRead, AsyncWrite, AsyncWriteExt};
use libp2p::{
    core::upgrade::{read_length_prefixed, read_varint, write_length_prefixed, write_varint},
    gossipsub::{self, TopicHash},
    mdns, request_response,
    swarm::{keep_alive, NetworkBehaviour},
};
use tokio::io;

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "ComposedEvent")]
pub struct ComposedBehaviour {
    pub request_response: request_response::Behaviour<FileExchangeCodec>,
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
    pub keep_alive: keep_alive::Behaviour,
}

#[derive(Debug, From)]
pub enum ComposedEvent {
    RequestResponse(request_response::Event<FileRequest, FileResponse>),
    Gossipsub(gossipsub::Event),
    Mdns(mdns::Event),
    KeepAlive(void::Void),
}
// Simple file exchange protocol
#[derive(Debug, Clone)]
pub struct FileExchangeProtocol();
#[derive(Clone, Default)]
pub struct FileExchangeCodec();

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileRequest(pub Request);

impl AsRef<str> for FileExchangeProtocol {
    fn as_ref(&self) -> &str {
        "file-exchange-protocol"
    }
}

#[async_trait]
impl request_response::Codec for FileExchangeCodec {
    type Protocol = FileExchangeProtocol;
    type Request = FileRequest;
    type Response = FileResponse;

    async fn read_request<T>(
        &mut self,
        _: &FileExchangeProtocol,
        io: &mut T,
    ) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        let data = read_length_prefixed(io, 1_000_000).await?;

        if data.is_empty() {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }

        let space_pos = data
            .iter()
            .position(|&b| b == b' ')
            .ok_or::<io::Error>(io::ErrorKind::InvalidData.into())?;
        let string_part = std::str::from_utf8(&data[0..space_pos])
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.to_string()))?;

        match string_part {
            "/file" => serde_json::from_slice(&data[space_pos + 1..]).map_or_else(
                |err| Err(io::Error::new(io::ErrorKind::InvalidData, err.to_string())),
                |file| Ok(FileRequest(Request::File(file))),
            ),
            "/group" => {
                let topic_hash =
                    TopicHash::from_raw(std::str::from_utf8(&data[space_pos + 1..]).unwrap());
                Ok(FileRequest(Request::Group(topic_hash)))
            }
            "/user" => serde_json::from_slice(&data[space_pos + 1..]).map_or_else(
                |err| Err(io::Error::new(io::ErrorKind::InvalidData, err.to_string())),
                |peer| Ok(FileRequest(Request::User(peer))),
            ),
            err => Err(io::Error::new(io::ErrorKind::InvalidData, err.to_string())),
        }
    }

    async fn read_response<T>(
        &mut self,
        _: &FileExchangeProtocol,
        io: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        let data = read_length_prefixed(io, 500_000_000).await?; // update transfer maximum
        if data.is_empty() {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }
        let space_pos = data.iter().position(|&b| b == b' ').unwrap();
        let string_part = std::str::from_utf8(&data[0..space_pos]).unwrap();
        match string_part {
            "/file" => Ok(FileResponse(Response::File(data[space_pos + 1..].to_vec()))),
            "/group" => serde_json::from_slice(&data[space_pos + 1..]).map_or_else(
                |err| Err(io::Error::new(io::ErrorKind::InvalidData, err.to_string())),
                |pair| Ok(FileResponse(Response::Group(pair))),
            ),
            "/error" => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                std::str::from_utf8(&data[space_pos + 1..]).unwrap(),
            )),
            "/user" => serde_json::from_slice(&data[space_pos + 1..]).map_or_else(
                |err| Err(io::Error::new(io::ErrorKind::InvalidData, err.to_string())),
                |peer| Ok(FileResponse(Response::User(peer))),
            ),
            _ => Err(io::ErrorKind::InvalidData.into()),
        }
    }

    async fn write_request<T>(
        &mut self,
        _: &FileExchangeProtocol,
        io: &mut T,
        FileRequest(data): FileRequest,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let req = match data {
            Request::File(file) => {
                let data = serde_json::to_vec(&file).unwrap();
                [b"/file ", data.as_slice()].concat()
            }
            Request::Group(topic_hash) => [b"/group ", topic_hash.as_str().as_bytes()].concat(),
            Request::User(peer) => {
                let data = serde_json::to_vec(&peer).unwrap();
                [b"/user ", data.as_slice()].concat()
            }
        };
        write_length_prefixed(io, req).await?;
        io.close().await?;

        Ok(())
    }

    async fn write_response<T>(
        &mut self,
        _: &FileExchangeProtocol,
        io: &mut T,
        FileResponse(resp): FileResponse,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let resp_data = match resp {
            Response::File(data) => [b"/file ", data.as_slice()].concat(),
            Response::Group(pair) => {
                [b"/group ", serde_json::to_vec(&pair).unwrap().as_slice()].concat()
            }
            Response::User(user) => {
                [b"/user ", serde_json::to_vec(&user).unwrap().as_slice()].concat()
            }
        };
        write_length_prefixed(io, resp_data).await?;
        io.close().await?;

        Ok(())
    }
}
