use std::sync::Arc;
use hyper::Uri;

use super::client::Client;
use super::tag::{ArticleKind, TagMap};
use super::parser;

type ErrorBox = Box<dyn std::error::Error>;

pub struct Draft {
    client: Arc<Client>,
    meta: ArticleMeta,
}

impl Draft {
    pub(super) async fn new(client: Arc<Client>, id: u32)
        -> Result<Self, ErrorBox> {
        let dest = Uri::builder()
            .scheme("https")
            .authority("ltn.hitomi.la")
            .path_and_query(format!("/galleryblock/{}.html", id))
            .build()?;

        let doc = client.get_html(dest).await?;

        Ok(Self {
            client,
            meta: parser::draft(&doc, id)?,
        })
    }

    pub async fn load(self) -> Result<Article, ErrorBox> {
        Article::new(self.client, self.meta.id).await
    }
}

pub struct ArticleMeta {
    pub id: u32,
    pub path: String,
    pub thumb: String,
    pub thumb_avif: String,
    pub preview: String,
    pub preview_avif: String,
    pub title: String,
    pub english_title: String,
    pub artists: Vec<String>,
    pub series: Vec<String>,
    pub kind: ArticleKind,
    pub language: String,
    pub tags: TagMap,
    pub date: String,
}

pub struct Article {
    client: Arc<Client>,
    meta: ArticleMeta,
    images: Vec<String>,
}

impl Article {
    pub(super) async fn new(client: Arc<Client>, id: u32)
        -> Result<Article, ErrorBox> {
        todo!()
    }
}
