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

use super::article::{EhArticle, EhArticleKind, EhPendingArticle};
use super::tag::{EhTagMap, EhTagKind};
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

fn get_bytes(client: &Client, dest: Uri)
    -> impl Future<Output = Result<Vec<u8>, ErrorBox>> + '_ {
    async move {
        let res = client.get(dest).await?;
        let bytes = hyper::body::to_bytes(res.into_body()).await?;

        Ok(bytes.to_vec())
    }
}

fn get_html(client: &Client, dest: Uri)
    -> impl Future<Output = Result<Document, ErrorBox>> + '_ {
    async move {
        let res = client.get(dest).await?;
        let bytes = hyper::body::to_bytes(res.into_body()).await?;
        let file = str::from_utf8(&bytes)?;

        Ok(Document::from(file))
    }
}

pub struct Page<'a> {
    client: &'a Client,
    page: usize,
    len: Option<usize>, // TODO
    query: String,

    task: Option<Pin<Box<dyn Future<Output = Result<Document, ErrorBox>> + 'a>>>
}

impl<'a> Page<'a> {
    pub(super) fn new(client: &'a Client, page: usize, query: String) -> Self {
        Self {
            client,
            page,
            len: None,
            query,
            task: None
        }
    }

    fn uri(&self) -> Result<Uri, impl Error> {
        Uri::builder()
            .scheme("https")
            .authority("e-hentai.org")
            .path_and_query(format!("?page={}&{}", self.page, self.query))
            .build()
    }

    pub fn page(&self) -> usize {
        self.page
    }

    pub fn set_page(&mut self, page: usize) {
        self.page = page;
    }
}

impl<'a> Stream for Page<'a> {
    type Item = Result<Vec<EhPendingArticle>, ErrorBox>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>)
        -> Poll<Option<Self::Item>> {
        if self.len.filter(|len| len <= &self.page).is_some() {
            return Poll::Ready(None);
        }

        let _self = self.get_mut();

        if _self.task.is_none() {
            _self.task = Some(Box::pin(get_html(_self.client, _self.uri()?)));
        }

        if let Some(ref mut task) = _self.task {
            task.as_mut().poll(cx).map(|res| {
                _self.task = None;

                // Result<Document, _> => Option<Result<Vec<_>, _>> (in single line!)
                res.map_or_else(|e| Some(Err(e)), |doc| parser::parse_list(&doc).transpose())
            })
        } else {
            // compile error if commented out
            unreachable!()
        }
    }
}

pub struct EhExplorer {
    client: Client
}

impl EhExplorer {
    pub fn new()
        -> impl Future<Output = Result<EhExplorer, ErrorBox>> {
        async {
            let https = HttpsConnector::new();
            let client = hyper::Client::builder()
                .build::<_, Body>(https);

            Ok(Self {
                client,
            })
        }
    }

    pub fn search(&self, keyword: &str) -> Page<'_> {
        Page::new(&self.client, 0, format!("f_search={}", keyword))
    }

    pub fn article(&self, pending: EhPendingArticle)
        -> impl Future<Output = Result<EhArticle, ErrorBox>> + '_ {
        // one page shows 40 images at max
        let page_len: usize = (pending.length - 1) / 40 + 1;

        async move {
            let doc = get_html(&self.client, pending.path.parse()?).await?;
            let mut article = parser::parse_article_info(&doc)?;

            let mut vec = parser::parse_image_list(&doc)?;
            article.images.append(&mut vec);

            // TODO: this could be done async
            for i in 1..page_len {
                let doc = get_html(
                    &self.client,
                    format!("{}?p={}", pending.path, i).parse()?
                ).await?;

                let mut vec = parser::parse_image_list(&doc)?;
                article.images.append(&mut vec);
            }

            Ok(article)
        }
    }

    pub fn save_images(&self, article: EhArticle)
        -> impl Future<Output = Result<Vec<Vec<u8>>, ErrorBox>> + '_ {
        async move {
            let mut res = Vec::new();

            for path in &article.images {
                let doc = get_html(&self.client, path.parse()?).await?;
                let path = parser::parse_image(&doc)?;

                let image = get_bytes(&self.client, path.parse()?).await?;
                res.push(image);
            }

            Ok(res)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn search() {
        let mut explorer = EhExplorer::new().await.unwrap();

        let mut page = explorer.search("language:korean").take(2);

        while let Some(list) = page.try_next().await.unwrap() {
            list.iter().for_each(|pend| println!("{}", pend.title));
        }

        // let article = explorer.article(list.pop().unwrap()).await.unwrap();

        // this takes too long...
        // let images = explorer.save_images(article).await.unwrap();
    }

    /*
    async fn ideal() -> Result<(), Box<dyn Error>> {
        let explorer = EhExplorer::new().await?;

        let search = explorer.search("artist:hota.");

        while let Some(page) = search.try_next().await? {
            for pending in page.iter() {
                println!("{}", pending.title());
                let article = pending.load_into().await?;
                assert!(article.tags().has("artist:hota."));
            }
        }

        let article = explorer.from_path("/g/1556174/cfe385099d/").await?;
        assert_eq!(
            article.title(), 
            "(C97) [Bad Mushrooms (Chicke III, 4why)] \
            Nibun no Yuudou | 2등분의 유혹 \
            (Gotoubun no Hanayome) [Korean] [Team Edge]"
        );
    }
    */
}
