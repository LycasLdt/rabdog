use anyhow::Result;
use chrono::Utc;
use serde::Deserialize;

use crate::utils::{
    self,
    decode::{compute_md5, decode_cbc_aes, decode_hex},
    get_next_data,
};

use super::{Download, DownloadAssetServer, DownloadContext, DownloadDescriptor};

const XMW_PROJECT_URL: &str = "https://world.xiaomawang.com/community/main/compose/";
const XMW_SB3_URL: &str =
    "https://community-api.xiaomawang.com/japi/v1/composition/get-encrypt-sb3";
const XMW_AES_KEY: &str = "xmwcommunityskey";
const XMW_AES_IV: &str = "0392139263920300";
const XMW_PROJECT_KEY_PREFIX: &str = "xiaomw135";

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
pub struct XMWDownload;

#[async_trait::async_trait]
impl Download for XMWDownload {
    fn descriptor(&self) -> DownloadDescriptor {
        DownloadDescriptor {
            display_name: "小码王",
            referer: "https://world.xiaomawang.com/",
            asset_server: DownloadAssetServer::split(
                "https://community-wscdn.xiaomawang.com/picture/",
                "https://community-wscdn.xiaomawang.com/audio/",
            ),
        }
    }

    async fn get(&self, context: &mut DownloadContext) -> Result<()> {
        let url = [XMW_PROJECT_URL, &context.id].concat();
        let html = context.get(&url).send().await?.text().await?;

        let data = get_next_data(&html)?;
        let json = serde_json::from_str::<XMWData>(&data)?
            .props
            .initial_state
            .detail;
        let project_url = crate::utils::Url::parse_with_params(
            XMW_SB3_URL,
            &[("compositionEncryptId", context.id.clone())],
        )?;
        context.set_info(project_url.clone(), json.compose_info.title, Vec::new());

        let res = context.get(project_url).send().await?;
        let data = res.json::<XMWProjectEncodedSb3>().await?.data;
        let buffer = match utils::Url::parse(&data) {
            Ok(url) => {
                let timestamp = Utc::now().timestamp().to_string();
                let key = compute_md5([XMW_PROJECT_KEY_PREFIX, &timestamp].concat());
                let query = &[("key", key), ("time", timestamp)];

                let res = context.get(url).query(query).send().await?;
                res.bytes().await?
            }
            Err(_) => data.into(),
        };

        context.set_buffer(buffer);
        Ok(())
    }
    fn decode(&self, context: &mut DownloadContext) -> Result<()> {
        let buf = context.buffer().to_vec();
        let hex = String::from_utf8(buf)?;
        if hex.starts_with('{') {
            return Ok(());
        }

        let input = decode_hex(hex)?;
        let buf = decode_cbc_aes(&input, XMW_AES_KEY, XMW_AES_IV)?;

        context.set_buffer(buf.into());
        Ok(())
    }
}
