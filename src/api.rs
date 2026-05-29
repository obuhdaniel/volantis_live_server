use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::{delete, get, post},
    Json, Router,
    body::Bytes,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use crate::{
    auth::TokenService,
    rooms::RoomService,
    error::AppResult,
    webhooks,
};

#[derive(Clone)]
pub struct AppState {
    pub tokens: Arc<TokenService>,
    pub rooms:  Arc<RoomService>,
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
fn default_true() -> bool { true }

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
fn default_participants() -> u32 { 50 }

// ── Router ────────────────────────────────────────────────────────────────

pub fn router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/token",                       post(issue_token))
        .route("/rooms",                       get(list_rooms).post(create_room))
        .route("/rooms/:name",                 delete(delete_room))
        .route("/rooms/:name/participants",    get(list_participants))
        .route("/rooms/:name/kick/:identity",  delete(kick_participant))
        .route("/webhook",                     post(webhook_handler))
        .route("/health",                      get(health))
        .layer(cors)
        .with_state(state)
}

// ── Handlers ──────────────────────────────────────────────────────────────

async fn health() -> &'static str { "ok" }

async fn issue_token(
    State(s): State<AppState>,
    Json(req): Json<TokenRequest>,
) -> AppResult<Json<TokenResponse>> {
    let token = s.tokens.create_join_token(
        &req.identity, &req.room,
        req.can_publish, req.can_subscribe, req.is_admin,
    )?;
    Ok(Json(TokenResponse { token, url: s.livekit_url.clone() }))
}

async fn create_room(
    State(s): State<AppState>,
    Json(req): Json<CreateRoomRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let room = s.rooms.create(&req.name, req.max_participants).await?;
    Ok(Json(serde_json::json!({
        "name": room.name,
        "sid": room.sid,
        "num_participants": room.num_participants,
    })))
}

async fn list_rooms(State(s): State<AppState>) -> AppResult<Json<serde_json::Value>> {
    let rooms: Vec<_> = s.rooms.list().await?;
    let list: Vec<_> = rooms.iter().map(|r| serde_json::json!({
        "name": r.name,
        "sid": r.sid,
        "num_participants": r.num_participants,
        "creation_time": r.creation_time,
    })).collect();
    Ok(Json(serde_json::json!({ "rooms": list })))
}

async fn delete_room(
    State(s): State<AppState>,
    Path(name): Path<String>,
) -> AppResult<StatusCode> {
    s.rooms.delete(&name).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_participants(
    State(s): State<AppState>,
    Path(room): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let participants: Vec<_> = s.rooms.participants(&room).await?;
    let list: Vec<_> = participants.iter().map(|p| serde_json::json!({
        "identity": p.identity,
        "sid": p.sid,
        "name": p.name,
        "joined_at": p.joined_at,
    })).collect();
    Ok(Json(serde_json::json!({ "participants": list })))
}

async fn kick_participant(
    State(s): State<AppState>,
    Path((room, identity)): Path<(String, String)>,
) -> AppResult<StatusCode> {
    s.rooms.kick(&room, &identity).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn webhook_handler(
    State(s): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> StatusCode {
    if let Some(event) = webhooks::verify_and_parse(&s.api_key, &s.api_secret, &headers, &body) {
        tokio::spawn(webhooks::handle_event(event));
        StatusCode::OK
    } else {
        StatusCode::UNAUTHORIZED
    }
}