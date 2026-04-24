use axum::{extract::State, http::StatusCode};
use promptwho_core::IngestService;
use promptwho_protocol::{IngestEventsRequest, IngestEventsResponse, IngestOpencodeEventsRequest};
use tracing::instrument;

use crate::extractors::MsgPackOrJson;
use crate::{errors::ServerError, extractors::ServerMsg, state::AppState};

#[utoipa::path(
    post,
    path = "/v1/events",
    request_body(content = IngestEventsRequest, content_type = "application/msgpack"),
    responses(
        (status = 202, description = "Accepted events", body = IngestEventsResponse, content_type = "application/msgpack"),
        (status = 400, description = "Invalid request", body = promptwho_protocol::ErrorResponse),
        (status = 500, description = "Server failure", body = promptwho_protocol::ErrorResponse)
    )
)]
#[instrument]
pub async fn ingest_events(
    State(state): State<AppState>,
    ServerMsg(body): ServerMsg<IngestEventsRequest>,
) -> Result<(StatusCode, ServerMsg<IngestEventsResponse>), ServerError> {
    let ingest = IngestService::new(state.store.as_ref());
    let response_format = body.response_format();
    let mut accepted = 0usize;
    for event in &body.events {
        let outcome = ingest
            .ingest_protocol_event(event.clone())
            .await
            .map_err(|error| ServerError::store(error, response_format))?;
        if outcome.inserted {
            accepted += 1;
        }
    }
    tracing::info!(
        "Ingested {} events, accepted {}",
        body.events.len(),
        accepted
    );
    Ok((
        StatusCode::ACCEPTED,
        ServerMsg(MsgPackOrJson::MsgPack(IngestEventsResponse {
            request_id: body.request_id,
            accepted,
            rejected: 0,
        })),
    ))
}

#[utoipa::path(
    post,
    path = "/v1/opencode/events",
    request_body(content = IngestOpencodeEventsRequest, content_type = "application/msgpack"),
    responses(
        (status = 202, description = "Accepted opencode events", body = IngestEventsResponse, content_type = "application/msgpack"),
        (status = 400, description = "Invalid request", body = promptwho_protocol::ErrorResponse),
        (status = 500, description = "Server failure", body = promptwho_protocol::ErrorResponse)
    )
)]
#[instrument]
pub async fn ingest_opencode_events(
    State(state): State<AppState>,
    ServerMsg(body): ServerMsg<IngestOpencodeEventsRequest>,
) -> Result<(StatusCode, ServerMsg<IngestEventsResponse>), ServerError> {
    let ingest = IngestService::new(state.store.as_ref());
    let response_format = body.response_format();
    let mut accepted = 0usize;

    for event in &body.events {
        let outcome = ingest
            .ingest_opencode_event(event.clone())
            .await
            .map_err(|error| ServerError::store(error, response_format))?;

        if outcome.is_some_and(|outcome| outcome.inserted) {
            accepted += 1;
        }
    }

    tracing::info!(
        "Ingested {} opencode events, accepted {}",
        body.events.len(),
        accepted
    );

    Ok((
        StatusCode::ACCEPTED,
        ServerMsg(MsgPackOrJson::MsgPack(IngestEventsResponse {
            request_id: body.request_id,
            accepted,
            rejected: 0,
        })),
    ))
}
