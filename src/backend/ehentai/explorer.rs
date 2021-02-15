use std::error::Error;
use std::future::Future;

use crate::client::Client;
use super::article::{EhArticle, EhArticleKind, EhPendingArticle};
use super::tag::{EhTagMap, EhTagKind};
use super::parser;

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
            let doc = self.client.query_html(&format!("/?f_search={}", query)).await?;
            parser::parse_list(&doc)
        }
    }

    pub fn article(&mut self, pending: EhPendingArticle)
        -> impl Future<Output = Result<EhArticle, Box<dyn Error>>> + '_ {
        // one page shows 40 images at max
        let page_len: usize = (pending.length - 1) / 40 + 1;

        async move {
            let doc = self.client.query_html(&pending.path).await?;
            let mut article = parser::parse_article_info(&doc)?;

            let mut vec = parser::parse_image_list(&doc)?;
            article.images.append(&mut vec);

            // TODO: this could be done async
            for i in 1..page_len {
                let doc = self.client.query_html(
                    &format!("{}?p={}", pending.path, i)
                ).await?;

                let mut vec = parser::parse_image_list(&doc)?;
                article.images.append(&mut vec);
            }

            Ok(article)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn search() {
        let mut explorer = EhExplorer::new().await.unwrap();

        let list = explorer.search("language:korean").await.unwrap();

        for pending in list.into_iter() {
            let article = explorer.article(pending).await.unwrap();

            println!("{}", article.original_title);
            println!("{}", article.kind);

            for tag in &article.tags[EhTagKind::Female] {
                print!("{} ", tag);
            }

            println!("");
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
