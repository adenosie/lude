/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::future::Future;
use std::task::{Context, Poll};
use std::error::Error;
use std::pin::Pin;

use tokio_stream::Stream;
use hyper::Uri;
use select::document::Document;

use super::explorer::Explorer;
use super::article::Draft;
use super::parser;

type ErrorBox = Box<dyn Error>;

pub struct Page<'a> {
    explorer: &'a Explorer,
    page: usize,
    results: Option<usize>,
    query: String,

    // what a long type...
    task: Option<Pin<Box<dyn Future<Output = Result<Document, ErrorBox>> + 'a>>>
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
    type Item = Result<Vec<Draft<'a>>, ErrorBox>;

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

                    match parser::article_list(&doc) {
                        Ok(Some(vec)) => Some(Ok(vec
                                .into_iter()
                                .map(|meta| Draft::new(_self.explorer, meta))
                                .collect())),
                        Ok(None) => None,
                        Err(e) => Some(Err(e))
                    }
                })
            })
        } else {
            // compile error if commented out
            unreachable!()
        }
    }
}

