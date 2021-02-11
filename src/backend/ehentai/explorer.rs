use std::ops::{Index, IndexMut};
use std::str::FromStr;
use std::error::Error;
use std::future::Future;

use hyper::{Body, Request, Response};
use select::document::Document;
use select::node::Node;
use select::predicate::{Predicate, And, Class, Name};

use crate::client::Client;
use super::article::{EhArticle, EhArticleKind, EhPendingArticle};

fn percent_encode(from: &str) -> String {
    let mut res = String::new();

    for byte in from.as_bytes() {
        match byte {
            // unreserved characters (MUST NOT be encoded)
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' 
                | b'-' | b'_' | b'.' | b'~' => {
                res.push(*byte as char);
            },
            _ => {
                res.push_str(&format!("%{:02X}", *byte));
            }
        }
    }

    res
}

pub struct EhExplorer {
    client: Client,
}

impl EhExplorer {
    pub fn new()
        -> impl Future<Output = Result<EhExplorer, Box<dyn Error>>> {
        async {
            let client = Client::new("e-hentai.org").await?;

            Ok(Self {
                client,
            })
        }
    }

    pub fn search(&mut self, query: &str)
        -> impl Future<Output = Result<Vec<EhPendingArticle>, Box<dyn Error>>> + '_ {
        let query = percent_encode(query);

        async move {
            let doc = self.query_html(&format!("/?f_search={}", query)).await?;
            let list = EhExplorer::parse_page(&doc);

            list
        }
    }

    fn query_html(&mut self, path: &str)
        -> impl Future<Output = Result<Document, Box<dyn Error>>> + '_ {
        let path = path.to_owned();

        async move {
            let req = Request::builder()
                .method("GET")
                .header("Host", "e-hentai.org")
                .uri(path)
                .header("Accept", "text/html")
                .body(Body::empty())?;

            let res = self.client.send_request(req).await?;
            let bytes = hyper::body::to_bytes(res.into_body()).await?;
            let file = String::from_utf8(bytes.to_vec())?;

            Ok(Document::from(file.as_str()))
        }
    }

    fn parse_page(doc: &Document)
        -> Result<Vec<EhPendingArticle>, Box<dyn Error>> {
        let table = doc
            .find(Name("table").and(Class("gltc")))
            .nth(0)
            .unwrap()
            .first_child()
            .unwrap();
        
        let mut list = Vec::new();

        // the first element is label
        for n in table.children().skip(1) {
            // advert!
            if n.first_child().unwrap().attr("class") == Some("itd") {
                continue;
            }

            list.push(EhExplorer::parse_node(&n)?);
        }

        Ok(list)
    }

    fn parse_node(node: &Node) -> Result<EhPendingArticle, Box<dyn Error>> {
        let mut iter = node.children();

        let first = iter.next().unwrap();
        let second = iter.next().unwrap();
        let third = iter.next().unwrap();
        let fourth = iter.next().unwrap();

        let kind = {
            let text = first.first_child().unwrap().text();
            text.parse::<EhArticleKind>()?
        };

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

            (thumb, date)
        };

        let (path, title, tags) = {
            let node = third.first_child().unwrap();

            let link = node.attr("href").unwrap().to_string();

            let mut iter = node.children();
            let title = iter.next().unwrap().text();
            
            let tags = iter
                .next()
                .unwrap()
                .children()
                .map(|x| x.attr("title").unwrap().to_string())
                .collect();

            (link, title, tags)
        };

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
                .text();

            let pages = pages
                .split_ascii_whitespace()
                .nth(0)
                .unwrap();

            let pages = pages.parse::<usize>()?;

            (uploader, pages)
        };

        Ok(EhPendingArticle {
            kind,
            thumb,
            date,
            path,
            title,
            tags,
            uploader,
            pages
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn search() {
        let mut explorer = EhExplorer::new().await.unwrap();

        let list = explorer.search("language:korean").await.unwrap();
        for pending in list {
            println!("{}", pending.title);
        }
    }

    /*
    async fn ideal() -> Result<(), Box<dyn Error>> {
        let explorer = EhExplorer::new().await?;

        for page in explorer.search("artist:hota.") {
            let page = page.await?;

            for pending in page.iter() {
                println!("{}", pending.title());
                let article = pending.load_into().await?;
                assert!(article.tags().has("artist:hota."));
            }
        }

        let article = explorer.from_path("/g/1556174/cfe385099d/").await?;
        assert_eq!(
            article.title(), 
            "(C97) [Bad Mushrooms (Chicke III, 4why)] \
            Nibun no Yuudou | 2등분의 유혹 \
            (Gotoubun no Hanayome) [Korean] [Team Edge]"
        );
    }
    */
}
