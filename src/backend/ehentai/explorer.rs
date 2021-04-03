/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::str;

use hyper::{Uri, Body};
use hyper::client::connect::HttpConnector;
use detour::HttpsConnector;
use select::document::Document;

use super::article::Article;
use super::page::Page;

type ErrorBox = Box<dyn std::error::Error>;

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

type Client = hyper::Client<HttpsConnector<HttpConnector>, Body>;

pub struct Explorer {
    client: Client
}

impl Explorer {
    pub async fn new() -> Result<Explorer, ErrorBox> {
        let https = HttpsConnector::new();
        let client = hyper::Client::builder()
            .build::<_, Body>(https);

        Ok(Self {
            client,
        })
    }

    pub(super) async fn get_bytes(&self, dest: Uri)
        -> Result<Vec<u8>, ErrorBox> {
        let res = self.client.get(dest).await?;
        let bytes = hyper::body::to_bytes(res.into_body()).await?;
    
        Ok(bytes.to_vec())
    }
    
    pub(super) async fn get_html(&self, dest: Uri)
        -> Result<Document, ErrorBox> {
        let res = self.client.get(dest).await?;
        let bytes = hyper::body::to_bytes(res.into_body()).await?;
        let file = str::from_utf8(&bytes)?;
    
        Ok(Document::from(file))
    }

    pub fn search(&self, keyword: &str) -> Page<'_> {
        Page::new(self, 0, format!("f_search={}", percent_encode(keyword)))
    }

    pub async fn article_from_path(&self, path: String)
        -> Result<Article<'_>, ErrorBox> {
        Article::new(self, path).await
    }
}
