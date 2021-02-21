use std::error::Error;
use select::document::Document;
use select::predicate::{Predicate, Attr, Class, Name};
use super::article::{EhArticleKind, EhPendingArticle, EhArticle};
use super::tag::{EhParseTagError, EhTagKind, EhTag, EhTagMap};

// take a document for a list page (e.g. search result),
// return the list of the articles in the document
pub fn parse_list(doc: &Document)
    -> Result<Vec<EhPendingArticle>, Box<dyn Error>> {
    let table = doc
        .find(Name("table").and(Class("gltc")))
        .nth(0).unwrap()
        .first_child().unwrap();

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
            text.parse::<EhArticleKind>()?
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
                .trim_start_matches("https://e-hentai.org")
                .to_string();

            let mut iter = node.children();
            let title = iter.next().unwrap().text();
            
            // although only some of the tags are visible in a browser,
            // there are all the tags in html; the rest are just hidden
            let tags = iter
                .next().unwrap()
                .children()
                .map(|x| x.attr("title").ok_or(EhParseTagError())?.parse::<EhTag>())
                .collect::<Result<EhTagMap, _>>()?;

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

        list.push(EhPendingArticle {
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

    Ok(list)
}

// take a document of an article gallery, return information of the article
//
// NOTE: this function DOES NOT parse the image list. call parse_image_list() 
// and change the article data accordingly to get the list of images.
pub fn parse_article_info(doc: &Document)
    -> Result<EhArticle, Box<dyn Error>> {
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
        .parse::<EhArticleKind>()?;

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
                .trim_start_matches("https://e-hentai.org")
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
        .split_ascii_whitespace()
        .nth(0).unwrap()
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
            more => more // n times
                .split_ascii_whitespace()
                .nth(0).unwrap()
                .parse::<usize>().unwrap()
        }
    };

    let rating_count = doc
        .find(Attr("id", "rating_count"))
        .nth(0).unwrap()
        .first_child().unwrap()
        .as_text().unwrap()
        .parse::<usize>().unwrap();

    let rating = doc
        .find(Attr("id", "rating_label"))
        .nth(0).unwrap()
        .first_child().unwrap()
        .as_text().unwrap()
        .split_ascii_whitespace()
        .nth(1).unwrap()
        .parse::<f64>().unwrap();

    let tags = {
        let list = doc
            .find(Attr("id", "taglist"))
            .nth(0).unwrap()
            .first_child().unwrap()
            .first_child().unwrap();

        let mut tags = EhTagMap::new();

        for row in list.children() {
            // remove last colon and parse
            let cat = row
                .first_child().unwrap()
                .first_child().unwrap()
                .as_text().unwrap();
            let cat = cat[..(cat.len() - 1)].parse::<EhTagKind>()?;

            for elem in row.last_child().unwrap().children() {
                tags[cat].push(elem.text());
            }
        }

        tags
    };

    Ok(EhArticle {
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
    })
}

// take a document of an article gallery, return list of link to image of the page
//
// NOTE: this function can only get 40 images in maximum at a time. get document
// of another page and call this again to obtain all images.
pub fn parse_image_list(doc: &Document)
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
            .trim_start_matches("https://e-hentai.org")
            .to_string();

        images.push(path);
    }
    
    Ok(images)
}

pub fn parse_image(doc: &Document)
    -> Result<String, Box<dyn Error>> {
    Ok(
        doc
        .find(Attr("id", "img"))
        .nth(0).unwrap()
        .attr("src").unwrap()
        .to_string()
    )
}
