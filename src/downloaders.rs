pub mod ccw;
pub mod clipcc;
pub mod xmw;

use std::{io::Write, path::PathBuf, str::FromStr};

use anyhow::Result;
use bytes::Bytes;
use once_cell::sync::{Lazy, OnceCell};
use regex::Regex;
use reqwest::{Client, ClientBuilder};
use tokio::runtime::Runtime;
use zip::{write::FileOptions, ZipWriter};

use crate::{
    output::{log_error, log_with_progress},
    Config,
};

pub static MANAGER_INSTANCE: OnceCell<DownloaderManager> = OnceCell::new();

pub static USER_AGENT: &str = concat!(
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64)",
    " ",
    "AppleWebKit/537.36 (KHTML, like Gecko)",
    " ",
    "Chrome/114.0.0.0 Safari/537.36"
);

#[async_trait::async_trait]
pub trait Downloader: Sync + Send {
    fn display_name(&self) -> &'static str;
    fn assets_server(&self) -> &'static str;

    async fn get(&self, context: &mut DownloaderContext) -> Result<()>;
    fn decode(&self, context: &mut DownloaderContext) -> Result<()>;
}

#[derive(Debug, Clone, Default)]
pub struct DownloaderContext {
    pub client: Client,
    pub id: String,
    pub url: Option<String>,
    pub title: Option<String>,
    pub authors: Vec<String>,
    pub buffer: Option<Bytes>,
}

impl DownloaderContext {
    pub fn new(client: Client, id: String) -> Self {
        DownloaderContext {
            client,
            id,
            ..Default::default()
        }
    }

    pub fn set_info<U: Into<String>>(&mut self, url: U, title: String, authors: Vec<String>) {
        (self.url, self.title, self.authors) = (Some(url.into()), Some(title), authors);
    }

    pub fn set_buffer(&mut self, buffer: Bytes) {
        self.buffer = Some(buffer);
    }

    pub fn buffer(&self) -> Bytes {
        self.buffer.clone().unwrap()
    }
}

#[derive(Default)]
pub struct DownloaderManager {
    downloaders: Vec<(Regex, Lazy<Box<dyn Downloader>>)>,
}

impl DownloaderManager {
    pub fn new() -> Self {
        DownloaderManager::default()
    }

    pub fn add(mut self, matcher: &str, init: fn() -> Box<dyn Downloader>) -> Self {
        self.downloaders
            .push((Regex::new(matcher).unwrap(), Lazy::new(init)));

        self
    }

    pub fn select<'a>(&'a self, source: &'a str) -> Option<(&str, &Box<dyn Downloader>)> {
        self.downloaders
            .iter()
            .find(|(r, _)| r.is_match(source))
            .and_then(move |(r, p)| {
                let caps = r.captures(source).unwrap();
                let id = caps.name("id").unwrap().as_str();

                Some((id, Lazy::force(&p)))
            })
    }

    pub fn is_valid(&self, source: &str) -> bool {
        self.downloaders.iter().any(|(r, _)| r.is_match(source))
    }
}

async fn get_buffer(context: &mut DownloaderContext) -> Result<()> {
    let url = context.url.clone().unwrap();
    let res = context.client.get(url).send().await?;

    context.set_buffer(res.bytes().await?);
    Ok(())
}

fn pack_sb3(path: &str, context: DownloaderContext) -> Result<()> {
    let mut path = PathBuf::from_str(path)?;
    path.push(context.title.unwrap());
    path.set_extension("sb3");

    let file = std::fs::File::create(path)?;
    let mut writer = ZipWriter::new(file);
    writer.start_file(
        "project.json",
        FileOptions::default().compression_method(zip::CompressionMethod::Deflated),
    )?;
    writer.write_all(&context.buffer.unwrap())?;
    writer.finish()?;

    Ok(())
}

pub fn download(cfg: Config) -> Result<()> {
    let m = MANAGER_INSTANCE.get().unwrap();
    let (id, downloader) = m.select(&cfg.source).expect("");

    log_with_progress(
        "[0/3]",
        format!(
            "检测到链接为 {} 作品, 作品ID {}",
            downloader.display_name(),
            id
        ),
    );

    let rt = Runtime::new().unwrap();
    let client = ClientBuilder::new().user_agent(USER_AGENT).build().unwrap();

    let status: Result<()> = rt.block_on(async {
        let mut context = DownloaderContext::new(client, id.into());

        log_with_progress("[1/3]", "获取作品信息...");
        downloader.get(&mut context).await?;

        log_with_progress(
            "[1/3]",
            format!(
                "获取作品内容, 作品链接: {} ...",
                context.url.clone().unwrap()
            ),
        );
        get_buffer(&mut context).await?;

        log_with_progress("[2/3]", "解码作品内容...");
        downloader.decode(&mut context)?;

        log_with_progress("[3/3]", "下载作品内容...");
        pack_sb3(&cfg.path, context)?;

        Ok(())
    });

    if let Err(err) = status {
        log_error(err)
    }

    drop(rt);
    Ok(())
}
