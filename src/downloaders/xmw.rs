use anyhow::Result;
use serde::Deserialize;

use crate::utils::{
    decode::{decode_cbc_aes, decode_hex},
    get_next_data,
};

use super::{Downloader, DownloaderAssetServer, DownloaderContext, DownloaderDescriptor};

const XMW_PROJECT_URL: &str = "https://world.xiaomawang.com/community/main/compose/";
const XMW_SB3_URL: &str =
    "https://community-api.xiaomawang.com/japi/v1/composition/get-encrypt-sb3";
const XMW_AES_KEY: &str = "xmwcommunityskey";
const XMW_AES_IV: &str = "0392139263920300";

#[derive(Deserialize)]
struct XMWData {
    props: XMWPropsData,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct XMWPropsData {
    initial_state: XMWInitialState,
}
#[derive(Deserialize)]
struct XMWInitialState {
    detail: XMWDetail,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct XMWDetail {
    compose_info: XMWProject,
}
#[derive(Deserialize)]
struct XMWProject {
    title: String,
}
#[derive(Deserialize)]
struct XMWProjectEncodedSb3 {
    data: String,
}

#[derive(Default)]
pub struct XMWDownloader;

#[async_trait::async_trait]
impl Downloader for XMWDownloader {
    fn descriptor(&self) -> DownloaderDescriptor {
        DownloaderDescriptor {
            display_name: "小码王",
            referer: "https://world.xiaomawang.com/",
            asset_server: DownloaderAssetServer::split(
                "https://community-wscdn.xiaomawang.com/picture/",
                "https://community-wscdn.xiaomawang.com/audio/",
            ),
        }
    }

    async fn get(&self, context: &mut DownloaderContext) -> Result<()> {
        let url = [XMW_PROJECT_URL, &context.id].concat();
        let html = context.client.get(&url).send().await?.text().await?;

        let data = get_next_data(&html)?;
        let json = serde_json::from_str::<XMWData>(&data)?
            .props
            .initial_state
            .detail;
        let project_url = reqwest::Url::parse_with_params(
            XMW_SB3_URL,
            &[("compositionEncryptId", context.id.clone())],
        )?;
        context.set_info(project_url, json.compose_info.title, Vec::new());

        Ok(())
    }
    fn decode(&self, context: &mut DownloaderContext) -> Result<()> {
        let content = context
            .buffer()
            .into_iter()
            .filter(|s| !s.is_ascii_whitespace())
            .collect::<Vec<u8>>();

        let hex = serde_json::from_slice::<XMWProjectEncodedSb3>(&content)?.data;
        let input = decode_hex(&hex)?;
        let buf = decode_cbc_aes(&input, XMW_AES_KEY, XMW_AES_IV)?;

        context.set_buffer(buf.into());
        Ok(())
    }
}
