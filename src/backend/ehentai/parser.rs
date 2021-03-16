/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::error::Error;
use std::num::ParseIntError;
use select::document::Document;
use select::node::Node;
use select::predicate::{Predicate, Attr, Class, Name};
use super::article::{ArticleKind, PendingArticle, Score, Comment, Article};
use super::tag::{ParseTagError, TagKind, Tag, TagMap};

// take a document for an article list,
// return total count of results of the list
pub fn search_results(doc: &Document) -> Result<usize, Box<dyn Error>> {
    Ok(doc.find(Class("ip"))
        .nth(0).unwrap()
        .first_child().unwrap()
        .as_text().unwrap() // this would be like "Showing 608,394 results"
        .strip_prefix("Showing ").unwrap()
        .strip_suffix(" results").unwrap()
        .replace(',', "")
        .parse::<usize>()?)
}

// take a document for a list page (e.g. search result),
// return the list of the articles in the document
pub fn article_list(doc: &Document)
    -> Result<Option<Vec<PendingArticle>>, Box<dyn Error>> {
    let table = doc
        .find(Name("table").and(Class("gltc")))
        .nth(0);

    // no hits found
    if table.is_none() {
        return Ok(None);
    }

    let table = table.unwrap().first_child().unwrap();

    // requested invalid page
    if table.children().nth(1).unwrap().first_child().unwrap().as_text().is_some() {
        return Ok(None);
    }

    // we can't '?' in a closure, so can't we map().collect()
    let mut list = Vec::new();

    // the first element is header row; skip
    for node in table.children().skip(1) {
        // advert!
        if node.first_child().unwrap().attr("class") == Some("itd") {
            continue;
        }

        let mut iter = node.children();

        // 4 columns of the row in total
        let first = iter.next().unwrap();
        let second = iter.next().unwrap();
        let third = iter.next().unwrap();
        let fourth = iter.next().unwrap();

        // the first column contains category of the article
        let kind = {
            let text = first.first_child().unwrap().text();
            text.parse::<ArticleKind>()?
        };

        // the second contains thumbnail, uploaded time,
        // rate, and download link (costing GP)
        let (thumb, posted) = {
            let mut iter = second.children().skip(1);

            let thumb = iter
                .next().unwrap()
                .find(Name("img"))
                .nth(0).unwrap()
                .attr("src").unwrap()
                .to_string();

            let posted = iter
                .next().unwrap()
                .first_child().unwrap()
                .text();

            // TODO: i don't know how to get rate...
            (thumb, posted)
        };

        // the third contains link, title, and tags
        let (path, title, tags) = {
            let node = third.first_child().unwrap();

            let path = node
                .attr("href").unwrap()
                .to_string();

            let mut iter = node.children();
            let title = iter.next().unwrap().text();
            
            // although only some of the tags are visible in a browser,
            // there are all the tags in html; the rest are just hidden
            let tags = iter
                .next().unwrap()
                .children()
                .map(|x| x.attr("title").ok_or(ParseTagError())?.parse::<Tag>())
                .collect::<Result<TagMap, _>>()?;

            (path, title, tags)
        };

        // the fourth contains uploader name and number of pages in the article
        let (uploader, length) = {
            let uploader = fourth
                .first_child().unwrap()
                .first_child().unwrap()
                .text();

            let length = fourth
                .last_child().unwrap()
                .text()
                .split_ascii_whitespace()
                .nth(0).unwrap()
                .parse::<usize>()?;

            (uploader, length)
        };

        list.push(PendingArticle {
            kind,
            thumb,
            posted,
            path,
            title,
            tags,
            uploader,
            length
        });
    }

    Ok(Some(list))
}

