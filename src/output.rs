use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use once_cell::sync::Lazy;
use std::fmt::Display;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

static SPINNER_STYLE: Lazy<ProgressStyle> = Lazy::new(|| {
    ProgressStyle::with_template(r#"{spinner:.dim.bold} [{prefix:.cyan.bold}] {wide_msg}"#).unwrap()
});

pub type OutputSender = UnboundedSender<(usize, Notification)>;
pub type OutputReceiver = UnboundedReceiver<(usize, Notification)>;

pub enum Notification {
    SelectedDownload { name: &'static str, id: String },
    FetchedProject(String),
    DecodedProject,
    DownloadedAsset(String),
    Finished,
}
impl Display for Notification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Notification::SelectedDownload { name, id } => {
                write!(f, "{} [{}]", name, id)
            }
            Notification::FetchedProject(title) => write!(f, "获取到作品信息 标题: {}", title),
            Notification::DecodedProject => write!(f, "作品解码完成"),
            Notification::DownloadedAsset(asset) => write!(f, "摆好资源: {}", asset),
            Notification::Finished => write!(f, "下载完成"),
        }
    }
}

pub struct OutputSession {
    receiver: OutputReceiver,
    bars: Vec<ProgressBar>,
}
impl OutputSession {
    pub fn empty(receiver: OutputReceiver) -> Self {
        Self {
            receiver,
            bars: Vec::new(),
        }
    }

    pub async fn sync(&mut self) {
        let mut multi = MultiProgress::new();

        while let Some((idx, notification)) = self.receiver.recv().await {
            self.act(&mut multi, idx, notification)
        }
    }

    fn act(&mut self, multi: &mut MultiProgress, idx: usize, notification: Notification) {
        let style = Lazy::force(&SPINNER_STYLE);

        if let Notification::SelectedDownload { .. } = notification {
            let spinner = ProgressBar::new_spinner()
                .with_style(style.clone())
                .with_prefix(notification.to_string());
            let bar = multi.add(spinner);

            self.bars.push(bar);
        }

        let bar = &self.bars[idx];
        bar.tick();
        bar.set_message(notification.to_string());

        if let Notification::Finished = notification {
            bar.abandon()
        }
    }
}
