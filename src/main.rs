use std::path::PathBuf;

use crate::downloads::setup_static;
use crate::downloads::{
    ccw::CCWDownload, clipcc::ClipccDownload, fortycode::FortycodeDownload,
    scratch_cn::ScratchCNDownload, xmw::XMWDownload,
};
use crate::output::OutputSession;

use clap::{value_parser, Parser};
use futures::future::try_join_all;
use once_cell::sync::Lazy;
use tokio::{runtime::Runtime, sync::mpsc::unbounded_channel};

mod downloads;
mod output;
mod utils;

macro_rules! downloads {
    ($manager:ident; $($init:expr => $matcher:literal),*) => {
        static $manager: once_cell::sync::Lazy<$crate::downloads::DownloadManager> = once_cell::sync::Lazy::new(|| {
            let mut manager = $crate::downloads::DownloadManager::new();
            $(
                manager.add($matcher, || Box::new($init));
            )*
            manager
        });
    };
}

downloads!(MANAGER;
    CCWDownload => r"^((https|http):\/\/)?(www\.)?ccw\.site\/detail\/(?<id>[a-z0-9]{24})(\?.*)?",
    ClipccDownload => r"^((https|http):\/\/)?codingclip\.com\/project\/(?<id>[0-9]+)(\?.*)?",
    XMWDownload => r"^((https|http):\/\/)?world.xiaomawang.com\/community\/main\/compose\/(?<id>[a-zA-Z0-9]{8})(\?.*)?",
    ScratchCNDownload => r"^((https|http):\/\/)?(www\.)?scratch-cn.cn\/project\/\?comid=(?<id>[a-zA-Z0-9]{24})(\?.*)?",
    FortycodeDownload => r"^((https|http):\/\/)?(www\.)?40code.com\/#page=work&id=(?<id>[0-9]+)(\?.*)?"
);

#[derive(Parser, Clone)]
#[command(arg_required_else_help(true), version, about, long_about = None)]
pub struct Config {
    /// 社区作品链接
    #[arg(required(true), value_parser = is_source_valid)]
    sources: Vec<String>,
    /// .sb3 文件存储路径
    #[arg(short, long, value_parser = value_parser!(PathBuf), default_value = ".")]
    path: PathBuf,
    /// 是否只下载 .sb3 文件中的 project.json
    #[arg(short, long)]
    no_assets: bool,
}

fn is_source_valid(source: &str) -> Result<String, String> {
    let m = Lazy::force(&MANAGER);

    match m.is_valid(source) {
        true => Ok(source.into()),
        false => Err("没有能胜任此链接的下载器".into()),
    }
}

fn main() -> anyhow::Result<()> {
    let (tx, rx) = unbounded_channel();
    let mut session = OutputSession::empty(rx);

    let config = Config::parse();
    setup_static(config.clone(), tx.clone());

    let manager = Lazy::force(&MANAGER);
    let sources = config.sources.as_slice();
    let tasks = sources.into_iter().enumerate().filter_map(|(idx, source)| {
        let mut download = manager.select(source)?;
        Some(async move { download.download(idx).await })
    });

    let rt = Runtime::new()?;
    let _ = rt.block_on(async {
        tokio::select! {
            _ = try_join_all(tasks) => drop(tx),
            _ = session.sync() => ()
        }
    });

    Ok(())
}
