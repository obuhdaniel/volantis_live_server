use crate::error::{AppError, AppResult};
use livekit_api::services::egress::{
    EgressClient, EgressListFilter, EgressListOptions, EgressOutput, RoomCompositeOptions,
};
use livekit_protocol::{
    EgressInfo, EncodedFileOutput, EncodedFileType, S3Upload, encoded_file_output,
};

pub struct EgressService {
    client: EgressClient,
    s3_access_key: String,
    s3_secret: String,
    s3_region: String,
    s3_bucket: String,
}

impl EgressService {
    pub fn new(
        url: &str,
        s3_access_key: String,
        s3_secret: String,
        s3_region: String,
        s3_bucket: String,
    ) -> AppResult<Self> {
        let client = EgressClient::new(url).map_err(|e| AppError::LiveKit(e.to_string()))?;
        Ok(Self {
            client,
            s3_access_key,
            s3_secret,
            s3_region,
            s3_bucket,
        })
    }

    pub async fn start_room_recording(
        &self,
        room: &str,
        filepath: Option<String>,
        layout: Option<String>,
        audio_only: bool,
    ) -> AppResult<EgressInfo> {
        let s3_upload = S3Upload {
            access_key: self.s3_access_key.clone(),
            secret: self.s3_secret.clone(),
            region: self.s3_region.clone(),
            bucket: self.s3_bucket.clone(),
            ..Default::default()
        };

        let file_output = EncodedFileOutput {
            file_type: EncodedFileType::Mp4 as i32,
            filepath: filepath.unwrap_or_else(|| format!("{room}-{{time}}.mp4")),
            output: Some(encoded_file_output::Output::S3(s3_upload)),
            ..Default::default()
        };

        let options = RoomCompositeOptions {
            layout: layout.unwrap_or_default(),
            audio_only,
            ..Default::default()
        };

        self.client
            .start_room_composite_egress(room, vec![EgressOutput::File(file_output)], options)
            .await
            .map_err(|e| AppError::LiveKit(format!("{e:?}")))
    }

    pub async fn stop(&self, egress_id: &str) -> AppResult<EgressInfo> {
        self.client
            .stop_egress(egress_id)
            .await
            .map_err(|e| AppError::LiveKit(e.to_string()))
    }

    pub async fn list_for_room(&self, room: &str) -> AppResult<Vec<EgressInfo>> {
        self.client
            .list_egress(EgressListOptions {
                filter: EgressListFilter::Room(room.to_string()),
                active: false,
                page_token: None,
            })
            .await
            .map_err(|e| AppError::LiveKit(format!("{e:?}")))
    }
}