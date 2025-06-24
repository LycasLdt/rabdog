use crate::utils::{
    decode::{decode_base64, decode_cbc_aes, Base64Purpose},
    sb3::Sb3Reader,
};

use super::{Download, DownloadAssetServer, DownloadContext, DownloadDescriptor};
use anyhow::{anyhow, Result};
use bytes::{BufMut, BytesMut};
use rabdog_schema::schema;
use reqwest::Method;

const CCW_DETAIL_URL: &str = "https://community-web.ccw.site/creation/detail";
const CCW_ACCESS_KEY: &str = "";
const BASE64_PREFIX: &str = "KzdnFCBRvq3";
const V2_PREFIX: [u8; 8] = [55, 122, 188, 175, 9, 5, 2, 7];
const ZIP_ARCHIEVE_PREFIX: [u8; 8] = [80, 75, 3, 4, 10, 0, 0, 0];

schema! {
    CCWDetailResponse;
    body.title: String,
    body.creation_release.project_link: String
}

#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
struct CCWDetailPayload<'a> {
    oid: &'a str,
    access_key: &'static str,
}

#[derive(Default)]
pub struct CCWDownload;

#[async_trait::async_trait]
impl Download for CCWDownload {
    fn descriptor(&self) -> DownloadDescriptor {
        DownloadDescriptor {
            display_name: "共创世界",
            referer: "https://www.ccw.site/",
            asset_server: DownloadAssetServer::same("https://m.ccw.site/user_projects_assets/"),
        }
    }

    async fn get(&self, context: &mut DownloadContext) -> Result<()> {
        let req = context
            .request(Method::POST, CCW_DETAIL_URL)
            .json(&CCWDetailPayload {
                oid: &context.id,
                access_key: CCW_ACCESS_KEY,
            });
        let res = req.send().await?.json::<CCWDetailResponse>().await?.body;

        context.set_info(res.creation_release.project_link, res.title, Vec::new());
        Ok(())
    }

    fn decode(&self, context: &mut DownloadContext) -> Result<()> {
        let decoder = CCWLikeDecoder::new();

        context.set_buffer(decoder.decode(&context.url.clone().unwrap(), context.buffer())?);

        Ok(())
    }
}

pub struct CCWLikeDecoder;

impl CCWLikeDecoder {
    pub fn new() -> CCWLikeDecoder {
        CCWLikeDecoder
    }

    pub fn decode(&self, url: &str, bytes: bytes::Bytes) -> Result<bytes::Bytes> {
        let prefix = bytes.clone().slice(..8).to_vec();
        let buf = match &prefix {
            p if p == &ZIP_ARCHIEVE_PREFIX => bytes.to_vec(),
            p if p == &V2_PREFIX => self.decode_v2(bytes)?,
            _ => self.decode_v3(url, bytes)?,
        };

        let mut project = Sb3Reader::from_zip(buf)?.0;
        if !project.starts_with(b"{") {
            project = self.decode_zip_content(project)?;
        }

        Ok(project.into())
    }

    fn decode_zip_content(&self, content: Vec<u8>) -> Result<Vec<u8>> {
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

    fn decode_v2(&self, bytes: bytes::Bytes) -> Result<Vec<u8>> {
        let mut buf = BytesMut::with_capacity(bytes.len());
        buf.put_slice(&ZIP_ARCHIEVE_PREFIX);
        buf.extend_from_slice(&bytes[8..]);

        Ok(buf.into())
    }
    fn decode_v3(&self, url: &str, bytes: bytes::Bytes) -> Result<Vec<u8>> {
        let asset_id = url
            .split('/')
            .next_back()
            .and_then(|f| f.split('.').next())
            .ok_or(anyhow!("incorrect project url"))?;

        let input = decode_base64(bytes, Base64Purpose::Standard)?;
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
}
