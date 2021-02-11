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

pub struct EhArticle {
    path: String,

    kind: EhArticleKind,
    title: String,
    original_title: String,
    uploader: String,
    tags: EhTagMap,

    // ... TODO
}
