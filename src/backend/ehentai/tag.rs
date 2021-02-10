use std::fmt;
use std::str::FromStr;
use std::ops::{Index, IndexMut};

pub struct EhParseTagError();

impl fmt::Display for EhParseTagError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Tag with wrong format was given")
    }
}

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
            "language" => EhTagKind::Language,
            "group" => EhTagKind::Group,
            "parody" => EhTagKind::Parody,
            "character" => EhTagKind::Character,
            "artist" => EhTagKind::Artist,
            "male" => EhTagKind::Male,
            "female" => EhTagKind::Female,
            "misc" => EhTagKind::Misc,
        }
    }
}

impl fmt::Display for EhTagKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

impl FromStr for (EhTagKind, String) {
    type Err = EhParseTagError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let colon = s.as_bytes().iter().position(|&x| x == b':');

        match colon {
            Some(pos) => {
                let category = s[..pos].parse()?;
                let tag = String::from(&s[(pos + 1)..]);

                (category, tag)
            },
            None => Err(EhParseTagError())
        }
    }
}

impl fmt::Display for (EhTagKind, String) {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.0, self.1)
    }
}

pub struct EhTags {
    // all would be sorted alphabetically
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

impl Index<EhTagKind> for EhTags {
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

impl IndexMut<EhTagKind> for EhTags {
    type Output = Vec<String>;

    fn index(&mut self, category: EhTagKind) -> &mut Self::Output {
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
