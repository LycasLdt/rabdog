use super::{Downloader, DownloaderAssetServer, DownloaderContext, DownloaderDescriptor};
use crate::utils::{decode::decode_cbc_aes, get_next_data};
use anyhow::Result;

const CLIPCC_SB3_URL: &str = "https://api.codingclip.com/v1/project/publicJson";
const CLIPCC_PROJECT_URL: &str = "https://codingclip.com/project/";
// clipccyydsclipccyydsclipccyydscc
const CLIPCC_AES_KEY: [u8; 32] = [
    99, 108, 105, 112, 99, 99, 121, 121, 100, 115, 99, 108, 105, 112, 99, 99, 121, 121, 100, 115,
    99, 108, 105, 112, 99, 99, 121, 121, 100, 115, 99, 99,
];
// clipteamyydsclip
const CLIPCC_AES_IV: [u8; 16] = [
    99, 108, 105, 112, 116, 101, 97, 109, 121, 121, 100, 115, 99, 108, 105, 112,
];
// ·-M8q -> {"ta
const CLIPCC_SOURCE_PREFIX: [u8; 5] = [221, 45, 77, 56, 113];

#[derive(serde::Deserialize)]
struct ClipccData {
    props: ClipccDataProps,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClipccDataProps {
    page_props: ClipccDataPageProps,
}

#[derive(serde::Deserialize)]
struct ClipccDataPageProps {
    project: ClipccProject,
}

#[derive(serde::Deserialize)]
struct ClipccProject {
    name: String,
    user_name: String,
}

#[derive(Default)]
pub struct ClipccDownloader;

#[async_trait::async_trait]
impl Downloader for ClipccDownloader {
    fn descriptor(&self) -> DownloaderDescriptor {
        DownloaderDescriptor {
            display_name: "Clipcc",
            referer: "https://codingclip.com/",
            asset_server: DownloaderAssetServer::same("https://api.codingclip.com/v1/project/asset/")
        }
    }

    async fn get(&self, context: &mut DownloaderContext) -> Result<()> {
        let url = [CLIPCC_PROJECT_URL, &context.id].concat();

        let res = context.client.get(url).send().await?.text().await?;
        let data = get_next_data(&res)?;
        let json = serde_json::from_str::<ClipccData>(&data)?.props.page_props;

        let project_url =
            reqwest::Url::parse_with_params(CLIPCC_SB3_URL, &[("id", context.id.clone())])?;

        context.set_info(project_url, json.project.name, vec![json.project.user_name]);
        Ok(())
    }
    fn decode(&self, context: &mut DownloaderContext) -> Result<()> {
        let buf = context.buffer();
        if buf.starts_with(&CLIPCC_SOURCE_PREFIX) {
            context.set_buffer(decode_cbc_aes(&buf, &CLIPCC_AES_KEY, &CLIPCC_AES_IV)?.into());
        }
        Ok(())
    }
}
