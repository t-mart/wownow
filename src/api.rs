/// TCP API calls to the Blizzard TACT server
use crate::response::{
    base::Error as ResponseError, summary::Response as SummaryResponse,
    versions::Response as VersionsResponse,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

const SOCKET_ADDR: (&str, u16) = ("us.version.battle.net", 1119);

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error(transparent)]
    Response(#[from] ResponseError),
}

pub type Result<T> = std::result::Result<T, Error>;

enum Request {
    Summary,
    Versions(String),
}

impl Request {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = match self {
            Self::Summary => b"v2/summary".to_vec(),
            Self::Versions(product) => format!("v2/products/{}/versions", product).into_bytes(),
        };
        bytes.extend(b"\r\n");
        bytes
    }
}

async fn tcp_send_and_recv(request: Request) -> Result<Vec<u8>> {
    let mut stream = TcpStream::connect(SOCKET_ADDR).await?;

    stream.write_all(&request.to_bytes()).await?;

    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer).await?;

    Ok(buffer)
}

pub async fn get_summary() -> Result<SummaryResponse> {
    let response = tcp_send_and_recv(Request::Summary).await?;
    Ok(response.as_slice().try_into()?)
}

pub async fn get_versions(product: &str) -> Result<VersionsResponse> {
    let response = tcp_send_and_recv(Request::Versions(product.to_owned())).await?;
    Ok(response.as_slice().try_into()?)
}