// take a document of an article gallery, return information of the article
//
// NOTE: this function DOES NOT parse the image list. call parse_image_list() 
// and change the article data accordingly to get the list of images.
pub fn article(doc: &Document)
    -> Result<Article, Box<dyn Error>> {
    let (title, original_title) = {
        let mut iter = doc.find(Attr("id", "gd2")).nth(0).unwrap().children();
        
        let title = iter.next().unwrap().text();
        let orig = iter.next().unwrap().text();

        (title, orig)
    };

    // TODO: parse thumbnail (though it's identical with pending article's thumbnail)

    let kind = doc
        .find(Attr("id", "gdc"))
        .nth(0).unwrap()
        .first_child().unwrap()
        .first_child().unwrap() // this should be a text node
        .as_text().unwrap()
        .parse::<ArticleKind>()?;

    let uploader = doc
        .find(Attr("id", "gdn"))
        .nth(0).unwrap()
        .first_child().unwrap()
        .text();

    // parse #gdd, which has most useful informations
    let mut iter = doc
        .find(Attr("id", "gdd"))
        .nth(0).unwrap()
        .first_child().unwrap()
        .first_child().unwrap()
        .children();

    let posted = iter
        .next().unwrap()
        .last_child().unwrap()
        .text();

    let parent = {
        let node = iter
            .next().unwrap()
            .last_child().unwrap()
            .first_child().unwrap();

        if let Some("None") = node.as_text() {
            // no parent; node.text() would be "None"
            String::new()
        } else {
            // get a link to the parent
            node.attr("href").unwrap()
                .to_string()
        }
    };

    // what is this for?
    let visible = iter
        .next().unwrap()
        .last_child().unwrap()
        .first_child().unwrap()
        .as_text().unwrap() == "Yes";

    let (language, translated) = {
        let node = iter
            .next().unwrap()
            .last_child().unwrap();

        let language = node.first_child().unwrap().text();
        let translated = node.last_child().unwrap().name().is_some();

        (language, translated)
    };

    let file_size = iter
        .next().unwrap()
        .last_child().unwrap()
        .text();

    let length = iter
        .next().unwrap()
        .last_child().unwrap()
        .first_child().unwrap()
        .as_text().unwrap()
        .strip_suffix(" pages").unwrap() // it seems there is no article with 1 page
        .parse::<usize>()?;

    let favorited = {
        let text = iter
            .next().unwrap()
            .last_child().unwrap()
            .first_child().unwrap()
            .as_text().unwrap();

        match text {
            "Never" => 0,
            "Once" => 1,
            "Twice" => 2,
            more => more // the text would be like "(n) times"
                .strip_suffix(" times").unwrap()
                .parse::<usize>()?
        }
    };

    let rating_count = doc
        .find(Attr("id", "rating_count"))
        .nth(0).unwrap()
        .first_child().unwrap()
        .as_text().unwrap()
        .parse::<usize>()?;

    let rating = doc
        .find(Attr("id", "rating_label"))
        .nth(0).unwrap()
        .first_child().unwrap()
        .as_text().unwrap()
        .split_ascii_whitespace()
        .nth(1).unwrap()
        .parse::<f64>()?;

    let tags = {
        let list = doc
            .find(Attr("id", "taglist"))
            .nth(0).unwrap()
            .first_child().unwrap()
            .first_child().unwrap();

        let mut tags = TagMap::new();

        for row in list.children() {
            // remove last colon and parse
            let cat = row
                .first_child().unwrap()
                .first_child().unwrap()
                .as_text().unwrap();
            let cat = cat[..(cat.len() - 1)].parse::<TagKind>()?;

            for elem in row.last_child().unwrap().children() {
                tags[cat].push(elem.text());
            }
        }

        tags
    };

    // parse comments; .c1 is a class each comment node belongs to
    let comments = doc
        .find(Class("c1"))
        .map(|node| comment(&node))
        .collect::<Result<_, _>>()?;

    Ok(Article {
        title,
        original_title,
        kind,
        uploader,
        posted,
        parent,
        visible,
        language,
        translated,
        file_size,
        length,
        favorited,
        rating_count,
        rating,
        tags,
        images: Vec::new(),
        comments,
    })
}

pub fn comments(doc: &Document) -> Result<Vec<Comment>, Box<dyn Error>> {
    doc.find(Class("c1")).map(|node| comment(&node)).collect()
}

