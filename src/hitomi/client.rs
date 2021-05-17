/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

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
}

impl Client {
    pub fn new() -> Self {
        let https = HttpsConnector::new();
        let inner = hyper::Client::builder()
            .build::<_, Body>(https);

        Self {
            inner,
        }
    }

    pub async fn get(&self, dest: Uri) -> Result<Response<Body>, ErrorBox> {
        Ok(self.inner.get(dest).await?)
    }

    pub async fn request(&self, req: Request<Body>) -> Result<Response<Body>, ErrorBox> {
        Ok(self.inner.request(req).await?)
    }

    pub async fn get_html(&self, dest: Uri) -> Result<Document, ErrorBox> {
        let res = self.get(dest).await?;
        let bytes = hyper::body::to_bytes(res.into_body()).await?;
        let file = str::from_utf8(&bytes)?;

        Ok(Document::from(file))
    }
}
