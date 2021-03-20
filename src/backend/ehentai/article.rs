/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt;
use std::str::FromStr;
use std::error::Error;
use std::future::Future;
use select::document::Document;

use super::tag::{ArticleKind, TagMap};
use super::explorer::Explorer;
use super::parser;

type ErrorBox = Box<dyn std::error::Error>;

#[derive(Clone)]
pub struct DraftMeta {
    pub kind: ArticleKind,
    pub thumb: String,
    pub posted: String,
    pub path: String,
    pub title: String,
    pub tags: TagMap,
    pub uploader: String,
    pub length: usize,
}

pub struct Draft<'a> {
    explorer: &'a Explorer,
    meta: DraftMeta,
}

impl<'a> Draft<'a> {
    pub(super) fn new(explorer: &'a Explorer, meta: DraftMeta) -> Self {
        Self {
            explorer,
            meta,
        }
    }

    pub fn meta(&self) -> &DraftMeta {
        &self.meta
    }

    pub fn load(self) -> impl Future<Output = Result<Article<'a>, ErrorBox>> + 'a {
        async move {
            let doc = self.explorer.get_html(self.meta.path.parse()?).await?;
            Article::from_html(self.explorer, &doc, self.meta.path)
        }
    }
}


#[derive(Clone)]
pub struct ArticleMeta {
    pub path: String,

    pub title: String,
    pub original_title: String,

    pub kind: ArticleKind,
    pub thumb: String,
    pub uploader: String,
    pub posted: String,
    pub parent: String,
    pub visible: bool, // 'offensive for everyone' flag
    pub language: String,
    pub translated: bool,
    pub file_size: String,
    pub length: usize,
    pub favorited: usize,
    pub rating_count: usize,
    pub rating: f64,

    pub tags: TagMap,
}

pub(super) struct Vote {
    pub(super) score: i64,
    pub(super) voters: Vec<(String, i64)>,
    pub(super) omitted: usize,
}

pub struct Comment {
    pub(super) posted: String,
    pub(super) edited: Option<String>,

    // None if uploader comment
    pub(super) vote: Option<Vote>,

    pub(super) writer: String,
    pub(super) content: String,
}

impl Comment {
    pub fn score(&self) -> Option<i64> {
        self.vote.as_ref().map(|v| v.score)
    }

    pub fn voters(&self) -> Option<&[(String, i64)]> {
        self.vote.as_ref().map(|v| v.voters.as_slice())
    }

    pub fn omitted_voter(&self) -> Option<usize> {
        self.vote.as_ref().map(|v| v.omitted)
    }
}

struct Image {
    // path to gallery viewer toward this image
    // (e.g. https://e-hentai.org/s/432a4627a6/1623741-8)
    link: String,

    // path to actual image file
    // (it's very long; usually form of https://*.*.hath.network/*)
    path: Option<String>,
    data: Option<Vec<u8>>,

    // NOTE: this is tricky, because what we see as 'preview images' of an article
    // is actually made up by chopping one big image to 100 pixels wide.
    // why do this? to reduce the number of requests.
    // preview: Option<Vec<u8>>,
}

impl Image {
    fn new(link: String) -> Self {
        Self {
            link,
            path: None,
            data: None
        }
    }
}

pub struct Article<'a> {
    explorer: &'a Explorer,

    meta: ArticleMeta,
    images: Vec<Image>,
    comments: Vec<Comment>,
}

impl<'a> Article<'a> {
    pub(super) fn from_html(explorer: &'a Explorer, doc: &Document, path: String)
        -> Result<Self, ErrorBox> {
        Ok(Self {
            explorer,
            meta: parser::article(doc, path)?,
            images: parser::image_list(doc)?
                .into_iter()
                .map(Image::new)
                .collect(),
            comments: parser::comments(doc)?,
        })
    }

    pub fn meta(&self) -> &ArticleMeta {
        &self.meta
    }

    async fn load_image_list(&mut self) -> Result<(), ErrorBox> {
        const IMAGES_PER_PAGE: usize = 40;
        let page_len = 1 + (self.meta.length - 1) / IMAGES_PER_PAGE;

        // start from 1 because we've already parsed page 0
        for i in 1..page_len {
            let doc = self.explorer.get_html(
                format!("{}?p={}", self.meta.path, i).parse()?
            ).await?;

            let mut list = parser::image_list(&doc)?
                .into_iter()
                .map(Image::new)
                .collect();

            self.images.append(&mut list);
        }

        Ok(())
    }

    async fn load_image(&mut self, index: usize) -> Result<(), ErrorBox> {
        if index >= self.meta.length {
            return Ok(());
        }

        if index >= self.images.len() {
            self.load_image_list().await?;
        }

        let image = &mut self.images[index];

        if image.data.is_none() {
            let doc = self.explorer.get_html(image.link.parse()?).await?;
            let path = parser::image(&doc)?;
            image.data = Some(self.explorer.get_bytes(path.parse()?).await?);
            image.path = Some(path);
        }

        Ok(())
    }

    pub fn image(&self, index: usize) -> Option<&[u8]> {
        self.images.get(index).and_then(|i| i.data.as_ref().map(Vec::as_slice))
    }

    async fn load_all_comments(&mut self) -> Result<(), ErrorBox> {
        let path = format!("{}?hc=1", self.meta.path).parse()?;
        let doc = self.explorer.get_html(path).await?;
        self.comments = parser::comments(&doc)?;

        Ok(())
    }
}
