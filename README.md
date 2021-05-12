# lude

This project is in very early stage; More awesome features will come soon. Keep watching me!

## Todo List

- Provide decent networking
    - [x] Bypass DPI censorships
    - [x] Asynchronous, concurrent, parallel or whatever
- Downloading hentais from:
    - [x] E-hentai
    - [ ] Exhentai
    - [ ] Hitomi
    - And more...

## Examples

from [`here`](src/ehentai/tests.rs):

```rust
use tokio::time::{sleep, Duration};
use tokio::sync::mpsc;
use std::sync::Arc;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use lude::ehentai::{Explorer, Article};

async fn parallel() {
    // load article infos 
    let url = String::from("https://e-hentai.org/g/1335995/ba04527f3d/");
    let explorer = Explorer::new();

    let mut article = explorer.article_from_path(url).await.unwrap();
    article.load_image_list().await.unwrap();
    let len = article.meta().length;

    // load images parallelly
    let (tx, mut rx) = mpsc::channel(len);

    let article = Arc::new(article);

    for i in 0..len {
        sleep(Duration::from_millis(100)).await;

        let tx = tx.clone();
        let article = Arc::clone(&article);

        tokio::spawn(async move {
            println!("downloading {}th image..", i);
            let image = article.load_image(i).await.unwrap();
            println!("saved {}th image!", i);
            tx.send((i, image)).await.unwrap();
        });
    }

    let mut path = PathBuf::from("./tests/parallel/");

    // clean the directory
    for entry in fs::read_dir(path.as_path()).unwrap() {
        let entry = entry.unwrap();
        fs::remove_file(entry.path()).unwrap();
    }

    println!("cleaned directory, waiting for images...");

    // save shits
    path.push("0.jpg");

    for _ in 0..len {
        let (i, image) = rx.recv().await.unwrap();
        println!("received {}th image", i);

        path.set_file_name(&format!("{}.jpg", i));
        let mut file = File::create(path.as_path()).unwrap();
        file.write_all(&image).unwrap();
    }
}

```

## License

This project is under MPL 2.0. Unless you modify the code of this library, You can freely link this both statically and dynamically to your code, with no worry of your code being infected. See [LICENSE](LICENSE) for full license.
