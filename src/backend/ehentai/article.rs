use super::tag::EhTags;

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

pub struct EhPendingArticle {
    // path to article after domain name
    // (e.g. "/g/1556174/cfe385099d/")
    path: String, 

    kind: EhArticleKind,
    title: String,
    uploader: String,
    page_paths: Vec<String>,
    brief_tags: EhTags
}

pub struct EhArticle {
    kind: EhArticleKind,
    title: String,
    original_title: String,
    uploader: String,
    tags: EhTags,

    // ... TODO
}
