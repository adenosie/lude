/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::str;
use std::sync::Arc;
use std::error::Error;

use super::client::Client;
use super::article::Article;
use super::page::Page;

type ErrorBox = Box<dyn Error>;

#[derive(Clone)]
pub struct Explorer {
    client: Arc<Client>,
}

impl Explorer {
    pub fn new() -> Self {
        let client = Client::new();

        Self {
            client: Arc::new(client),
        }
    }

    pub fn with_cookies(member_id: &str, pass_hash: &str) -> Self {
        let mut client = Client::new();
        client.set_cookies(member_id, pass_hash);

        Self {
            client: Arc::new(client),
        }
    }

    pub fn search(&self, keyword: &str) -> Page {
        Page::new(self.client.clone(), 0, keyword)
    }

    pub async fn article_from_path(&self, path: String)
        -> Result<Article, ErrorBox> {
        Article::new(self.client.clone(), path).await
    }
}
