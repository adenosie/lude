/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt;
use std::str::FromStr;
use std::iter::{FromIterator, IntoIterator};
use std::error::Error;
use std::ops::{Index, IndexMut};

#[derive(Debug)]
pub struct ParseTagError();

impl fmt::Display for ParseTagError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Tag with wrong format was given")
    }
}

impl Error for ParseTagError {

}

#[derive(Clone, Copy)]
pub enum TagKind {
    Reclass,
    Language,
    Group,
    Parody,
    Character,
    Artist,
    Male,
    Female,
    Misc,
}

impl FromStr for TagKind {
    type Err = ParseTagError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            // alternates according to [https://ehwiki.org/wiki/Namespace]
            "reclass" | "r" => Ok(TagKind::Reclass),
            "language" | "lang" => Ok(TagKind::Language),
            "group" | "creator" | "circle" | "g" => Ok(TagKind::Group),
            "parody" | "series" | "p" => Ok(TagKind::Parody),
            "character" | "char" | "c" => Ok(TagKind::Character),
            "artist" | "a" => Ok(TagKind::Artist),
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
                TagKind::Reclass => write!(f, "r"),
                TagKind::Language => write!(f, "lang"),
                TagKind::Group => write!(f, "g"),
                TagKind::Parody => write!(f, "p"),
                TagKind::Character => write!(f, "c"),
                TagKind::Artist => write!(f, "a"),
                TagKind::Male => write!(f, "m"),
                TagKind::Female => write!(f, "f"),
                TagKind::Misc => write!(f, "misc")
            }
        } else {
            match *self {
                TagKind::Reclass => write!(f, "reclass"),
                TagKind::Language => write!(f, "language"),
                TagKind::Group => write!(f, "group"),
                TagKind::Parody => write!(f, "parody"),
                TagKind::Character => write!(f, "character"),
                TagKind::Artist => write!(f, "artist"),
                TagKind::Male => write!(f, "male"),
                TagKind::Female => write!(f, "female"),
                TagKind::Misc => write!(f, "misc")
            }
        }
    }
}

pub struct Tag(TagKind, String);

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
            None => Err(ParseTagError())
        }
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.0, self.1)
    }
}

#[derive(Default)]
pub struct TagMap {
    // all is (probably) sorted alphabetically
    // (because the webpage gives tags so)
    language: Vec<String>,
    group: Vec<String>,
    parody: Vec<String>,
    character: Vec<String>,
    artist: Vec<String>,
    male: Vec<String>,
    female: Vec<String>,
    reclass: Vec<String>,
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
            TagKind::Reclass => &self.reclass,
            TagKind::Language => &self.language,
            TagKind::Group => &self.group,
            TagKind::Parody => &self.parody,
            TagKind::Character => &self.character,
            TagKind::Artist => &self.artist,
            TagKind::Male => &self.male,
            TagKind::Female => &self.female,
            TagKind::Misc => &self.misc,
        }
    }
}

impl IndexMut<TagKind> for TagMap {
    fn index_mut(&mut self, category: TagKind) -> &mut Self::Output {
        match category {
            TagKind::Reclass => &mut self.reclass,
            TagKind::Language => &mut self.language,
            TagKind::Group => &mut self.group,
            TagKind::Parody => &mut self.parody,
            TagKind::Character => &mut self.character,
            TagKind::Artist => &mut self.artist,
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
