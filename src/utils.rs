use anyhow::{anyhow, Result};
use scraper::Html;

pub use reqwest::Url;
pub mod decode;
pub mod sb3;

#[macro_export]
macro_rules! selector {
    ($name:ident,$s:literal) => {
        static $name: once_cell::sync::Lazy<scraper::Selector> =
            once_cell::sync::Lazy::new(|| scraper::Selector::parse($s).unwrap());
    };
}

selector!(NEXT_DATA_SELECTOR, "#__NEXT_DATA__");

pub fn get_next_data(text: &str) -> Result<String> {
    let document = Html::parse_document(text);
    let element = document
        .select(&NEXT_DATA_SELECTOR)
        .next()
        .ok_or(anyhow!("__NEXT_DATA__"))?;
    let text = element.text().collect::<String>();

    Ok(text)
}
