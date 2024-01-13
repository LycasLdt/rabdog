use crate::downloaders::{
    ccw::CCWDownloader, clipcc::ClipccDownloader, download, DownloaderManager, MANAGER_INSTANCE,
};
use clap::Parser;
use downloaders::xmw::XMWDownloader;

mod decoder;
mod downloaders;
mod output;
mod utils;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Config {
    /// The url of the project
    #[arg(value_parser = is_source_valid)]
    source: String,

    /// The path to save the file
    #[arg(short, long, default_value = "/")]
    path: String,
}

fn is_source_valid(source: &str) -> Result<String, String> {
    let m = MANAGER_INSTANCE.get().unwrap();

    match m.is_valid(source) {
        true => Ok(source.into()),
        false => Err("没有能胜任此链接的下载器".into()),
    }
}

fn main() -> anyhow::Result<()> {
    let _ = MANAGER_INSTANCE.get_or_init(|| {
        DownloaderManager::new()
            .add(
                r"^((https|http):\/\/)?(www\.)?ccw\.site\/detail\/(?<id>[a-z0-9]{24})(\?.*)?",
                || Box::new(CCWDownloader),
            )
            .add(
                r"^((https|http):\/\/)?codingclip\.com\/project\/(?<id>[0-9]+)(\?.*)?",
                || Box::new(ClipccDownloader),
            )
            .add(
                r"^((https|http):\/\/)?world.xiaomawang.com\/community\/main\/compose\/(?<id>[a-zA-Z0-9]{8})(\?.*)?",
                || Box::new(XMWDownloader),
            )
    });

    let cfg = Config::parse();

    download(cfg)
}
