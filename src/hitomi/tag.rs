/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt;
use std::str::FromStr;
use std::iter::{FromIterator, IntoIterator};
use std::error::Error;
use std::ops::{Index, IndexMut};

#[derive(Debug, Copy, Clone)]
pub enum ArticleKind {
    Doujinshi,
    Manga,
    ArtistCG,
    GameCG,
}

#[derive(Debug)]
pub struct ParseArticleKindError();

impl fmt::Display for ParseArticleKindError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to format gallery type")
    }
}

impl Error for ParseArticleKindError {}

impl FromStr for ArticleKind {
    type Err = ParseArticleKindError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Doujinshi" => Ok(ArticleKind::Doujinshi),
            "Manga" => Ok(ArticleKind::Manga),
            "Artist CG" => Ok(ArticleKind::ArtistCG),
            "Game CG" => Ok(ArticleKind::GameCG),
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
        }
    }
}

#[derive(Debug)]
pub struct ParseTagError();

impl fmt::Display for ParseTagError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Tag with wrong format was given")
    }
}

impl Error for ParseTagError {}

#[derive(Clone, Copy)]
pub enum TagKind {
    Male,
    Female,
    Misc,
}

impl FromStr for TagKind {
    type Err = ParseTagError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "male" | "m" => Ok(TagKind::Male),
            "female" | "f" => Ok(TagKind::Female),
            "misc" | "" => Ok(TagKind::Misc),
            _ => Err(ParseTagError())
        }
    }
}

impl fmt::Display for TagKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            // give shorter names
            match *self {
                TagKind::Male => write!(f, "m"),
                TagKind::Female => write!(f, "f"),
                TagKind::Misc => Ok(()),
            }
        } else {
            match *self {
                TagKind::Male => write!(f, "male"),
                TagKind::Female => write!(f, "female"),
                TagKind::Misc => Ok(())
            }
        }
    }
}

pub struct Tag(pub TagKind, pub String);

impl FromStr for Tag {
    type Err = ParseTagError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let colon = s.as_bytes().iter().position(|&x| x == b':');

        match colon {
            Some(pos) => {
                let category = s[..pos].parse()?;
                let tag = String::from(&s[(pos + 1)..]);

                Ok(Tag(category, tag))
            },
            None => Ok(Tag(TagKind::Misc, s.to_owned()))
        }
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            TagKind::Misc => write!(f, "{}", self.1),
            _ => write!(f, "{}:{}", self.0, self.1)
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct TagMap {
    // all is (probably) sorted alphabetically
    // (just because the webpage gives tags so)
    male: Vec<String>,
    female: Vec<String>,
    misc: Vec<String>,
}

impl TagMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, tag: Tag) {
        self[tag.0].push(tag.1);
    }

    pub fn has(&mut self, tag: &Tag) -> bool {
        self[tag.0].iter().any(|x| x == &tag.1)
    }
}

impl Index<TagKind> for TagMap {
    type Output = Vec<String>;

    fn index(&self, category: TagKind) -> &Self::Output {
        match category {
            TagKind::Male => &self.male,
            TagKind::Female => &self.female,
            TagKind::Misc => &self.misc,
        }
    }
}

impl IndexMut<TagKind> for TagMap {
    fn index_mut(&mut self, category: TagKind) -> &mut Self::Output {
        match category {
            TagKind::Male => &mut self.male,
            TagKind::Female => &mut self.female,
            TagKind::Misc => &mut self.misc,
        }
    }
}

impl FromIterator<Tag> for TagMap {
    fn from_iter<I: IntoIterator<Item = Tag>>(iter: I) -> Self {
        let mut tags = TagMap::new();
        
        for tag in iter {
            tags.add(tag);
        }

        tags
    }
}
