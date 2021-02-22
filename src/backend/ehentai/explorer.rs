use std::error::Error;
use std::future::Future;

use hyper::{Client, Uri, Body};
use hyper::client::connect::HttpConnector;
use detour::HttpsConnector;
use select::document::Document;

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
    client: Client<HttpsConnector<HttpConnector>, Body>,
}

impl EhExplorer {
    pub fn new()
        -> impl Future<Output = Result<EhExplorer, Box<dyn Error>>> {
        async {
            let https = HttpsConnector::new();
            let client = Client::builder()
                .build::<_, Body>(https);

            Ok(Self {
                client,
            })
        }
    }

    fn query(&self, dest: Uri)
        -> impl Future<Output = Result<Vec<u8>, Box<dyn Error>>> + '_ {
        async move {
            let res = self.client.get(dest).await?;

            let bytes = hyper::body::to_bytes(res.into_body()).await?;
            Ok(bytes.to_vec())
        }
    }

    fn query_html(&self, dest: Uri)
        -> impl Future<Output = Result<Document, Box<dyn Error>>> + '_ {
        async move {
            let bytes = self.query(dest).await?;
            let file = String::from_utf8(bytes)?;
            Ok(Document::from(file.as_str()))
        }
    }

    pub fn search(&self, query: &str)
        -> impl Future<Output = Result<Vec<EhPendingArticle>, Box<dyn Error>>> + '_ {
        let query = percent_encode(query);

        async move {
            let doc = self.query_html(
                format!("https://e-hentai.org/?f_search={}", query).parse()?
            ).await?;
            parser::parse_list(&doc)
        }
    }

    pub fn article(&self, pending: EhPendingArticle)
        -> impl Future<Output = Result<EhArticle, Box<dyn Error>>> + '_ {
        // one page shows 40 images at max
        let page_len: usize = (pending.length - 1) / 40 + 1;

        async move {
            let doc = self.query_html(pending.path.parse()?).await?;
            let mut article = parser::parse_article_info(&doc)?;

            let mut vec = parser::parse_image_list(&doc)?;
            article.images.append(&mut vec);

            // TODO: this could be done async
            for i in 1..page_len {
                let doc = self.query_html(
                    format!("{}?p={}", pending.path, i).parse()?
                ).await?;

                let mut vec = parser::parse_image_list(&doc)?;
                article.images.append(&mut vec);
            }

            Ok(article)
        }
    }

    pub fn save_images(&self, article: EhArticle)
        -> impl Future<Output = Result<Vec<Vec<u8>>, Box<dyn Error>>> + '_ {
        async move {
            let mut res = Vec::new();

            for path in &article.images {
                let doc = self.query_html(path.parse()?).await?;
                let path = parser::parse_image(&doc)?;

                let image = self.query(path.parse()?).await?;
                res.push(image);
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

        let images = explorer.save_images(article).await.unwrap();

        use std::fs::File;
        use std::io::prelude::*;

        for (i, image) in images.iter().enumerate() {
            println!("saving {}th image", i + 1);
            let mut file = File::create(format!("test/{}.jpg", i + 1)).unwrap();
            file.write_all(&image).unwrap();
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
