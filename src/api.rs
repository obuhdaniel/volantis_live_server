use crate::{
    agents::AgentService, auth::TokenService, egress::EgressService, error::AppResult,
    rooms::RoomService, webhooks,
};
use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
pub struct AppState {
    pub tokens: Arc<TokenService>,
    pub rooms: Arc<RoomService>,
    pub egress: Arc<EgressService>,
    pub agents: Arc<AgentService>,
    pub agent_name: String,
    pub api_key: String,
    pub api_secret: String,
    pub livekit_url: String,
}

// ── Request / Response types ──────────────────────────────────────────────

#[derive(Deserialize)]
pub struct TokenRequest {
    pub identity: String,
    pub room: String,
    #[serde(default = "default_true")]
    pub can_publish: bool,
    #[serde(default = "default_true")]
    pub can_subscribe: bool,
    #[serde(default)]
    pub is_admin: bool,
}
fn default_true() -> bool {
    true
}

#[derive(Serialize)]
pub struct TokenResponse {
    pub token: String,
    pub url: String,
}

#[derive(Deserialize)]
pub struct CreateRoomRequest {
    pub name: String,
    #[serde(default = "default_participants")]
    pub max_participants: u32,
}

#[derive(Deserialize)]
pub struct StartRecordingRequest {
    pub filepath: Option<String>,
    pub layout: Option<String>,
    #[serde(default)]
    pub audio_only: bool,
}

#[derive(Deserialize)]
pub struct MuteRequest {
    pub track_sid: String,
    #[serde(default = "default_true")]
    pub muted: bool,
}

#[derive(Deserialize)]
pub struct MuteAllRequest {
    #[serde(default = "default_true")]
    pub muted: bool,
    #[serde(default = "default_true")]
    pub audio_only: bool,
}

#[derive(Deserialize)]
pub struct DispatchAgentRequest {
    pub agent_name: Option<String>,
    #[serde(default)]
    pub metadata: Option<String>,
}

fn default_participants() -> u32 {
    50
}

// ── Router ────────────────────────────────────────────────────────────────

pub fn router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/token", post(issue_token))
        .route("/rooms", get(list_rooms).post(create_room))
        .route("/rooms/{name}", delete(delete_room))
        .route("/rooms/{name}/participants", get(list_participants))
        .route("/rooms/{name}/kick/{identity}", delete(kick_participant))
        .route("/webhook", post(webhook_handler))
        .route("/health", get(health))
        .route("/rooms/{name}/recording/start", post(start_recording))
        .route("/rooms/{name}/egress", get(get_room_egress))
        .route("/egress/{id}/stop", post(stop_recording))
        .route("/rooms/{name}/participants/{identity}/mute", post(mute_one))
        .route("/rooms/{name}/mute-all", post(mute_all))
        .route(
            "/rooms/{name}/agent-dispatch",
            get(list_agent_dispatches).post(create_agent_dispatch),
        )
        .route(
            "/rooms/{name}/agent-dispatch/{dispatch_id}",
            delete(delete_agent_dispatch),
        )
        .layer(cors)
        .with_state(state)
}

// ── Handlers ──────────────────────────────────────────────────────────────

async fn health() -> &'static str {
    "ok"
}

async fn issue_token(
    State(s): State<AppState>,
    Json(req): Json<TokenRequest>,
) -> AppResult<Json<TokenResponse>> {
    let token = s.tokens.create_join_token(
        &req.identity,
        &req.room,
        req.can_publish,
        req.can_subscribe,
        req.is_admin,
    )?;
    Ok(Json(TokenResponse {
        token,
        url: s.livekit_url.clone(),
    }))
}

async fn create_room(
    State(s): State<AppState>,
    Json(req): Json<CreateRoomRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let room = s.rooms.create(&req.name).await?;
    Ok(Json(serde_json::json!({
        "name": room.name,
        "sid": room.sid,
        "num_participants": room.num_participants,
    })))
}

async fn list_rooms(State(s): State<AppState>) -> AppResult<Json<serde_json::Value>> {
    let rooms: Vec<_> = s.rooms.list().await?;
    let list: Vec<_> = rooms
        .iter()
        .map(|r| {
            serde_json::json!({
                "name": r.name,
                "sid": r.sid,
                "num_participants": r.num_participants,
                "creation_time": r.creation_time,
            })
        })
        .collect();
    Ok(Json(serde_json::json!({ "rooms": list })))
}

