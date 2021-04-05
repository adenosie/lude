/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::str;
use std::sync::Arc;

use hyper::{Uri, Body};
use hyper::client::connect::HttpConnector;
use detour::HttpsConnector;
use select::document::Document;

use super::article::Article;
use super::page::Page;

type ErrorBox = Box<dyn std::error::Error>;
type Client = hyper::Client<HttpsConnector<HttpConnector>, Body>;

pub struct Explorer {
    client: Client
}

impl Explorer {
    pub async fn new() -> Result<Arc<Explorer>, ErrorBox> {
        let https = HttpsConnector::new();
        let client = hyper::Client::builder()
            .build::<_, Body>(https);

        Ok(Arc::new(Self {
            client,
        }))
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

    pub fn search(self: &Arc<Self>, keyword: &str) -> Page {
        Page::new(self.clone(), 0, keyword)
    }

    pub async fn article_from_path(self: &Arc<Self>, path: String)
        -> Result<Article, ErrorBox> {
        Article::new(self.clone(), path).await
    }
}
