use std::future::Future;
use std::error::Error;
use tokio::task::JoinHandle;
use hyper::Body;
use hyper::client::conn::{Builder, SendRequest};
use hyper::{Request, Response};
use tokio::net::TcpStream;
use tokio_native_tls::{TlsConnector, native_tls};

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
            // FIXME: why it doesn't work?
            let tls = native_tls::TlsConnector::new()?;
            let tls = TlsConnector::from(tls);

            let sock = TcpStream::connect((host.as_str(), 443)).await?;
            
            // an error occurs HERE when bypassing
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

    fn send_request(&mut self, req: Request<Body>)
        -> impl Future<Output = hyper::Result<Response<Body>>> {
        self.sender.send_request(req)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn conn_google() {
        let mut client = Client::new("google.com").await.unwrap();
        
        let req = Request::builder()
            .method("GET")
            .header("Host", "google.com")
            .body(Body::from(""))
            .unwrap();

        let res = client.send_request(req).await.unwrap();
        println!("google.com: {}", res.status());
    }

    #[tokio::test]
    async fn conn_hitomi() {
        let mut client = Client::new("hitomi.la").await.unwrap();
        
        let req = Request::builder()
            .method("GET")
            .header("Host", "hitomi.la")
            .body(Body::from(""))
            .unwrap();

        let res = client.send_request(req).await.unwrap();
        println!("hitomi.la: {}", res.status());
    }
}
