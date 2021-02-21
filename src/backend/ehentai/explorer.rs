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

    pub fn save_images(&mut self, article: EhArticle)
        -> impl Future<Output = Result<Vec<Vec<u8>>, Box<dyn Error>>> + '_ {
        async move {
            let mut res = Vec::new();

            for path in &article.images {
                let doc = self.client.query_html(path).await?;
                let path = parser::parse_image(&doc)?;

                // FIXME: full images are outside the e-hentai.org domain,
                // which is something like https://*.*.hath.network/h/*/*.jpg.
                // we can only connect to one domain(e-hentai.org) currently,
                // so we can't get the images right now.

                // let resp = self.client.get(&path).await?;
                return unimplemented!();
            }

            Ok(res)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn search() {
        let mut explorer = EhExplorer::new().await.unwrap();

        let mut list = explorer.search("language:korean").await.unwrap();
        let article = explorer.article(list.pop().unwrap()).await.unwrap();

        /*
        let images = explorer.save_images(article).await.unwrap();

        use std::fs::File;
        use std::io::prelude::*;

        for (i, image) in images.iter().enumerate() {
            let mut file = File::create(format!("test/{}.jpg", i)).unwrap();
            file.write_all(&image).unwrap();
        }
        */
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
