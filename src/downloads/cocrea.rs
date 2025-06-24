use anyhow::Result;
use chrono::Utc;
use rabdog_schema::schema;

use super::{Download, DownloadAssetServer, DownloadContext, DownloadDescriptor};
use crate::{downloads::ccw::CCWLikeDecoder, utils::get_next_data};

const COCREA_PROJECT_URL: &str = "https://www.cocrea.world/";

schema! {
    CocreaData;
    props.page_props.creation_data.title: String,
    props.page_props.creation_data.author.username: String,
    props.page_props.creation_data.creation_release_resp.project_link: String
}

pub struct CocreaDownload;

#[async_trait::async_trait]
impl Download for CocreaDownload {
    fn descriptor(&self) -> DownloadDescriptor {
        DownloadDescriptor {
            display_name: "Cocrea World",
            referer: "https://www.cocrea.world/",
            asset_server: DownloadAssetServer::same(
                "https://assets.cocrea.world/user_projects_assets/",
            ),
        }
    }
    async fn get(&self, context: &mut DownloadContext) -> Result<()> {
        let url = [COCREA_PROJECT_URL, &context.id].concat();
        let res = context.get(url).send().await?.text().await?;
        let data = get_next_data(&res)?;
        let json = serde_json::from_str::<CocreaData>(&data)?.props.page_props;

        let timestamp = Utc::now().timestamp_millis().to_string();
        let project_url = crate::utils::Url::parse_with_params(
            &json.creation_data.creation_release_resp.project_link,
            &[("t", timestamp)],
        )?;

        context.set_info(
            project_url,
            json.creation_data.title,
            vec![json.creation_data.author.username],
        );

        Ok(())
    }
    fn decode(&self, context: &mut DownloadContext) -> Result<()> {
        let decoder = CCWLikeDecoder::new();

        context.set_buffer(decoder.decode(&context.url.clone().unwrap(), context.buffer())?);

        Ok(())
    }
}
