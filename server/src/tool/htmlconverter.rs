use scraper::{Html, Selector};
use tracing::info;

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

pub fn get_text_from_xml(s: &str) -> String {
    let frag = scraper::Html::parse_fragment(s);
    let mut result = String::new();
    for node in frag.tree {
        if let scraper::node::Node::Text(text) = node {
            result.push_str(text.text.trim());
        }
    }

    result
}

pub fn contains_only_text(old: &str, new: &str) -> bool {
    let old = get_text_from_xml(old);
    let new = get_text_from_xml(new);

    return old.contains(new.as_str());
}
