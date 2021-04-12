/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::str;
use std::fmt;
use std::sync::Arc;
use std::error::Error;

use hyper::{Uri, Body, Request, Response};
use hyper::client::connect::HttpConnector;
use detour::HttpsConnector;
use select::document::Document;

use super::cookie::CookieStore;
use super::article::Article;
use super::page::Page;

type ErrorBox = Box<dyn Error>;
type Client = hyper::Client<HttpsConnector<HttpConnector>, Body>;

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

pub struct Explorer {
    client: Client,
    cookies: CookieStore,
}

impl Explorer {
    pub fn owned() -> Explorer {
        let https = HttpsConnector::new();
        let client = hyper::Client::builder()
            .build::<_, Body>(https);

        let mut cookies = CookieStore::new();
        cookies.set("nw=1");

        Self {
            client,
            cookies,
        }
    }

    pub fn new() -> Arc<Explorer> {
        Arc::new(Explorer::owned())
    }

    pub async fn login(username: &str, password: &str)
        -> Result<Arc<Explorer>, ErrorBox> {
        unimplemented!()

        // let mut _self = Explorer::owned();

        // const ORIGIN: &str = "https://forums.e-hentai.org";
        // const REFERER: &str = "https://forums.e-hentai.org/index.php?act=Login&CODE=00";
        // const POST_DEST: &str = "https://forums.e-hentai.org/index.php?act=Login&CODE=01";

        // let form = format!(
        //     "CookieDate=1&UserName={}&PassWord={}&submit=Log%20me%20in",
        //     percent_encode(username),
        //     percent_encode(password)
        // );

        // let req = Request::post(POST_DEST)
        //     .header("Connection", "keep-alive")
        //     .header("Referer", REFERER)
        //     .header("Content-Type", "application/x-www-form-urlencoded")
        //     .header("Content-Length", form.len())
        //     .body(form.into())?;

        // let res = _self.client.request(req).await?;
        // res.headers()
        //     .get_all("Set-Cookie")
        //     .iter()
        //     .map(|val| val.to_str().unwrap())
        //     .for_each(|s| _self.cookies.set(s));

        // if _self.cookies.get("ipb_pass_hash").is_none() {
        //     panic!("login failed!");
        // }

        // Ok(Arc::new(_self))
    }

    pub async fn with_cookies(member_id: &str, pass_hash: &str)
        -> Arc<Explorer> {
        let mut _self = Explorer::owned();
        _self.cookies.set(&format!("ipb_member_id={}", member_id));
        _self.cookies.set(&format!("ipb_pass_hash={}", pass_hash));

        Ok(Arc::new(_self))
    }

    async fn get(&self, dest: Uri, mime: &str)
        -> Result<Response<Body>, ErrorBox> {
        let cookie = self.cookies
            .bake(dest.host().unwrap());

        let req = Request::get(dest)
            .header("Content-Type", mime)
            .header("Cookie", cookie)
            .body(Body::empty())?;

        let res = self.client.request(req).await?;
        Ok(res)
    }

    pub(super) async fn get_image(&self, dest: Uri)
        -> Result<Vec<u8>, ErrorBox> {
        let res = self.get(dest, "image/*").await?;
        let bytes = hyper::body::to_bytes(res.into_body()).await?;
    
        Ok(bytes.to_vec())
    }

    pub(super) async fn get_html(&self, dest: Uri)
        -> Result<Document, ErrorBox> {
        let res = self.get(dest, "text/html").await?;
        let bytes = hyper::body::to_bytes(res.into_body()).await?;
        let file = str::from_utf8(&bytes)?;
    
        Ok(Document::from(file))
    }

    pub fn search(self: &Arc<Self>, keyword: &str) -> Page {
        Page::new(self.clone(), 0, &percent_encode(keyword))
    }

    pub async fn article_from_path(self: &Arc<Self>, path: String)
        -> Result<Article, ErrorBox> {
        Article::new(self.clone(), path).await
    }
}
