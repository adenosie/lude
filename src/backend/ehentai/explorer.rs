use std::fmt;
use std::ops::{Index, IndexMut};
use std::str::FromStr;
use std::error::Error;
use core::future::Future;
use hyper::{Request, Response};
use crate::client::Client;

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
        -> impl Future<Output = Result<String, Box<dyn Error>>> + '_ {
        let query = percent_encode(query);

        async move {
            let vec = self.client
                .get_bytes(&format!("/?f_search={}", query)).await?;

            Ok(String::from_utf8(vec)?)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn search() {
        let mut explorer = EhExplorer::new().await.unwrap();

        let html = explorer.search("language:korean").await.unwrap();
        println!("{}", html.len());
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
