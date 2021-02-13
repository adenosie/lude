use std::fmt;
use std::str::FromStr;
use std::error::Error;
use std::ops::{Index, IndexMut};

#[derive(Debug)]
pub struct EhParseTagError();

impl fmt::Display for EhParseTagError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Tag with wrong format was given")
    }
}

impl Error for EhParseTagError {

}

#[derive(Clone, Copy)]
pub enum EhTagKind {
    Language,
    Group,
    Parody,
    Character,
    Artist,
    Male,
    Female,
    Misc
}

impl FromStr for EhTagKind {
    type Err = EhParseTagError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            // alternates according to [https://ehwiki.org/wiki/Namespace]
            "language" | "lang" => Ok(EhTagKind::Language),
            "group" | "creator" | "circle" | "g" => Ok(EhTagKind::Group),
            "parody" | "series" | "p" => Ok(EhTagKind::Parody),
            "character" | "char" | "c" => Ok(EhTagKind::Character),
            "artist" | "a" => Ok(EhTagKind::Artist),
            "male" | "m" => Ok(EhTagKind::Male),
            "female" | "f" => Ok(EhTagKind::Female),
            "misc" => Ok(EhTagKind::Misc),
            _ => Err(EhParseTagError())
        }
    }
}

impl fmt::Display for EhTagKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            // give shorter names
            match *self {
                EhTagKind::Language => write!(f, "lang"),
                EhTagKind::Group => write!(f, "g"),
                EhTagKind::Parody => write!(f, "p"),
                EhTagKind::Character => write!(f, "c"),
                EhTagKind::Artist => write!(f, "a"),
                EhTagKind::Male => write!(f, "m"),
                EhTagKind::Female => write!(f, "f"),
                EhTagKind::Misc => write!(f, "misc")
            }
        } else {
            match *self {
                EhTagKind::Language => write!(f, "language"),
                EhTagKind::Group => write!(f, "group"),
                EhTagKind::Parody => write!(f, "parody"),
                EhTagKind::Character => write!(f, "character"),
                EhTagKind::Artist => write!(f, "artist"),
                EhTagKind::Male => write!(f, "male"),
                EhTagKind::Female => write!(f, "female"),
                EhTagKind::Misc => write!(f, "misc")
            }
        }
    }
}

pub struct EhTag(EhTagKind, String);

impl FromStr for EhTag {
    type Err = EhParseTagError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let colon = s.as_bytes().iter().position(|&x| x == b':');

        match colon {
            Some(pos) => {
                let category = s[..pos].parse()?;
                let tag = String::from(&s[(pos + 1)..]);

                Ok(EhTag(category, tag))
            },
            None => Err(EhParseTagError())
        }
    }
}

impl fmt::Display for EhTag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.0, self.1)
    }
}

pub struct EhTagMap {
    // all is (probably) sorted alphabetically
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

impl EhTagMap {
    pub fn add(&mut self, tag: EhTag) {
        self[tag.0].push(tag.1);
    }

    pub fn has(&mut self, tag: &EhTag) -> bool {
        self[tag.0].iter().any(|x| x == &tag.1)
    }
}

impl Index<EhTagKind> for EhTagMap {
    type Output = Vec<String>;

    fn index(&self, category: EhTagKind) -> &Self::Output {
        match category {
            EhTagKind::Language => &self.language,
            EhTagKind::Group => &self.group,
            EhTagKind::Parody => &self.parody,
            EhTagKind::Character => &self.character,
            EhTagKind::Artist => &self.artist,
            EhTagKind::Male => &self.male,
            EhTagKind::Female => &self.female,
            EhTagKind::Misc => &self.misc,
        }
    }
}

impl IndexMut<EhTagKind> for EhTagMap {
    fn index_mut(&mut self, category: EhTagKind) -> &mut Self::Output {
        match category {
            EhTagKind::Language => &mut self.language,
            EhTagKind::Group => &mut self.group,
            EhTagKind::Parody => &mut self.parody,
            EhTagKind::Character => &mut self.character,
            EhTagKind::Artist => &mut self.artist,
            EhTagKind::Male => &mut self.male,
            EhTagKind::Female => &mut self.female,
            EhTagKind::Misc => &mut self.misc,
        }
    }
}
