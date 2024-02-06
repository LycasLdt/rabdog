use anyhow::{Error, Result};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use once_cell::sync::Lazy;
use owo_colors::{AnsiColors, OwoColorize};
use std::fmt::Display;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

static SPINNER_STYLE: Lazy<ProgressStyle> = Lazy::new(|| {
    ProgressStyle::with_template(r#"{spinner:.bold} [{prefix:.bold}] {wide_msg}"#).unwrap()
});

pub type OutputMessage = (NotificationIndex, Notification);

pub enum NotificationIndex {
    Single(usize),
    All,
}
pub enum Notification {
    SelectedDownload { name: &'static str, id: String },
    FetchedProject(String),
    DecodedProject,
    DownloadedAsset(String),
    Finished,
    Canceled,
    Error(Error),
}
impl Display for Notification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Notification::SelectedDownload { .. } => write!(f, "..."),
            Notification::FetchedProject(title) => write!(f, "获取到作品信息 标题: {}", title),
            Notification::DecodedProject => write!(f, "作品解码完成"),
            Notification::DownloadedAsset(asset) => write!(f, "摆好资源: {}", asset),
            Notification::Finished => write!(f, "下载完成"),
            Notification::Canceled => write!(f, "下载作品任务已取消"),
            Notification::Error(err) => write!(f, "遇到错误: {}", err),
        }
    }
}

enum ProgressStatus {
    Running,
    Finished,
    Error,
    Canceled,
}
impl ProgressStatus {
    fn color(&self) -> AnsiColors {
        match self {
            ProgressStatus::Running => AnsiColors::BrightCyan,
            ProgressStatus::Finished => AnsiColors::BrightGreen,
            ProgressStatus::Error => AnsiColors::BrightRed,
            ProgressStatus::Canceled => AnsiColors::Yellow,
        }
    }
}
struct NotifyProgress {
    inner: ProgressBar,

    status: ProgressStatus,
    description: String,
}
impl NotifyProgress {
    fn new(description: String) -> Self {
        let style = Lazy::force(&SPINNER_STYLE);
        let inner = ProgressBar::new_spinner().with_style(style.clone());

        Self {
            inner,
            status: ProgressStatus::Running,
            description,
        }
    }
    fn set_status(&mut self, status: ProgressStatus) {
        self.status = status
    }

    fn remote(&mut self, multi: &mut MultiProgress) {
        let inner = multi.add(self.inner.clone());
        self.inner = inner;
    }
    fn update<M: Display>(&mut self, message: M) {
        self.inner.tick();

        self.inner
            .set_prefix(self.description.color(self.status.color()).to_string());
        self.inner.set_message(message.to_string());

        if !matches!(self.status, ProgressStatus::Running) {
            self.inner.abandon()
        }
    }
}

pub struct OutputReceiver {
    inner: UnboundedReceiver<OutputMessage>,
    bars: Vec<NotifyProgress>,
}
impl OutputReceiver {
    pub fn empty(inner: UnboundedReceiver<OutputMessage>) -> Self {
        Self {
            inner,
            bars: Vec::new(),
        }
    }

    pub async fn sync(&mut self) {
        let mut multi = MultiProgress::new();

        while let Some((index, notification)) = self.inner.recv().await {
            let range = match index {
                NotificationIndex::All => 0..self.bars.len(),
                NotificationIndex::Single(idx) => idx..idx + 1,
            };
            range.for_each(|idx| self.act(&mut multi, idx, &notification))
        }
    }

    fn act(&mut self, multi: &mut MultiProgress, idx: usize, notification: &Notification) {
        if let Notification::SelectedDownload { name, ref id } = notification {
            let description = format!("{} [{}]", name, id);
            let mut bar = NotifyProgress::new(description);
            bar.remote(multi);

            self.bars.push(bar);
        }

        let bar = &mut self.bars[idx];
        let status = match notification {
            Notification::Finished => ProgressStatus::Finished,
            Notification::Error(_) => ProgressStatus::Error,
            Notification::Canceled => ProgressStatus::Canceled,
            _ => ProgressStatus::Running,
        };

        bar.set_status(status);
        bar.update(notification)
    }
}

#[derive(Clone)]
pub struct OutputSender {
    inner: UnboundedSender<OutputMessage>,
}
impl OutputSender {
    pub fn empty(inner: UnboundedSender<OutputMessage>) -> Self {
        Self { inner }
    }
    pub fn send_single(&self, idx: usize, notification: Notification) -> Result<()> {
        self.inner
            .send((NotificationIndex::Single(idx), notification))
            .map_err(|err| err.into())
    }
    pub fn send_global(&self, notification: Notification) -> Result<()> {
        self.inner
            .send((NotificationIndex::All, notification))
            .map_err(|err| err.into())
    }
}

pub fn output_channel() -> (OutputSender, OutputReceiver) {
    let (tx, rx) = unbounded_channel();
    (OutputSender::empty(tx), OutputReceiver::empty(rx))
}