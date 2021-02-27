/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt;
use std::str::FromStr;
use std::error::Error;
use super::tag::TagMap;

pub enum ArticleKind {
    Doujinshi,
    Manga,
    ArtistCG,
    GameCG,
    Western,
    NonH,
    ImageSet,
    Cosplay,
    AsianPorn,
    Misc,
    Private
}

#[derive(Debug)]
pub struct ParseArticleKindError();

impl fmt::Display for ParseArticleKindError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to format gallery type")
    }
}

impl Error for ParseArticleKindError {

}

impl FromStr for ArticleKind {
    type Err = ParseArticleKindError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Doujinshi" => Ok(ArticleKind::Doujinshi),
            "Manga" => Ok(ArticleKind::Manga),
            "Artist CG" => Ok(ArticleKind::ArtistCG),
            "Game CG" => Ok(ArticleKind::GameCG),
            "Western" => Ok(ArticleKind::Western),
            "Non-H" => Ok(ArticleKind::NonH),
            "Image Set" => Ok(ArticleKind::ImageSet),
            "Cosplay" => Ok(ArticleKind::Cosplay),
            "Asian Porn" => Ok(ArticleKind::AsianPorn),
            "Misc" => Ok(ArticleKind::Misc),
            "Private" => Ok(ArticleKind::Private),
            _ => Err(ParseArticleKindError())
        }
    }
}

impl fmt::Display for ArticleKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ArticleKind::Doujinshi => write!(f, "Doujinshi"),
            ArticleKind::Manga => write!(f, "Manga"),
            ArticleKind::ArtistCG => write!(f, "Artist CG"),
            ArticleKind::GameCG => write!(f, "Game CG"),
            ArticleKind::Western => write!(f, "Western"),
            ArticleKind::NonH => write!(f, "Non-H"),
            ArticleKind::ImageSet => write!(f, "Image Set"),
            ArticleKind::Cosplay => write!(f, "Cosplay"),
            ArticleKind::AsianPorn => write!(f, "Asian Porn"),
            ArticleKind::Misc => write!(f, "Misc"),
            ArticleKind::Private => write!(f, "Private"),
        }
    }
}

pub struct PendingArticle {
    pub(crate) kind: ArticleKind,
    pub(crate) thumb: String,
    pub(crate) posted: String,
    pub(crate) path: String,
    pub(crate) title: String,
    pub(crate) tags: TagMap,
    pub(crate) uploader: String,
    pub(crate) length: usize,
}

pub struct Score {
    pub score: i64,
    pub votes: Vec<(String, i64)>,
    pub omitted_voters: usize,
}

pub struct Comment {
    pub(crate) posted: String,
    pub(crate) edited: Option<String>,

    // None if uploader comment
    pub(crate) score: Option<Score>,

    pub(crate) writer: String,
    pub(crate) content: String,
}

pub struct Article {
    pub(crate) title: String,
    pub(crate) original_title: String,

    pub(crate) kind: ArticleKind,
    // pub(crate) thumb: String,
    pub(crate) uploader: String,
    pub(crate) posted: String,
    pub(crate) parent: String,
    pub(crate) visible: bool, // what is this for?
    pub(crate) language: String,
    pub(crate) translated: bool,
    pub(crate) file_size: String,
    pub(crate) length: usize,
    pub(crate) favorited: usize,
    pub(crate) rating_count: usize,
    pub(crate) rating: f64,

    pub(crate) tags: TagMap,

    pub(crate) images: Vec<String>,
    pub(crate) comments: Vec<Comment>
}
