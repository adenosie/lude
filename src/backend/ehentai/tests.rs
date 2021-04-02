use super::*;

#[tokio::test]
async fn search() {
    use tokio_stream::StreamExt;

    let explorer = Explorer::new().await.unwrap();
    let mut page = explorer.search("language:korean").take(3);

    while let Some(list) = page.try_next().await.unwrap() {
        for draft in list.into_iter().take(3) {
            let article = draft.load().await.unwrap();
            println!("{:#?}", article.meta());
            println!("{:#?}", article.comments());
        }
    }
}

#[tokio::test]
async fn download() {
    use tokio_stream::StreamExt;
    use std::fs;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;

    let explorer = Explorer::new().await.unwrap();
    let mut page = explorer.search("language:korean female:yuri females only").take(1);

    // get the latest article
    let list = page.try_next().await.unwrap().unwrap();
    let draft = list.into_iter().nth(0).unwrap();
    let mut article = draft.load().await.unwrap();

    let mut path = PathBuf::from("./tests/download/");

    // clean the directory
    for entry in fs::read_dir(path.as_path()).unwrap() {
        let entry = entry.unwrap();
        fs::remove_file(entry.path()).unwrap();
    }

    // download shits
    path.push("0.jpg");
    for i in 0..article.meta().length {
        let image = article.load_image(i).await.unwrap();

        path.set_file_name(&format!("{}.jpg", i));
        let mut file = File::create(path.as_path()).unwrap();

        file.write_all(&image).unwrap();
    }
}
