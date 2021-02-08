use super;
use crate::client::Client;

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
    Misc
}

pub struct EhArticlePreloaded {
    kind: EhArticleKind,
    title: String,
    uploader: String,
    pages: usize,
}

pub struct EhArticle {
    kind: EhArticleKind,
    title: String,
    original_title: String,
    uploader: String,
    // ... TODO
}

pub struct EhExplorer {
    client: Client,
}

impl EhExplorer {
    fn search(query: &str) -> EhSearcher {

    }
}
