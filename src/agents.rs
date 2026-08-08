use crate::error::{AppError, AppResult};
use livekit_api::services::agent_dispatch::AgentDispatchClient;
use livekit_protocol::AgentDispatch;

pub struct AgentService {
    client: AgentDispatchClient,
}

impl AgentService {
    pub fn new(url: &str) -> AppResult<Self> {
        let client = AgentDispatchClient::new(url).map_err(|e| AppError::LiveKit(e.to_string()))?;
        Ok(Self { client })
    }

    pub async fn dispatch(
        &self,
        room: &str,
        agent_name: &str,
        metadata: Option<String>,
    ) -> AppResult<AgentDispatch> {
        let req = livekit_protocol::CreateAgentDispatchRequest {
            agent_name: agent_name.to_string(),
            room: room.to_string(),
            metadata: metadata.unwrap_or_default(),
            ..Default::default()
        };
        self.client
            .create_dispatch(req)
            .await
            .map_err(|e| AppError::LiveKit(format!("{e:?}")))
    }

    pub async fn list(&self, room: &str) -> AppResult<Vec<AgentDispatch>> {
        self.client
            .list_dispatch(room.to_string())
            .await
            .map_err(|e| AppError::LiveKit(format!("{e:?}")))
    }

    pub async fn delete(&self, dispatch_id: &str, room: &str) -> AppResult<AgentDispatch> {
        self.client
            .delete_dispatch(dispatch_id.to_string(), room.to_string())
            .await
            .map_err(|e| AppError::LiveKit(format!("{e:?}")))
    }
}
