/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use std::task::{Poll, Context};
use std::str;

use tokio_stream::{Stream, StreamExt};
use hyper::{Uri, Body};
use hyper::client::connect::HttpConnector;
use detour::HttpsConnector;
use select::document::Document;

use super::article::{Article, Draft};
use super::page::Page;
use super::tag::{TagMap, TagKind, ArticleKind};
use super::parser;

type ErrorBox = Box<dyn Error>;

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

    // TODO
    //
    // pub fn save_images(&self, article: Article)
    //     -> impl Future<Output = Result<Vec<Vec<u8>>, ErrorBox>> {
    //     let client = self.client.clone();

    //     async move {
    //         let mut res = Vec::new();

    //         for path in &article.images {
    //             let doc = get_html(&client, path.parse()?).await?;
    //             let path = parser::image(&doc)?;

    //             let image = get_bytes(&client, path.parse()?).await?;
    //             res.push(image);
    //         }

    //         Ok(res)
    //     }
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn search() {
        let mut explorer = Explorer::new().await.unwrap();

        let mut page = explorer.search("language:korean").skip(1).take(2);

        while let Some(mut list) = page.try_next().await.unwrap() {
            for draft in list.into_iter().take(3) {
                let article = draft.load().await.unwrap();
                println!("{} pages", article.meta().length);
            }
        }

        // let article = explorer.article(list.pop().unwrap()).await.unwrap();

        // this takes too long...
        // let images = explorer.save_images(article).await.unwrap();
    }
}
