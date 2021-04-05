use super::*;

#[tokio::test]
async fn search() {
    let explorer = Explorer::new().await.unwrap();
    let mut page = explorer.search("language:korean").take(3);

    while let Some(list) = page.next().await.unwrap() {
        for draft in list.into_iter().take(3) {
            let article = draft.load().await.unwrap();
            println!("{:#?}", article.meta());
            println!("{:#?}", article.comments());
        }
    }
}

#[tokio::test]
async fn light() {
    use std::fs::File;
    use std::io::Write;

    let url = String::from("https://e-hentai.org/g/1556174/cfe385099d/");
    let explorer = Explorer::new().await.unwrap();
    let article = explorer.article_from_path(url).await.unwrap();
    
    let first = article.load_image(0).await.unwrap();
    let mut file = File::create("./tests/light.jpg").unwrap();
    file.write_all(&first).unwrap();
}

// FIXME
#[tokio::test]
async fn heavy() {
    use tokio::time::{sleep, Duration};
    use tokio::sync::Mutex;
    use std::sync::Arc;
    use std::fs;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;

    // load article infos 
    let url = String::from("https://e-hentai.org/g/1556174/cfe385099d/");

    let explorer = Explorer::new().await.unwrap();

    let mut article = explorer.article_from_path(url).await.unwrap();
    article.load_image_list().await.unwrap();
    let len = article.meta().length;

    // load images parallelly
    let mut v = Vec::new();
    v.resize_with(len, || Mutex::new(Vec::new()));
    let v = Arc::new(v);

    let article = Arc::new(article);

    let mut joiner = Vec::new();
    for i in 0..len {
        // sleep a bit to prevent ban
        sleep(Duration::from_millis(300)).await;

        let article = Arc::clone(&article);
        let v = Arc::clone(&v);

        let handle = tokio::spawn(async move {
            println!("downloading {}th image..", i);
            let image = article.load_image(i).await.unwrap();
            println!("got {}th image", i);
            let mut v = v[i].lock().await;
            v.extend(image);
            println!("saved {}th image!", i);
        });

        joiner.push(handle);
    }

    for handle in joiner {
        handle.await.unwrap();
    }

    let v = Arc::try_unwrap(v).unwrap();
    let images: Vec<_> = v
        .into_iter()
        .map(Mutex::into_inner)
        .collect();

    let mut path = PathBuf::from("./tests/heavy/");

    // clean the directory
    for entry in fs::read_dir(path.as_path()).unwrap() {
        let entry = entry.unwrap();
        fs::remove_file(entry.path()).unwrap();
    }

    // save shits
    path.push("0.jpg");
    
    for i in 0..article.meta().length {
        path.set_file_name(&format!("{}.jpg", i));
        let mut file = File::create(path.as_path()).unwrap();
        file.write_all(&images[i]).unwrap();
    }
}
