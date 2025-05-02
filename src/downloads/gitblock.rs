use anyhow::{ensure, Result};
use rabdog_schema::schema;
use reqwest::{header, Method};

use crate::utils::decode::decode_cbc_aes;

use super::{Download, DownloadAssetServer, DownloadContext, DownloadDescriptor};

const GITBLOCK_SB3_URL: &str = "https://asset.gitblock.cn/Project/download/";
const GITBLOCK_PROJECT_URL: &str = "https://gitblock.cn/WebApi/Projects/$/Get";
const GITBLOCK_KEY: &str = "4A9745825F24883B657AFC4E4626A0F2";
const GITBLOCK_IV: &str = "4A9745825F24883B";

schema! {
    GitblockResponse;
    access_limit_level: usize,
    access_limit_tips: String,
    project.title: String,
    project.version: usize,
    project.creator.username: String,
}

#[derive(Default)]
pub struct GitblockDownload;

#[async_trait::async_trait]
impl Download for GitblockDownload {
    fn descriptor(&self) -> DownloadDescriptor {
        DownloadDescriptor {
            display_name: "稽木世界",
            referer: "https://gitblock.cn",
            asset_server: DownloadAssetServer::same(
                "https://cdn.gitblock.cn/Project/GetAsset?name=",
            ),
        }
    }

    async fn get(&self, context: &mut DownloadContext) -> Result<()> {
        let project_url = GITBLOCK_PROJECT_URL.replace("$", &context.id);
        let response = context
            .request(Method::POST, project_url)
            .header(header::CONTENT_LENGTH, 0)
            .send()
            .await?;
        let json = response.json::<GitblockResponse>().await?;

        // 该作品有亿点大，暂时被限流。升级VIP或通过2级真人认证后可以访问
        ensure!(json.access_limit_level <= 1, json.access_limit_tips);

        let sb3_url = crate::utils::Url::parse_with_params(
            GITBLOCK_SB3_URL,
            &[("id", context.id.as_str()), ("v", &json.project.version.to_string())],
        )?;

        context.set_info(
            sb3_url,
            json.project.title,
            vec![json.project.creator.username],
        );
        Ok(())
    }

    fn decode(&self, context: &mut DownloadContext) -> Result<()> {
        let result = decode_cbc_aes(&context.buffer(), GITBLOCK_KEY, GITBLOCK_IV)?;

        context.set_buffer(result.into());

        Ok(())
    }
}
