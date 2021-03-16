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

use super::article::{Article, ArticleKind, PendingArticle};
use super::tag::{TagMap, TagKind};
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

pub struct Page<'a> {
    explorer: &'a Explorer,
    page: usize,
    results: Option<usize>,
    query: String,

    // what a long type...
    task: Option<Pin<Box<dyn Future<Output = Result<Document, ErrorBox>>>>>
}

impl<'a> Page<'a> {
    pub(super) fn new(explorer: &'a Explorer, page: usize, query: String) -> Self {
        Self {
            explorer,
            page,
            results: None,
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

    // number of found search results
    pub fn results(&self) -> Option<usize> {
        self.results
    }

    pub fn len(&self) -> Option<usize> {
        const ARTICLES_PER_PAGE: usize = 25;

        // self.results must not be 0
        self.results.map(|n| (n - 1) / ARTICLES_PER_PAGE + 1)
    }

    pub fn page(&self) -> usize {
        self.page
    }

    // Stream doesn't provide nth() nor overloading skip()
    pub fn skip(mut self, n: usize) -> Self {
        self.page += n;
        self.task = None; // do i have to reset?
        self
    }
}

impl<'a> Stream for Page<'a> {
    type Item = Result<Vec<PendingArticle>, ErrorBox>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>)
        -> Poll<Option<Self::Item>> {
        // if self.len().filter(|len| len <= &self.page).is_some() {
        //     return Poll::Ready(None);
        // }

        let _self = self.get_mut();

        if _self.task.is_none() {
            _self.task = Some(
                Box::pin(_self.explorer.get_html(_self.uri()?))
            );
        }

        if let Some(ref mut task) = _self.task {
            task.as_mut().poll(cx).map(|res| {
                _self.task = None;

                res.map_or_else(|e| Some(Err(e)), |doc| {
                    _self.page += 1;
                    match parser::search_results(&doc) {
                        Ok(n) => _self.results = Some(n),
                        Err(e) => return Some(Err(e))
                    }

                    parser::article_list(&doc).transpose()
                })
            })
        } else {
            // compile error if commented out
            unreachable!()
        }
    }
}

pub struct Explorer {
    client: Client
}

impl Explorer {
    pub fn new()
        -> impl Future<Output = Result<Explorer, ErrorBox>> {
        async {
            let https = HttpsConnector::new();
            let client = hyper::Client::builder()
                .build::<_, Body>(https);

            Ok(Self {
                client,
            })
        }
    }

    pub(super) fn get_bytes(&self, dest: Uri)
        -> impl Future<Output = Result<Vec<u8>, ErrorBox>> {
        let task = self.client.get(dest);
        async move {
            let res = task.await?;
            let bytes = hyper::body::to_bytes(res.into_body()).await?;
    
            Ok(bytes.to_vec())
        }
    }
    
    pub(super) fn get_html(&self, dest: Uri)
        -> impl Future<Output = Result<Document, ErrorBox>> {
        let task = self.client.get(dest);
        async move {
            let res = task.await?;
            let bytes = hyper::body::to_bytes(res.into_body()).await?;
            let file = str::from_utf8(&bytes)?;
    
            Ok(Document::from(file))
        }
    }

    pub fn search(&self, keyword: &str) -> Page<'_> {
        Page::new(self, 0, format!("f_search={}", percent_encode(keyword)))
    }

    // FIXME: this can't parse articles where 'offensive for everyone' flag is set
    pub fn article_from_path(&self, path: &str)
        -> impl Future<Output = Result<Article, ErrorBox>> {
        // // every client.get() clones itself internally; cloning client seems to be cheap
        // let _self = self.clone();
        // let path = path.to_owned();

        let task = self.get_html(format!("{}?hc=1", path).parse().unwrap());

        async move {
            let doc = task.await?;
            let mut article = parser::article(&doc)?;

            let mut vec = parser::image_list(&doc)?;
            article.images.append(&mut vec);

            const IMAGES_PER_PAGE: usize = 40;
            let page_len = (article.length - 1) / IMAGES_PER_PAGE + 1;

            // TODO: i'd really not like to clone self;
            // there should be a better way
            //
            // // TODO: this could be done async
            // for i in 1..page_len {
            //     let doc = _self.get_html(
            //         format!("{}?p={}", path, i).parse()?
            //     ).await?;

            //     let mut vec = parser::image_list(&doc)?;
            //     article.images.append(&mut vec);
            // }

            Ok(article)
        }
    }

    pub fn article(&self, pending: PendingArticle)
        -> impl Future<Output = Result<Article, ErrorBox>> {
        self.article_from_path(&pending.path)
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
            list.iter().for_each(|pend| println!("{}", pend.title));
            let article = explorer.article(list.pop().unwrap()).await.unwrap();
        }

        // let article = explorer.article(list.pop().unwrap()).await.unwrap();

        // this takes too long...
        // let images = explorer.save_images(article).await.unwrap();
    }
}
