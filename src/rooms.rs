use livekit_api::services::room::{CreateRoomOptions, RoomClient};
use livekit_protocol::{Room, ParticipantInfo, TrackInfo};
use crate::error::{AppError, AppResult};

pub struct RoomService {
    client: RoomClient,
}

impl RoomService {
    pub fn new(url: &str) -> AppResult<Self> {
        let client = RoomClient::new(url)
            .map_err(|e| AppError::LiveKit(e.to_string()))?;
        Ok(Self { client })
    }

    pub async fn create(&self, name: &str, max_participants: u32) -> AppResult<Room> {
        let opts = CreateRoomOptions {
            max_participants,
            empty_timeout: 600,
            ..Default::default()
        };
        self.client.create_room(name, opts).await
            .map_err(|e| AppError::LiveKit(e.to_string()))
    }

    pub async fn list(&self) -> AppResult<Vec<Room>> {
        self.client.list_rooms(vec![]).await
            .map_err(|e| AppError::LiveKit(e.to_string()))
    }

    pub async fn delete(&self, name: &str) -> AppResult<()> {
        self.client.delete_room(name).await
            .map_err(|e| AppError::LiveKit(e.to_string()))
    }

    pub async fn participants(&self, room: &str) -> AppResult<Vec<ParticipantInfo>> {
        self.client.list_participants(room).await
            .map_err(|e| AppError::LiveKit(e.to_string()))
    }

    pub async fn kick(&self, room: &str, identity: &str) -> AppResult<()> {
        self.client.remove_participant(room, identity).await
            .map_err(|e| AppError::LiveKit(e.to_string()))
    }

    pub async fn mute_participant(
        &self,
        room: &str,
        identity: &str,
        track_sid: &str,
        muted: bool,
    ) -> AppResult<()> {
        let _: TrackInfo = self.client
            .mute_published_track(room, identity, track_sid, muted)
            .await
            .map_err(|e| AppError::LiveKit(e.to_string()))?;
        Ok(())
  }
}