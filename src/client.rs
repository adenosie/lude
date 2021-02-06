use std::future::Future;
use std::error::Error;
use tokio::task::JoinHandle;
use tokio::net::TcpStream;
use tokio_native_tls::{TlsConnector, native_tls};
use hyper::http;
use hyper::{Body, Request, Response};
use hyper::client::conn::{Builder, SendRequest};

use crate::detour::Detour;

struct Client {
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

    pub fn get(&mut self, path: &str)
        -> http::Result<impl Future<Output = hyper::Result<Response<Body>>>> {
        let req = Request::builder()
            .method("GET")
            .uri(path)
            .header("Host", &self.host)
            .body(Body::from(""))?;

        Ok(self.send_request(req))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::error::Error;
    async fn conn_client(host: &str) -> Result<(), Box<dyn Error>> {
        let mut client = Client::new(host).await?;
        let res = client.get("/")?.await?;

        let status = res.status();
        println!("{}: {}", host, status);
        assert!(!status.is_client_error());

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
