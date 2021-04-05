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

#[tokio::test]
async fn sequential() {
    use std::fs;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;

    let url = String::from("https://e-hentai.org/g/1335995/ba04527f3d/");

    let explorer = Explorer::new().await.unwrap();
    let mut article = explorer.article_from_path(url).await.unwrap();
    article.load_image_list().await.unwrap();

    let mut path = PathBuf::from("./tests/sequential/");

    // clean the directory
    for entry in fs::read_dir(path.as_path()).unwrap() {
        let entry = entry.unwrap();
        fs::remove_file(entry.path()).unwrap();
    }

    // download shits
    path.push("0.jpg");

    let len = article.meta().length;
    for i in 0..len {
        let image = article.load_image(i).await.unwrap();

        path.set_file_name(&format!("{}.jpg", i));
        let mut file = File::create(path.as_path()).unwrap();

        file.write_all(&image).unwrap();
    }
}

// FIXME
#[tokio::test]
async fn parallel() {
    use tokio::time::{sleep, Duration};
    use tokio::sync::mpsc;
    use std::sync::Arc;
    use std::fs;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;

    // load article infos 
    let url = String::from("https://e-hentai.org/g/1335995/ba04527f3d/");

    let explorer = Explorer::new().await.unwrap();

    let mut article = explorer.article_from_path(url).await.unwrap();
    article.load_image_list().await.unwrap();
    let len = article.meta().length;

    // load images parallelly
    let (tx, mut rx) = mpsc::channel(len);

    let article = Arc::new(article);

    for i in 0..len {
        // sleep a bit to prevent ban
        sleep(Duration::from_millis(500)).await;

        let tx = tx.clone();
        let article = Arc::clone(&article);

        tokio::spawn(async move {
            println!("downloading {}th image..", i);
            let image = article.load_image(i).await.unwrap();
            tx.send((i, image)).await.unwrap();
            println!("saved {}th image!", i);
        });
    }

    let mut path = PathBuf::from("./tests/parallel/");

    // clean the directory
    for entry in fs::read_dir(path.as_path()).unwrap() {
        let entry = entry.unwrap();
        fs::remove_file(entry.path()).unwrap();
    }

    // save shits
    path.push("0.jpg");
    
    while let Some((i, image)) = rx.recv().await {
        path.set_file_name(&format!("{}.jpg", i));
        let mut file = File::create(path.as_path()).unwrap();
        file.write_all(&image).unwrap();
    }
}
