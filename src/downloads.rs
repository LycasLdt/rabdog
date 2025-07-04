pub mod ccw;
pub mod clipcc;
pub mod cocrea;
pub mod fortycode;
pub mod gitblock;
pub mod scratch_cn;
pub mod scratch;
pub mod xmw;

use std::{io::Write, path::PathBuf, sync::Arc};

use anyhow::Result;
use bytes::Bytes;
use futures::future::try_join_all;
use once_cell::sync::{Lazy, OnceCell};
use regex::Regex;
use reqwest::{header, Client, IntoUrl, Method, RequestBuilder};
use tokio::sync::Mutex;

use crate::{
    output::{Notification, OutputSender},
    utils::sb3::{Sb3Asset, Sb3AssetKind, Sb3Reader, Sb3Writer},
    Config,
};

pub static CONTEXT: OnceCell<(Config, Client, OutputSender)> = OnceCell::new();

pub static USER_AGENT: &str = concat!(
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64)",
    " ",
    "AppleWebKit/537.36 (KHTML, like Gecko)",
    " ",
    "Chrome/131.0.0.0 Safari/537.36"
);
pub const INVALID_PATH: &str = r#"\/:*?"<>|"#;

#[derive(Default, Clone)]
pub struct DownloadDescriptor {
    display_name: &'static str,
    referer: &'static str,
    asset_server: DownloadAssetServer,
}
#[derive(Clone, Default)]
pub struct DownloadAssetServer {
    costumes: &'static str,
    sounds: &'static str,
}
impl DownloadAssetServer {
    pub fn same(url: &'static str) -> Self {
        Self {
            costumes: url,
            sounds: url,
        }
    }
    pub fn split(costumes: &'static str, sounds: &'static str) -> Self {
        Self { costumes, sounds }
    }

    pub async fn download_asset(
        &self,
        writer: &mut Sb3Writer<std::fs::File>,
        asset: Sb3Asset,
        context: DownloadContext,
    ) -> Result<()> {
        let asset_server = match asset.kind {
            Sb3AssetKind::Costume => self.costumes,
            Sb3AssetKind::Sound => self.sounds,
        };
        let url = &[asset_server, &asset.md5ext].concat();

        let res = context.get(url).send().await?.bytes().await?;
        writer.add_asset(&asset.md5ext, &res)?;
        Ok(())
    }
}

#[async_trait::async_trait]
pub trait Download: Sync + Send {
    fn descriptor(&self) -> DownloadDescriptor;

    async fn get(&self, context: &mut DownloadContext) -> Result<()>;
    fn decode(&self, context: &mut DownloadContext) -> Result<()>;
}

#[derive(Clone, Default)]
pub struct DownloadContext {
    pub descriptor: DownloadDescriptor,
    pub id: String,
    pub url: Option<String>,
    pub title: Option<String>,
    pub authors: Vec<String>,
    pub buffer: Option<Bytes>,
}

impl DownloadContext {
    pub fn new(id: String, descriptor: DownloadDescriptor) -> Self {
        DownloadContext {
            id,
            descriptor,
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

    pub fn request<U: IntoUrl>(&self, method: Method, url: U) -> RequestBuilder {
        let (_, client, _) = CONTEXT.get().unwrap();

        let DownloadDescriptor { referer, .. } = self.descriptor;

        client.request(method, url).header(header::REFERER, referer)
    }
    pub fn get<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.request(Method::GET, url)
    }
}

#[derive(Default)]
pub struct DownloadManager {
    downloaders: Vec<(Regex, Lazy<Box<dyn Download>>)>,
}

impl DownloadManager {
    pub fn new() -> Self {
        DownloadManager::default()
    }

    pub fn add(&mut self, matcher: &str, init: fn() -> Box<dyn Download>) {
        self.downloaders
            .push((Regex::new(matcher).unwrap(), Lazy::new(init)));
    }

