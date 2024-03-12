use scraper::{Html, Selector};

use crate::model::db::Article;

pub fn replace_image(html: &str) -> String {
    let document = Html::parse_document(html);

    let img_selector = Selector::parse("img").unwrap();
    let img_elements = document.select(&img_selector);

    let mut imgs = vec![];

    for img_element in img_elements {
        let src_attr = img_element.value().attr("src");

        if let Some(src) = src_attr {
            imgs.push(src)
        }
    }

    let mut s = document.root_element().html();
    /*     for ele in imgs {
        s = s.replace(
            format!("\"{}\"", ele).as_str(),
            format!("\"http://chinslt.com:1105/api/images/{}\"", ele).as_str(),
        );
    } */

    s
}

pub fn replace_image_article(e: Article) -> Article {
    e
}
