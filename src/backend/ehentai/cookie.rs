use std::collections::HashMap;

pub struct Cookie {
    value: String,
    // expires: DateTime,
}

impl Cookie {
    fn new(value: String) -> Self {
        Self {
            value,
        }
    }
}

pub struct CookieStore {
    pairs: HashMap<String, Cookie>,
}

impl CookieStore {
    pub fn new() -> Self {
        Self {
            pairs: HashMap::new()
        }
    }

    pub fn set(&mut self, header: &str) {
        let mut line = header.split(';');

        // payload comes first
        let pair = line.next().unwrap_or(header);

        let delim = pair
            .find('=')
            .expect("Set-Cookie header doesn't have '='");

        let (key, val) = pair.split_at(delim);
        let (key, val) = (key.trim(), val.trim());

        // TODO
        // for av in line {
        //
        // }
        
        let cookie = Cookie::new(val.into());

        self.pairs.insert(key.into(), cookie);
    }

    pub fn bake(&self, _host: &str) -> String {
        let mut res = String::new();

        for (key, val) in self.pairs.iter() {
            res.push_str(key);
            res.push('=');
            res.push_str(&val.value);
            res.push_str("; ");
        }

        res
    }
}