    pub fn select<'a>(&'a self, source: &'a str) -> Option<Handler<'a>> {
        self.downloaders
            .iter()
            .find(|(r, _)| r.is_match(source))
            .map(move |(r, p)| {
                let caps = r.captures(source).unwrap();
                let id = caps.name("id").unwrap().as_str();

                Handler::new(id, Lazy::force(p).as_ref())
            })
    }

    pub fn is_valid(&self, source: &str) -> bool {
        self.downloaders.iter().any(|(r, _)| r.is_match(source))
    }
}

#[derive(Clone)]
pub struct Handler<'a> {
    idx: Option<usize>,
    downloader: &'a dyn Download,
    context: DownloadContext,
}
impl<'a> Handler<'a> {
    pub fn new(id: &'a str, downloader: &'a dyn Download) -> Self {
        let context = DownloadContext::new(id.to_owned(), downloader.descriptor());

        Self {
            idx: None,
            downloader,
            context,
        }
    }

    pub async fn download(&mut self, idx: usize) {
        let (_, _, tx) = CONTEXT.get().unwrap();
        self.idx = Some(idx);

        if let Err(err) = self.download_inner().await {
            tx.send_single(idx, Notification::Error(err)).unwrap();
        }
    }

    async fn download_inner(&mut self) -> Result<()> {
        let (config, _, tx) = CONTEXT.get().unwrap();
        let idx = self.idx.unwrap();

        tx.send_single(
            idx,
            Notification::SelectedDownload {
                name: self.downloader.descriptor().display_name,
                id: self.context.id.clone(),
            },
        )?;

        self.downloader.get(&mut self.context).await.and_then(|_| {
            let title = self.context.clone().title.unwrap();
            tx.send_single(idx, Notification::FetchedProject(title))
        })?;
        self.get_buffer().await?;
        self.downloader
            .decode(&mut self.context)
            .and_then(|_| tx.send_single(idx, Notification::DecodedProject))?;
        self.pack_sb3(config.path.clone()).await?;
        tx.send_single(idx, Notification::Finished)?;

        Ok(())
    }
    async fn get_buffer(&mut self) -> Result<()> {
        if self.context.buffer.is_some() {
            return Ok(());
        }

        let url = self.context.url.clone().unwrap();
        let res = self.context.get(url).send().await?;

        self.context.set_buffer(res.bytes().await?);
        Ok(())
    }
    async fn pack_sb3(&self, mut path: PathBuf) -> Result<()> {
        let DownloadDescriptor { asset_server, .. } = self.downloader.descriptor();
        let (config, _, _) = CONTEXT.get().unwrap();

        let context = &self.context;
        let mut title = context.title.clone().unwrap();
        title.retain(|c| !INVALID_PATH.contains(c));

        path.push(title);
        path.set_extension(if config.no_assets { "json" } else { "sb3" });

        let mut file = std::fs::File::create(path)?;

        if config.no_assets {
            file.write_all(&context.buffer())?;
            return Ok(());
        }

        let writer = Arc::new(Mutex::new(Sb3Writer::new(file)));
        writer.lock().await.set_project_json(context.buffer())?;

        let reader = Sb3Reader::parse(context.buffer());
        let assets = reader.assets()?.into_iter().map(|asset| async {
            let (_, _, tx) = CONTEXT.get().unwrap();
            let arc = Arc::clone(&writer);
            let mut writer = arc.lock().await;

            tx.send_single(
                self.idx.unwrap(),
                Notification::DownloadedAsset(asset.md5ext.clone()),
            )?;
            asset_server
                .download_asset(&mut writer, asset, context.clone())
                .await
        });

        if let Some(extensions) = reader.community_extensions()? {
            let (_, _, tx) = CONTEXT.get().unwrap();
            tx.send_single(
                self.idx.unwrap(),
                Notification::WarnCommunityExtensions(extensions),
            )?;
        }

        try_join_all(assets).await?;

        Ok(())
    }
}

pub fn setup_static(config: Config, tx: OutputSender) {
    let client = Client::builder().user_agent(USER_AGENT).build().unwrap();

    CONTEXT.get_or_init(|| (config, client, tx));
}
