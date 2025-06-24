use anyhow::{anyhow, Result};

use super::{Download, DownloadAssetServer, DownloadContext, DownloadDescriptor};

const SCRATCH_API_PROXY_URL: &str = "https://trampoline.turbowarp.org/api/projects/";
const SCRATCH_SB3_PROXY_URL: &str = "https://chilipar.alibga.icu/projects";

#[derive(serde::Deserialize)]
pub struct ScratchResponse {
    pub title: String,
    pub project_token: String,
    pub author: _ScratchResponseAuthor,
}
#[derive(serde::Deserialize)]
pub struct _ScratchResponseAuthor {
    pub username: String,
}

pub struct ScratchDownload;

#[async_trait::async_trait]
impl Download for ScratchDownload {
    fn descriptor(&self) -> DownloadDescriptor {
        DownloadDescriptor {
            display_name: "Scratch",
            referer: "https://scratch.mit.edu/",
            asset_server: DownloadAssetServer::same("https://chilipar.alibga.icu/assets/"),
        }
    }
    async fn get(&self, context: &mut DownloadContext) -> Result<()> {
        let project_url = [SCRATCH_API_PROXY_URL, &context.id].concat();

        let res = context.get(project_url).send().await?;

        let json = res.json::<ScratchResponse>().await?;

        let mut sb3_url = crate::utils::Url::parse(SCRATCH_SB3_PROXY_URL)?;
        sb3_url
            .path_segments_mut()
            .map_err(|_| anyhow!("cannot be base"))?
            .push(&context.id);
        sb3_url
            .query_pairs_mut()
            .append_pair("token", &json.project_token);

        context.set_info(sb3_url, json.title, vec![json.author.username]);

        Ok(())
    }
    fn decode(&self, _: &mut DownloadContext) -> Result<()> {
        Ok(())
    }
}
