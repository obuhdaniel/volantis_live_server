use livekit_api::access_token::{AccessToken, VideoGrants};
use std::time::Duration;
use crate::error::{AppError, AppResult};

#[derive(Clone)]
pub struct TokenService {
    pub api_key: String,
    pub api_secret: String,
}

impl TokenService {
    pub fn new(api_key: &str, api_secret: &str) -> Self {
        Self {
            api_key: api_key.to_owned(),
            api_secret: api_secret.to_owned(),
        }
    }

    pub fn create_join_token(
        &self,
        identity: &str,
        room: &str,
        can_publish: bool,
        can_subscribe: bool,
        is_admin: bool,
    ) -> AppResult<String> {
        let grants = VideoGrants {
            room: room.to_owned(),
            room_join: true,
            room_admin: is_admin,
            can_publish,
            can_subscribe,
            can_publish_data: true,
            ..Default::default()
        };

        AccessToken::with_api_key(&self.api_key, &self.api_secret)
            .with_identity(identity)
            .with_name(identity)
            .with_ttl(Duration::from_secs(3600 * 4)) // 4 hours
            .with_grants(grants)
            .to_jwt()
            .map_err(|e| AppError::LiveKit(e.to_string()))
    }
}