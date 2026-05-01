use axum::{extract::State, http::StatusCode};
use promptwho_core::ingest::service::Ingest;
use promptwho_core::{IngestError, IngestService};
use promptwho_protocol::{IngestEventsRequest, IngestEventsResponse};
use tracing::instrument;

use crate::extractors::MsgPackOrJson;
use crate::{errors::ServerError, extractors::ServerMsg, state::AppState};

async fn ingest_with<I>(
    ingest: I,
    events: &Vec<I::IngestEventEnvelope>,
) -> Result<usize, IngestError>
where
    I: Ingest + Sync,
{
    let mut accepted = 0usize;
    for event in events {
        let outcome = ingest.ingest_protocol_event(event).await?;
        if outcome.inserted {
            accepted += 1;
        }
    }

    Ok(accepted)
}

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
#[instrument(skip(state, body))]
pub async fn ingest_events(
    State(state): State<AppState>,
    ServerMsg(body): ServerMsg<IngestEventsRequest>,
) -> Result<(StatusCode, ServerMsg<IngestEventsResponse>), ServerError> {
    let response_format = body.response_format();
    let event_count = body.events.len();
    let request_id = body.request_id;
    let actions = body.as_ref().actions();
    let accepted = ingest_with(IngestService::new(state.store.as_ref()), &body.events)
        .await
        .map_err(|e| ServerError::store(e, response_format))?;
    if accepted == 0 {
        tracing::warn!("No events were ingested from the request: {actions:?}");
    } else if accepted < event_count {
        tracing::warn!(
            "Only ingested {} out of {} events from the request",
            accepted,
            event_count
        );
    } else if event_count == 1 && accepted == 1 {
        tracing::info!(
            "Ingested 1 event: {}",
            actions.first().map(|a| a.as_str()).unwrap_or("unknown")
        );
    } else {
        tracing::info!("Ingested {} / {} events", accepted, event_count);
    }

    Ok((
        StatusCode::ACCEPTED,
        ServerMsg(MsgPackOrJson::MsgPack(IngestEventsResponse {
            request_id,
            accepted,
            rejected: event_count - accepted,
        })),
    ))
}
