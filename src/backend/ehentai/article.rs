/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::slice;

use super::tag::{ArticleKind, TagMap};
use super::explorer::Explorer;
use super::parser;

type ErrorBox = Box<dyn std::error::Error>;

#[derive(Debug, Clone)]
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

    pub async fn load(self) -> Result<Article<'a>, ErrorBox> {
        Article::new(self.explorer, self.meta.path).await
    }
}


#[derive(Debug, Clone)]
pub struct ArticleMeta {
    pub path: String,

    pub title: String,
    pub original_title: String,

    pub kind: ArticleKind,
    pub thumb: String,
    pub uploader: String,
    pub posted: String,
    pub parent: Option<String>,
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

#[derive(Debug)]
pub(super) struct Vote {
    pub(super) score: i64,
    pub(super) voters: Vec<(String, i64)>,
    pub(super) omitted: usize,
}

#[derive(Debug)]
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

    pub fn voters(&self) -> Option<slice::Iter<'_, (String, i64)>> {
        self.vote.as_ref().map(|v| v.voters.iter())
    }

    pub fn omitted_voter(&self) -> Option<usize> {
        self.vote.as_ref().map(|v| v.omitted)
    }
}

pub struct Article<'a> {
    explorer: &'a Explorer,

    meta: ArticleMeta,
    links: Vec<String>,
    comments: Vec<Comment>,
}

impl<'a> Article<'a> {
    pub(super) async fn new(explorer: &'a Explorer, path: String)
        -> Result<Article<'a>, ErrorBox> {
        let doc = explorer.get_html(path.parse()?).await?;
        Ok(Self {
            explorer,
            meta: parser::article(&doc, path)?,
            links: parser::image_list(&doc)?,
            comments: parser::comments(&doc)?,
        })
    }

    pub fn meta(&self) -> &ArticleMeta {
        &self.meta
    }

    // it's O(1) to random access
    pub fn comments(&self) -> slice::Iter<'_, Comment> {
        self.comments.iter()
    }

    pub async fn load_image_list(&mut self) -> Result<(), ErrorBox> {
        const IMAGES_PER_PAGE: usize = 40;
        let page_len = 1 + (self.meta.length - 1) / IMAGES_PER_PAGE;

        // start from 1 because we've already parsed page 0
        for i in 1..page_len {
            let doc = self.explorer.get_html(
                format!("{}?p={}", self.meta.path, i).parse()?
            ).await?;

            self.links.extend(parser::image_list(&doc)?);
        }

        Ok(())
    }

    pub async fn load_image(&self, index: usize) -> Result<Vec<u8>, ErrorBox> {
        // is this really the best?
        if index >= self.links.len() {
            panic!(":P"); // TODO
        }

        let doc = self.explorer.get_html(self.links[index].parse()?).await?;
        let path = parser::image(&doc)?;

        let data = self.explorer.get_bytes(path.parse()?).await?;
        Ok(data)
    }

    pub async fn load_all_comments(&mut self) -> Result<(), ErrorBox> {
        let path = format!("{}?hc=1", self.meta.path).parse()?;
        let doc = self.explorer.get_html(path).await?;
        self.comments = parser::comments(&doc)?;

        Ok(())
    }
}
