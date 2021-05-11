/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;
use std::error::Error;
use hyper::Uri;

use super::client::Client;
use super::article::Draft;
use super::parser;

type ErrorBox = Box<dyn Error>;

pub struct Page {
    client: Arc<Client>,
    page: usize,
    results: Option<usize>,
    limit: Option<usize>,
    query: String,
}

impl Page {
    pub(super) fn new(client: Arc<Client>, page: usize, keyword: &str) -> Self {
        let query = format!("f_search={}", keyword);

        Self {
            client,
            page,
            results: None,
            limit: None,
            query,
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

        if let Some(lim) = self.limit {
            Some(lim)
        } else if let Some(n) = self.results {
            // self.results must not be 0
            Some((n - 1) / ARTICLES_PER_PAGE + 1)
        } else {
            None
        }
    }

    pub fn page(&self) -> usize {
        self.page
    }

    // both Iterator and Stream suck, so i have to mimic them by myself...
    pub fn skip(mut self, n: usize) -> Self {
        self.page += n;
        self
    }

    pub fn take(mut self, n: usize) -> Self {
        self.limit = Some(self.page + n);
        self
    }

    pub async fn next(&mut self) -> Result<Option<Vec<Draft>>, ErrorBox> {
        if self.len().filter(|len| len <= &self.page).is_some() {
            return Ok(None);
        }

        let doc = self.client.get_html(self.uri()?).await?;
        self.page += 1;
        self.results = Some(parser::search_results(&doc)?);

        if let Some(list) = parser::article_list(&doc)? {
            let list = list
                .into_iter()
                .map(|meta| Draft::new(self.client.clone(), meta))
                .collect();
            
            Ok(Some(list))
        } else {
            Ok(None)
        }
    }
}