fn comment(node: &Node) -> Result<Comment, Box<dyn Error>> {
    let (top, bottom, votes, edited) = {
        let mut iter = node.children();

        let first = iter.next().unwrap();
        let second = iter.next().unwrap();
        let third = iter.next().unwrap();
        let fourth = iter.next();

        // if the comment is edited, "Last edited on (date)." 
        // message is shown before the votes list
        if let Some(fourth) = fourth {
            (first, second, fourth, Some(third))
        } else {
            (first, second, third, None)
        }
    };

    let (left, right) = {
        let mut iter = top.children();

        (iter.next().unwrap(), iter.next().unwrap())
    };

    // parse integer which is always prefixed with a '+' or '-' sign
    // necessary for parsing score of a comment, which is prefixed so
    fn parse_prefixed(text: &str) -> Result<i64, ParseIntError> {
        // rust's parse() can't comprehend prefix '+' sign
        let sign = match text.chars().nth(0) {
            Some('+') => 1,
            Some('-') => -1,
            _ => unreachable!() // logically impossible
        };
    
        text[1..].parse::<i64>().map(|x| x * sign)
    }
    
    let (posted, writer) = {
        let mut iter = left.children();

        let posted = iter
            .next().unwrap()
            .as_text().unwrap()
            .strip_prefix("Posted on ").unwrap()
            .strip_suffix(" by:   ").unwrap() // " by: &nbsp; "
            .to_owned();

        let writer = iter
            .next().unwrap()
            .first_child().unwrap()
            .text();

        (posted, writer)
    };

    let score = if right.is(Class("c4")) {
        None
    } else {
        let text = right
            .last_child().unwrap()
            .first_child().unwrap()
            .as_text().unwrap();

        let score = parse_prefixed(text)?;

        // parse a string formatted like "(writer) (score)"
        fn parse_vote(vote: &str) -> Result<(String, i64), ParseIntError> {
            // position of the last whitespace
            let pos = vote.rfind(' ').unwrap();

            Ok((vote[..pos].to_owned(), parse_prefixed(&vote[(pos + 1)..])?))
        }

        let omitted_voters = votes
            .last_child().unwrap()
            .as_text()
            .and_then(|text| text.strip_prefix(", and "))
            .and_then(|text| text.strip_suffix(" more..."))
            .map_or(Ok(0), |text| text.parse())?;

        let votes = {
            let mut list = Vec::new();

            let base = votes
                .first_child().unwrap()
                .as_text().unwrap();

            let base = base
                .strip_suffix(", ")
                .unwrap_or(base);

            list.push(parse_vote(base)?);

            for span in votes.find(Name("span")) {
                let vote = span
                    .first_child().unwrap()
                    .as_text().unwrap();

                list.push(parse_vote(vote)?);
            }

            list
        };

        Some(Score {
            score,
            votes,
            omitted_voters
        })
    };

    let edited = if let Some(node) = edited {
        Some(node
            .children()
            .nth(1).unwrap()
            .text())
    } else {
        None
    };

    let content = bottom
        .first_child().unwrap()
        .text();
    
    Ok(Comment {
        posted,
        edited,
        score,
        writer,
        content
    })
}

// take a document of an article gallery, return list of link to image of the page
//
// NOTE: this function can only get 40 images in maximum at a time. get document
// of another page and call this again to obtain all images.
pub fn image_list(doc: &Document)
    -> Result<Vec<String>, Box<dyn Error>> {
    let mut images = Vec::new();
    
    // is finding from id faster? i could just find by class...
    let list = doc.find(Attr("id", "gdt")).nth(0).unwrap();

    for node in list.children() {
        // advert!
        if node.attr("class") != Some("gdtm") {
            continue;
        }

        let path = node
            .first_child().unwrap()
            .first_child().unwrap()
            .attr("href").unwrap()
            .to_string();

        images.push(path);
    }
    
    Ok(images)
}

pub fn image(doc: &Document)
    -> Result<String, Box<dyn Error>> {
    Ok(
        doc
        .find(Attr("id", "img"))
        .nth(0).unwrap()
        .attr("src").unwrap()
        .to_string()
    )
}
