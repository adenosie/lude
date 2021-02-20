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

impl fmt::Display for EhArticleKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EhArticleKind::Doujinshi => write!(f, "Doujinshi"),
            EhArticleKind::Manga => write!(f, "Manga"),
            EhArticleKind::ArtistCG => write!(f, "Artist CG"),
            EhArticleKind::GameCG => write!(f, "Game CG"),
            EhArticleKind::Western => write!(f, "Western"),
            EhArticleKind::NonH => write!(f, "Non-H"),
            EhArticleKind::ImageSet => write!(f, "Image Set"),
            EhArticleKind::Cosplay => write!(f, "Cosplay"),
            EhArticleKind::AsianPorn => write!(f, "Asian Porn"),
            EhArticleKind::Misc => write!(f, "Misc"),
            EhArticleKind::Private => write!(f, "Private"),
        }
    }
}

pub struct EhPendingArticle {
    pub(crate) kind: EhArticleKind,
    pub(crate) thumb: String,
    pub(crate) posted: String,
    pub(crate) path: String,
    pub(crate) title: String,
    pub(crate) tags: EhTagMap,
    pub(crate) uploader: String,
    pub(crate) length: usize,
}

pub struct EhArticle {
    pub(crate) title: String,
    pub(crate) original_title: String,

    pub(crate) kind: EhArticleKind,
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

    pub(crate) tags: EhTagMap,

    pub(crate) images: Vec<String>,
}
