use axum::{http::HeaderMap, body::Bytes};
use livekit_api::webhooks::WebhookReceiver;
use livekit_api::access_token::TokenVerifier;
use livekit_protocol::WebhookEvent;
use tracing::{info, warn};

pub fn verify_and_parse(
    api_key: &str,
    api_secret: &str,
    headers: &HeaderMap,
    body: &Bytes,
) -> Option<WebhookEvent> {
    let verifier = TokenVerifier::with_api_key(api_key, api_secret);
    let receiver = WebhookReceiver::new(verifier);
    let auth = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let body_str = std::str::from_utf8(body).ok()?;

    match receiver.receive(body_str, auth) {
        Ok(event) => Some(event),
        Err(e) => {
            warn!("Webhook verification failed: {}", e);
            None
        }
    }
}

pub async fn handle_event(event: WebhookEvent) {
    match event.event.as_str() {
        "room_started" => {
            if let Some(room) = &event.room {
                info!("Room started: {}", room.name);
            }
        }
        "room_finished" => {
            if let Some(room) = &event.room {
                info!("Room finished: {}", room.name);
            }
        }
        "participant_joined" => {
            if let (Some(room), Some(p)) = (&event.room, &event.participant) {
                info!("Participant joined: {} in {}", p.identity, room.name);
            }
        }
        "participant_left" => {
            if let (Some(room), Some(p)) = (&event.room, &event.participant) {
                info!("Participant left: {} in {}", p.identity, room.name);
            }
        }
        "track_published" => {
            if let Some(p) = &event.participant {
                info!("Track published by: {}", p.identity);
            }
        }
        "egress_started" => {
            info!("Recording started: {:?}", event.egress_info);
        }
        "egress_ended" => {
            info!("Recording ended: {:?}", event.egress_info);
        }
        other => {
            info!("Unhandled webhook event: {}", other);
        }
    }
}