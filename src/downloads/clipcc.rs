use super::{Download, DownloadAssetServer, DownloadContext, DownloadDescriptor};
use crate::utils::{decode::decode_cbc_aes, get_next_data};
use anyhow::Result;
use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::Utc;
use rabdog_schema::schema;
use rsa::{pkcs8::DecodePublicKey, Pkcs1v15Encrypt, RsaPublicKey};

const CLIPCC_SB3_URL: &str = "https://api.codingclip.com/v1/project/download";
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
const CLIPCC_PUBLIC_KEY: &str = r#"-----BEGIN PUBLIC KEY-----
MIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQCzOaIJxii0ItmbVx1/lWTJxGht
M/sPHGRyX/n4u7XFy89C+BPweyhowXMVvoN8aJivSrUC8wwn3/fDbq3PLF8Wm+37
fmZw7JJssyEsow4x/TE6N9b0Hq8mYwLXHSAWWBHL0uzQeRtxfa9ZQsvpkGW/VoBJ
CP/tf54FNKZWpN+VZwIDAQAB
-----END PUBLIC KEY-----
"#;

schema! {
    ClipccData;
    props.page_props.project.name: String,
    props.page_props.project.user_name: String
}

#[derive(Default)]
pub struct ClipccDownload;

#[async_trait::async_trait]
impl Download for ClipccDownload {
    fn descriptor(&self) -> DownloadDescriptor {
        DownloadDescriptor {
            display_name: "Clipcc",
            referer: "https://codingclip.com/",
            asset_server: DownloadAssetServer::same("https://api.codingclip.com/v1/project/asset/"),
        }
    }

    async fn get(&self, context: &mut DownloadContext) -> Result<()> {
        let url = [CLIPCC_PROJECT_URL, &context.id].concat();

        let res = context.get(url).send().await?.text().await?;
        let data = get_next_data(&res)?;
        let json = serde_json::from_str::<ClipccData>(&data)?.props.page_props;

        let timestamp = Utc::now().timestamp_millis().to_string();
        let asset_id = ["public", &timestamp, &context.id].join("|");

        let mut rng = rand::thread_rng();
        let public_key = RsaPublicKey::from_public_key_pem(CLIPCC_PUBLIC_KEY)?;
        let keys = public_key.encrypt(&mut rng, Pkcs1v15Encrypt, asset_id.as_bytes())?;
        let keys = STANDARD.encode(keys);

        let project_url = crate::utils::Url::parse_with_params(CLIPCC_SB3_URL, &[("keys", keys)])?;

        context.set_info(project_url, json.project.name, vec![json.project.user_name]);
        Ok(())
    }
    fn decode(&self, context: &mut DownloadContext) -> Result<()> {
        let buf = context.buffer();
        if buf.starts_with(&CLIPCC_SOURCE_PREFIX) {
            context.set_buffer(decode_cbc_aes(&buf, CLIPCC_AES_KEY, CLIPCC_AES_IV)?.into());
        }
        Ok(())
    }
}
