use std::fmt;
use std::error::Error;
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

pub struct ParseTagError();

impl fmt::Display for ParseTagError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Tag with wrong format was given")
    }
}

pub struct EhTags {
    // all are sorted alphabetically
    // (because the webpage gives tags so)
    language: Vec<String>,
    group: Vec<String>,
    parody: Vec<String>,
    character: Vec<String>,
    artist: Vec<String>,
    male: Vec<String>,
    female: Vec<String>,
    misc: Vec<String>,
}

impl EhTags {
    fn language(&self) -> &[String] {
        self.language
    }

    fn group(&self) -> &[String] {
        self.group
    }

    fn parody(&self) -> &[String] {
        self.parody
    }

    fn artist(&self) -> &[String] {
        self.artist
    }

    fn male(&self) -> &[String] {
        self.male
    }

    fn female(&self) -> &[String] {
        self.female
    }

    fn misc(&self) -> &[String] {
        self.misc
    }

    // process colon-separated strings such as "female:yuri"
    // underscores are replaced with space
    fn add(&mut self, tag: &str) -> Result<(), Error> {
        let colon = tag
            .iter()
            .enumerate()
            .position(|c| c == ':');

        let (first, second) = match colon {
            Some(pos) => {
                (tag[..pos], tag[(pos + 1)..])
            },
            None => {
                return Err(ParseTagError());
            }
        }

        match first {
            "language" => { self.language.push(String::from(second)); },
            "group"    => { self.language.push(String::from(second)); },
            "parody"   => { self.language.push(String::from(second)); },
            "artist"   => { self.language.push(String::from(second)); },
            "male"     => { self.language.push(String::from(second)); },
            "female"   => { self.language.push(String::from(second)); },
            "misc"     => { self.language.push(String::from(second)); },
            _ => { return Err(ParseTagError) }
        }

        Ok(())
    }
}

pub struct EhArticlePreloaded {
    // path to article AFTER domain name
    // (e.g. "/g/1556174/cfe385099d/")
    path: String, 

    kind: EhArticleKind,
    title: String,
    uploader: String,
    pages: usize,
    brief_tags: EhTags
}

pub struct EhArticle {
    kind: EhArticleKind,
    title: String,
    original_title: String,
    uploader: String,
    tags: Tags,

    // ... TODO
}
