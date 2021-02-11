use std::fmt;
use std::str::FromStr;
use std::error::Error;
use super::tag::EhTagMap;

pub enum EhArticleKind {
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
pub struct EhParseArticleError();

impl fmt::Display for EhParseArticleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to format article type")
    }
}

impl Error for EhParseArticleError {

}

impl FromStr for EhArticleKind {
    type Err = EhParseArticleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Doujinshi" => Ok(EhArticleKind::Doujinshi),
            "Manga" => Ok(EhArticleKind::Manga),
            "Artist CG" => Ok(EhArticleKind::ArtistCG),
            "Game CG" => Ok(EhArticleKind::GameCG),
            "Western" => Ok(EhArticleKind::Western),
            "Non-H" => Ok(EhArticleKind::NonH),
            "Image Set" => Ok(EhArticleKind::ImageSet),
            "Cosplay" => Ok(EhArticleKind::Cosplay),
            "Asian Porn" => Ok(EhArticleKind::AsianPorn),
            "Misc" => Ok(EhArticleKind::Misc),
            "Private" => Ok(EhArticleKind::Private),
            _ => Err(EhParseArticleError())
        }
    }
}

pub struct EhPendingArticle {
    pub(crate) kind: EhArticleKind,
    pub(crate) thumb: String,
    pub(crate) date: String,
    pub(crate) path: String,
    pub(crate) title: String,
    pub(crate) tags: Vec<String>,
    pub(crate) uploader: String,
    pub(crate) pages: usize,
}

pub struct EhArticle {
    path: String,

    kind: EhArticleKind,
    title: String,
    original_title: String,
    uploader: String,
    tags: EhTagMap,

    // ... TODO
}
