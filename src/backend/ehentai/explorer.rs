/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::str;
use std::sync::Arc;
use std::error::Error;

use hyper::{Uri, Body, Request, Response};
use hyper::client::connect::HttpConnector;
use detour::HttpsConnector;
use select::document::Document;

use super::article::Article;
use super::page::Page;

type ErrorBox = Box<dyn Error>;
type Client = hyper::Client<HttpsConnector<HttpConnector>, Body>;

fn percent_encode(from: &str) -> String {
    let mut res = String::new();

    for byte in from.as_bytes() {
        match byte {
            // unreserved characters (MUST NOT be encoded)
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' 
                | b'-' | b'_' | b'.' | b'~' => {
                res.push(*byte as char);
            },
            _ => {
                res.push_str(&format!("%{:02X}", *byte));
            }
        }
    }

    res
}

pub struct Explorer {
    client: Client,
    cookie: String,
}

impl Explorer {
    pub fn owned() -> Explorer {
        let https = HttpsConnector::new();
        let client = hyper::Client::builder()
            .build::<_, Body>(https);

        Self {
            client,
            cookie: String::new(),
        }
    }

    pub fn new() -> Arc<Explorer> {
        Arc::new(Explorer::owned())
    }

    pub async fn with_cookies(member_id: &str, pass_hash: &str)
        -> Arc<Explorer> {
        let mut _self = Explorer::owned();
        _self.cookie = format!(
            "ipb_member_id={}; ipb_pass_hash={}",
            member_id, pass_hash
        );

        Arc::new(_self)
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

        let res = self.client.request(req).await?;
        Ok(res)
    }

    pub(super) async fn get_image(&self, dest: Uri)
        -> Result<Vec<u8>, ErrorBox> {
        let res = self.get(dest, "image/*").await?;
        let bytes = hyper::body::to_bytes(res.into_body()).await?;
    
        Ok(bytes.to_vec())
    }

    pub(super) async fn get_html(&self, dest: Uri)
        -> Result<Document, ErrorBox> {
        let res = self.get(dest, "text/html").await?;
        let bytes = hyper::body::to_bytes(res.into_body()).await?;
        let file = str::from_utf8(&bytes)?;
    
        Ok(Document::from(file))
    }

    pub fn search(self: &Arc<Self>, keyword: &str) -> Page {
        Page::new(self.clone(), 0, &percent_encode(keyword))
    }

    pub async fn article_from_path(self: &Arc<Self>, path: String)
        -> Result<Article, ErrorBox> {
        Article::new(self.clone(), path).await
    }
}
