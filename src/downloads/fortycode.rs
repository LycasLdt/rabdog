use anyhow::Result;
use serde::Deserialize;

use crate::utils::decode::{decode_cbc_aes, decode_hex};

use super::{Download, DownloadAssetServer, DownloadContext, DownloadDescriptor};

const FORTYCODE_PROJECT_URL: &str =
    "https://service-dq726wx5-1302921490.sh.apigw.tencentcs.com/work/info";
const FORTYCODE_SB3_URL: &str =
    "https://service-dq726wx5-1302921490.sh.apigw.tencentcs.com/work/work";
const FORTYCODE_AES_KEY: &str = "9609274736591562";
const FORTYCODE_AES_IV: &str = "4312549111852919";

#[derive(Deserialize)]
pub struct FortycodeResponse {
    data: FortycodeData,
}
#[derive(Deserialize)]
pub struct FortycodeData {
    name: String,
    nickname: String,
}

pub struct FortycodeDownload;

#[async_trait::async_trait]
impl Download for FortycodeDownload {
    fn descriptor(&self) -> DownloadDescriptor {
        DownloadDescriptor {
            display_name: "40code",
            referer: "https://www.40code.com/",
            asset_server: DownloadAssetServer::same(
                "https://40code-cdn.zq990.com/static/internalapi/asset/",
            ),
        }
    }
    async fn get(&self, context: &mut DownloadContext) -> Result<()> {
        let query = [
            ("id", context.id.as_str()),
            ("sha", ""),
            ("etime", ""),
            ("token", ""),
        ];
        let req = context.get(FORTYCODE_PROJECT_URL).query(&query);
        let res = req.send().await?.json::<FortycodeResponse>().await?;
        let sb3_url = crate::utils::Url::parse_with_params(FORTYCODE_SB3_URL, &query)?;
        context.set_info(sb3_url, res.data.name, vec![res.data.nickname]);

        Ok(())
    }
    fn decode(&self, context: &mut DownloadContext) -> Result<()> {
        let buf = context.buffer().to_vec();
        let hex = String::from_utf8(buf)?;
        if hex.starts_with('{') {
            return Ok(());
        }

        let input = decode_hex(&hex)?;
        let buf = decode_cbc_aes(&input, FORTYCODE_AES_KEY, FORTYCODE_AES_IV)?;

        context.set_buffer(buf.into());
        Ok(())
    }
}
