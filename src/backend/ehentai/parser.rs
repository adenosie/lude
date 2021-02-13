use std::error::Error;
use select::document::Document;
use select::predicate::{Predicate, Attr, Class, Name};
use super::article::{EhArticleKind, EhPendingArticle, EhArticle};
use super::tag::{EhTagKind, EhTag, EhTagMap};

// take a document for a list page (e.g. search result),
// return the list of the articles in the document
pub fn parse_list(doc: &Document)
    -> Result<Vec<EhPendingArticle>, Box<dyn Error>> {
    let table = doc
        .find(Name("table").and(Class("gltc")))
        .nth(0)
        .unwrap()
        .first_child()
        .unwrap();

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
        let (thumb, date) = {
            let mut iter = second.children().skip(1);

            let thumb = iter
                .next()
                .unwrap()
                .find(Name("img"))
                .nth(0)
                .unwrap()
                .attr("src")
                .unwrap()
                .to_string();

            let date = iter
                .next()
                .unwrap()
                .first_child()
                .unwrap()
                .text();

            // TODO: i don't know how to get rate...
            (thumb, date)
        };

        // the third contains link, title, and tags
        let (path, title, tags) = {
            let node = third.first_child().unwrap();

            let path = node
                .attr("href")
                .unwrap()
                .trim_start_matches("https://e-hentai.org")
                .to_string();

            let mut iter = node.children();
            let title = iter.next().unwrap().text();
            
            // although only some of the tags are visible in a browser,
            // there are all the tags in html; the rest are just hidden
            let tags = iter
                .next()
                .unwrap()
                .children()
                .map(|x| x.attr("title").unwrap().to_string())
                .collect();

            (path, title, tags)
        };

        // the fourth contains uploader name and number of pages in the article
        let (uploader, pages) = {
            let uploader = fourth
                .first_child()
                .unwrap()
                .first_child()
                .unwrap()
                .text();

            let pages = fourth
                .last_child()
                .unwrap()
                .text()
                .split_ascii_whitespace()
                .nth(0)
                .unwrap()
                .parse::<usize>()?;

            (uploader, pages)
        };

        list.push(EhPendingArticle {
            kind,
            thumb,
            date,
            path,
            title,
            tags,
            uploader,
            pages
        });
    }

    Ok(list)
}

// take a document of an article gallery, return list of link to image of the page
//
// NOTE: this method can only get 40 images at max. get document of another
// page and call this again to obtain all images.
pub fn parse_image_list(doc: &Document)
    -> Result<Vec<String>, Box<dyn Error>> {
    let mut images = Vec::new();
    let list = doc.find(Attr("id", "gdt")).nth(0).unwrap();

    for node in list.children() {
        // advert!
        if node.attr("class") != Some("gdtm") {
            continue;
        }

        let path = node
            .first_child()
            .unwrap()
            .first_child()
            .unwrap()
            .attr("href")
            .unwrap()
            .trim_start_matches("https://e-hentai.org")
            .to_string();

        images.push(path);
    }
    
    Ok(images)
}
