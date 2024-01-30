use anyhow::Result;
use scraper::Html;

use crate::selector;

use super::{Download, DownloadAssetServer, DownloadContext, DownloadDescriptor};

const SCRATCHCN_SB3_URL: &str = "https://www.xiaoyaqian.cn/userfile/scratch/";
const SCRATCHCN_PROJECT_URL: &str = "https://www.scratch-cn.cn/project/?comid=";

selector!(PROJECT_ID_SELECTOR, "#_s_");
selector!(PROJECT_TITLE_SELECTOR, ".work-title > h3");

pub struct ScratchCNDownload;

#[async_trait::async_trait]
impl Download for ScratchCNDownload {
    fn descriptor(&self) -> DownloadDescriptor {
        DownloadDescriptor {
            display_name: "Scratch中社",
            referer: "https://www.scratch-cn.cn/",
            asset_server: DownloadAssetServer::same(
                "https://www.rgfpz.cn/scratch/00a6ad64232a90b4f6f5cc859b9d7f53/",
            ),
        }
    }
    async fn get(&self, context: &mut DownloadContext) -> Result<()> {
        let project_url = [SCRATCHCN_PROJECT_URL, &context.id].concat();
        let res = context.get(project_url).send().await?.text().await?;

        let document = Html::parse_document(&res);
        let project_id = document
            .select(&PROJECT_ID_SELECTOR)
            .next()
            .unwrap()
            .attr("value")
            .unwrap();
        let title = document
            .select(&PROJECT_TITLE_SELECTOR)
            .next()
            .unwrap()
            .text()
            .next()
            .unwrap();

        let sb3_url = [SCRATCHCN_SB3_URL, project_id].concat();
        context.set_info(sb3_url, title.to_owned(), vec![]);

        Ok(())
    }
    fn decode(&self, _: &mut DownloadContext) -> Result<()> {
        Ok(())
    }
}
