/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::error::Error;
use select::document::Document;
use select::node::Node;
use select::predicate::Name;
use super::article::{ArticleMeta};
use super::tag::{TagKind, Tag};

pub fn draft(doc: &Document, id: u32)
    -> Result<ArticleMeta, Box<dyn Error>> {
    let mut iter = doc
        .nth(0).unwrap()
        .children();

    let img = iter
        .next().unwrap()
        .first_child().unwrap();

    let (thumb, thumb_avif, preview, preview_avif) = {
        fn extract(node: &Node) -> (String, String) {
            let picture = node
                .first_child().unwrap();

            let srcset = picture
                .first_child().unwrap()
                .attr("srcset").unwrap();

            let pos = srcset.find(" 2x,").unwrap();
            let avif = srcset[..pos].to_owned();

            let srcset = picture
                .last_child().unwrap()
                .attr("srcset").unwrap();

            let pos = srcset.find(" 2x,").unwrap();
            let jpg = srcset[..pos].to_owned();

            (jpg, avif)
        }

        let mut iter = img.children();

        let (thumb, thumb_avif) = extract(&iter.next().unwrap());
        let (preview, preview_avif) = extract(&iter.next().unwrap());

        (thumb, thumb_avif, preview, preview_avif)
    };

    let heading = iter
        .next().unwrap()
        .first_child().unwrap();

    let (path, english_title, title) = {
        let path = heading
            .attr("href").unwrap()
            .to_owned();

        let english_title = heading
            .attr("title").unwrap()
            .to_owned();

        let title = heading
            .first_child().unwrap()
            .as_text().unwrap()
            .to_owned();

        (path, english_title, title)
    };

    let artist_list = iter
        .next().unwrap()
        .first_child().unwrap();

    let artists = artist_list.find(Name("li")).map(|node| {
        node.first_child().unwrap()
            .first_child().unwrap()
            .as_text().unwrap()
            .to_owned()
    }).collect();

    let content = iter.next().unwrap();
    
    let (series, kind, language, tags) = {
        let mut iter = content
            .first_child().unwrap()
            .children();

        let mut series = Vec::new();

        let tr = iter.next().unwrap();
        for node in tr.children().skip(1) {
            let text = node
                .first_child().unwrap()
                .as_text().unwrap();

            if text == "N/A" {
                break;
            } else {
                series.push(text.to_owned());
            }
        }

        let kind = iter
            .next().unwrap()
            .last_child().unwrap()
            .first_child().unwrap()
            .first_child().unwrap()
            .as_text().unwrap()
            .parse()?;

        let language = iter
            .next().unwrap()
            .last_child().unwrap()
            .first_child().unwrap()
            .first_child().unwrap()
            .as_text().unwrap()
            .to_owned();

        let ul = iter
            .next().unwrap()
            .last_child().unwrap()
            .first_child().unwrap();

        let tags = ul.children().map(|node| {
            let text = node
                .first_child().unwrap()
                .first_child().unwrap()
                .as_text().unwrap();

            if text.ends_with('♀') {
                Tag(TagKind::Female, text[..(text.len() - 4)].to_owned())
            } else if text.ends_with('♂') {
                Tag(TagKind::Male, text[..(text.len() - 4)].to_owned())
            } else {
                Tag(TagKind::Misc, text.to_owned())
            }
        }).collect();

        (series, kind, language, tags)
    };

    let date = content
        .last_child().unwrap()
        .first_child().unwrap()
        .as_text().unwrap()
        .to_owned();

    Ok(ArticleMeta {
        id,
        thumb,
        thumb_avif,
        preview,
        preview_avif,
        path,
        english_title,
        title,
        artists,
        series,
        kind,
        language,
        tags,
        date
    })
}
