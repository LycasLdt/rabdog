use crate::utils::{
    decode::{decode_base64, decode_cbc_aes, Base64Purpose}, sb3::Sb3Reader
};

use super::{Download, DownloadAssetServer, DownloadContext, DownloadDescriptor};
use anyhow::{anyhow, Result};
use bytes::{BufMut, BytesMut};
use reqwest::Method;

const CCW_DETAIL_URL: &str = "https://community-web.ccw.site/creation/detail";
const CCW_ACCESS_KEY: &str = "";
const BASE64_PREFIX: &str = "KzdnFCBRvq3";
const V2_PREFIX: [u8; 8] = [55, 122, 188, 175, 9, 5, 2, 7];
const ZIP_ARCHIEVE_PREFIX: [u8; 8] = [80, 75, 3, 4, 10, 0, 0, 0];

#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
struct CCWDetailPayload<'a> {
    oid: &'a str,
    access_key: &'static str,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct CCWDetailResponse {
    body: CCWDetailBody,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct CCWDetailBody {
    title: String,
    creation_release: CCWRelease,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct CCWRelease {
    project_link: String,
}

#[derive(Default)]
pub struct CCWDownload;

#[async_trait::async_trait]
impl Download for CCWDownload {
    fn descriptor(&self) -> DownloadDescriptor {
        DownloadDescriptor {
            display_name: "共创世界",
            referer: "https://www.ccw.site/",
            asset_server: DownloadAssetServer::same("https://m.ccw.site/user_projects_assets/")
        }
    }

    async fn get(&self, context: &mut DownloadContext) -> Result<()> {
        let req = context.request(Method::POST, CCW_DETAIL_URL).json(&CCWDetailPayload {
            oid: &context.id,
            access_key: CCW_ACCESS_KEY,
        });
        let res = req.send().await?.json::<CCWDetailResponse>().await?.body;

        context.set_info(res.creation_release.project_link, res.title, Vec::new());
        Ok(())
    }

    fn decode(&self, context: &mut DownloadContext) -> Result<()> {
        let prefix = context.buffer().slice(..8).to_vec();
        let buf = match &prefix {
            p if p == &ZIP_ARCHIEVE_PREFIX => context.buffer().to_vec(),
            p if p == &V2_PREFIX => decode_v2(context.clone())?,
            _ => decode_v3(context.clone())?,
        };

        let mut project = Sb3Reader::from_zip(buf)?.0;
        if !project.starts_with(b"{") {
            project = decode_zip_content(project)?;
        }

        context.set_buffer(project.into());
        Ok(())
    }
}

fn decode_zip_content(content: Vec<u8>) -> Result<Vec<u8>> {
    #[inline]
    fn substr(content: String, mut start: usize, mut end: usize) -> Result<String> {
        if start > end {
            std::mem::swap(&mut start, &mut end)
        }

        let substr = content.get(start..end).unwrap();
        Ok(substr.to_string())
    }

    #[inline]
    fn get_char(content: String, index: usize) -> Result<String> {
        let c = vec![content.as_bytes()[index]];

        Ok(String::from_utf8(c)?)
    }

    #[inline]
    fn transform(content: String) -> Result<String> {
        let length = content.len() - 1;
        let last_char = length % 10;
        let length_char = get_char(content.clone(), length)?;
        let prefix = substr(content.clone(), 0, last_char)?;
        let suffix = substr(content.clone(), last_char + 1, length)?;
        Ok([prefix, length_char, suffix].concat())
    }

    let content = String::from_utf8(content)?;
    let res = transform(content)
        .and_then(|c| decode_base64(c, Base64Purpose::Standard))
        .and_then(|c| {
            Ok(percent_encoding::percent_decode(&c)
                .decode_utf8()?
                .to_string())
        })?;

    Ok(res.into())
}

fn decode_v2(context: DownloadContext) -> Result<Vec<u8>> {
    let buffer = context.buffer();
    let mut buf = BytesMut::with_capacity(buffer.len());
    buf.put_slice(&ZIP_ARCHIEVE_PREFIX);
    buf.extend_from_slice(&buffer[8..]);

    Ok(buf.into())
}

fn decode_v3(context: DownloadContext) -> Result<Vec<u8>> {
    let url = context.url.clone().unwrap();
    let asset_id = url
        .split('/')
        .last()
        .and_then(|f| f.split('.').next())
        .ok_or(anyhow!("incorrect project url"))?;

    let input = decode_base64(context.buffer(), Base64Purpose::Standard)?;
    let key = decode_base64(
        [BASE64_PREFIX, asset_id].concat(),
        Base64Purpose::StandardNoPad,
    )?;
    let key = key.as_slice();
    let iv = &key[..16];

    let buf = decode_cbc_aes(&input, key, iv)
        .and_then(|input| Ok(String::from_utf8(input)?))?
        .split(',')
        .map(|s| s.parse::<u8>().unwrap())
        .collect::<Vec<u8>>();

    Ok(buf)
}
