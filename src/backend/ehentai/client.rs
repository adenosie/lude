
use std::str;
use std::error::Error;

use hyper::{Uri, Body, Request, Response};
use hyper::client::connect::HttpConnector;
use detour::HttpsConnector;
use select::document::Document;

type ErrorBox = Box<dyn Error>;
type Connector = HttpsConnector<HttpConnector>;

pub struct Client {
    inner: hyper::Client<Connector, Body>,
    cookie: String,
}

impl Client {
    pub fn new() -> Self {
        let https = HttpsConnector::new();
        let inner = hyper::Client::builder()
            .build::<_, Body>(https);

        Self {
            inner,
            cookie: String::new(),
        }
    }

    pub fn with_cookies(member_id: &str, pass_hash: &str)
        -> Self {
        let https = HttpsConnector::new();
        let inner = hyper::Client::builder()
            .build::<_, Body>(https);

        let cookie = format!(
            "ipb_member_id={}; ipb_pass_hash={}",
            member_id, pass_hash
        );

        Self {
            inner,
            cookie,
        }
    }

    async fn get(&self, dest: Uri, mime: &str)
        -> Result<Response<Body>, ErrorBox> {
        let req = if dest.host() == Some("e-hentai.org") {
            Request::get(dest)
                .header("Content-Type", mime)
                .header("Cookie", self.cookie.as_str())
                .body(Body::empty())?
        } else {
            Request::get(dest)
                .header("Content-Type", mime)
                .body(Body::empty())?
        };

        let res = self.inner.request(req).await?;
        Ok(res)
    }

    pub async fn get_image(&self, dest: Uri)
        -> Result<Vec<u8>, ErrorBox> {
        let res = self.get(dest, "image/*").await?;
        let bytes = hyper::body::to_bytes(res.into_body()).await?;
    
        Ok(bytes.to_vec())
    }

    pub async fn get_html(&self, dest: Uri)
        -> Result<Document, ErrorBox> {
        let res = self.get(dest, "text/html").await?;
        let bytes = hyper::body::to_bytes(res.into_body()).await?;
        let file = str::from_utf8(&bytes)?;
    
        Ok(Document::from(file))
    }
}
