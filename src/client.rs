use std::future::Future;
use std::error::Error;
use std::fmt;
use tokio::task::JoinHandle;
use tokio::net::TcpStream;
use tokio_native_tls::{TlsConnector, native_tls};
use hyper::body::{Bytes, HttpBody};
use hyper::{Body, Request, Response, StatusCode};
use hyper::client::conn::{Builder, SendRequest};

use crate::detour::Detour;

#[derive(Debug, Clone)]
pub struct ResponseError(StatusCode);

impl fmt::Display for ResponseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for ResponseError {

}

pub struct Client {
    host: String,
    sender: SendRequest<Body>,
    handle: JoinHandle<()>,
}

impl Client {
    pub fn new(host: &str)
        -> impl Future<Output = Result<Client, Box<dyn Error>>> {
        let host = String::from(host);
        async move {
            let tls = native_tls::TlsConnector::new()?;
            let tls = TlsConnector::from(tls);

            let sock = TcpStream::connect((host.as_str(), 443)).await?;
            let sock = tls.connect(host.as_str(), Detour::new(sock)).await?;
    
            let (sender, conn) = Builder::new()
                .handshake::<_, Body>(sock)
                .await?;

            let host_moved = host.to_owned();

            let handle = tokio::spawn(async move {
                if let Err(e) = conn.await {
                    eprintln!("{}: {}", host_moved, e);
                }
            });

            Ok(Self {
                host,
                sender,
                handle,
            })
        }
    }

    pub fn send_request(&mut self, req: Request<Body>)
        -> impl Future<Output = hyper::Result<Response<Body>>> {
        self.sender.send_request(req)
    }

    // send a GET and return only body. head is ignored
    // does this absolutely needed?
    pub fn get_bytes(&mut self, path: &str)
        -> impl Future<Output = Result<Vec<u8>, Box<dyn Error>>> + '_ {
        let path = path.to_owned();

        async move {
            let req = Request::builder()
                .method("GET")
                .uri(path)
                .header("Host", &self.host)
                .header("Accept", "*/*")
                .body(Body::from(""))?;

            let res = self.send_request(req).await?;
            if !res.status().is_server_error() && !res.status().is_client_error() {
                Ok(res.into_body().data().await.unwrap_or(Ok(Bytes::new()))?.to_vec())
            } else {
                Err(ResponseError(res.status()).into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::error::Error;
    async fn conn_client(host: &str) -> Result<(), Box<dyn Error>> {
        let mut client = Client::new(host).await?;
        let res = client.get_bytes("/").await?;

        Ok(())
    }

    #[tokio::test]
    async fn conn_unblocked() {
        conn_client("google.com").await.unwrap();
        conn_client("example.com").await.unwrap();
        conn_client("naver.com").await.unwrap();
        conn_client("knowhow.or.kr").await.unwrap();
    }

    #[tokio::test]
    async fn conn_blocked() {
        conn_client("e-hentai.org").await.unwrap();
        conn_client("hitomi.la").await.unwrap();
        conn_client("xvideos.com").await.unwrap();
        conn_client("manatoki.net").await.unwrap();
    }
}