async fn delete_room(State(s): State<AppState>, Path(name): Path<String>) -> AppResult<StatusCode> {
    s.rooms.delete(&name).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_participants(
    State(s): State<AppState>,
    Path(room): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let participants: Vec<_> = s.rooms.participants(&room).await?;
    let list: Vec<_> = participants
        .iter()
        .map(|p| {
            serde_json::json!({
                "identity": p.identity,
                "sid": p.sid,
                "name": p.name,
                "joined_at": p.joined_at,
            })
        })
        .collect();
    Ok(Json(serde_json::json!({ "participants": list })))
}

async fn kick_participant(
    State(s): State<AppState>,
    Path((room, identity)): Path<(String, String)>,
) -> AppResult<StatusCode> {
    s.rooms.kick(&room, &identity).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn webhook_handler(State(s): State<AppState>, headers: HeaderMap, body: Bytes) -> StatusCode {
    if let Some(event) = webhooks::verify_and_parse(&s.api_key, &s.api_secret, &headers, &body) {
        tokio::spawn(webhooks::handle_event(event));
        StatusCode::OK
    } else {
        StatusCode::UNAUTHORIZED
    }
}


async fn start_recording(
    State(s): State<AppState>,
    Path(room): Path<String>,
    Json(req): Json<StartRecordingRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let info = s.egress
        .start_room_recording(&room, req.filepath, req.layout, req.audio_only)
        .await?;
    Ok(Json(serde_json::json!({
        "egress_id": info.egress_id,
        "status": info.status,
        "room_name": info.room_name,
        "started_at": info.started_at,
        "ended_at": info.ended_at,
        "file_results": info.file_results,
        "error": info.error,
    })))
}

async fn stop_recording(
    State(s): State<AppState>,
    Path(egress_id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let info = s.egress.stop(&egress_id).await?;
    Ok(Json(serde_json::json!({
        "egress_id": info.egress_id,
        "status": info.status,
        "started_at": info.started_at,
        "ended_at": info.ended_at,
        "file_results": info.file_results,
        "error": info.error,
    })))
}

fn egress_status_label(status: i32) -> &'static str {
    use livekit_protocol::EgressStatus as S;
    match S::try_from(status) {
        Ok(S::EgressStarting) => "starting",
        Ok(S::EgressActive) | Ok(S::EgressEnding) => "in_progress",
        Ok(S::EgressComplete) => "finished",
        Ok(S::EgressFailed) => "failed",
        Ok(S::EgressAborted) => "aborted",
        Ok(S::EgressLimitReached) => "limit_reached",
        Err(_) => "unknown",
    }
}
async fn get_room_egress(
    State(s): State<AppState>,
    Path(room): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let egresses = s.egress.list_for_room(&room).await?;

    // pick the most relevant egress (active > latest)
    let maybe = egresses
        .iter()
        .find(|e| e.status == livekit_protocol::EgressStatus::EgressActive as i32)
        .or_else(|| egresses.first());

    if let Some(info) = maybe {
       

        return Ok(Json(serde_json::json!({
            "active": info.status == livekit_protocol::EgressStatus::EgressActive as i32,
            "egress_id": info.egress_id,
            "status": info.status,
            "status_label": egress_status_label(info.status),
            "room_name": info.room_name,
            "started_at": info.started_at,
            "ended_at": info.ended_at,
            "error": info.error,
            "file_results": info.file_results,
        })));
    }

    // fallback (no egress found)
    Ok(Json(serde_json::json!({
        "active": false,
        "egress_id": null,
        "status": null,
        "status_label": null,
        "room_name": room,
        "started_at": null,
        "ended_at": null,
        "error": null,
        "file_results": [],
    })))
}

async fn mute_one(
    State(s): State<AppState>,
    Path((room, identity)): Path<(String, String)>,
    Json(req): Json<MuteRequest>,
) -> AppResult<StatusCode> {
    s.rooms.mute_participant(&room, &identity, &req.track_sid, req.muted).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn mute_all(
    State(s): State<AppState>,
    Path(room): Path<String>,
    Json(req): Json<MuteAllRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let count = s.rooms.mute_all(&room, req.muted, req.audio_only).await?;
    Ok(Json(serde_json::json!({ "muted_tracks": count })))
}

async fn create_agent_dispatch(
    State(s): State<AppState>,
    Path(room): Path<String>,
    Json(req): Json<DispatchAgentRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let agent_name = req.agent_name.unwrap_or_else(|| s.agent_name.clone());
    let dispatch = s.agents.dispatch(&room, &agent_name, req.metadata).await?;
    Ok(Json(serde_json::json!({
        "id": dispatch.id,
        "agent_name": dispatch.agent_name,
        "room": dispatch.room,
        "metadata": dispatch.metadata,
    })))
}

async fn list_agent_dispatches(
    State(s): State<AppState>,
    Path(room): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let dispatches: Vec<_> = s.agents.list(&room).await?;
    let list: Vec<_> = dispatches
        .iter()
        .map(|d| {
            serde_json::json!({
                "id": d.id,
                "agent_name": d.agent_name,
                "room": d.room,
                "metadata": d.metadata,
            })
        })
        .collect();
    Ok(Json(serde_json::json!({ "agent_dispatches": list })))
}

async fn delete_agent_dispatch(
    State(s): State<AppState>,
    Path((room, dispatch_id)): Path<(String, String)>,
) -> AppResult<StatusCode> {
    s.agents.delete(&dispatch_id, &room).await?;
    Ok(StatusCode::NO_CONTENT)
}

