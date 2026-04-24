use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::event::EventEnvelope;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct IngestEventsRequest {
    #[serde(alias = "requestId")]
    #[serde(deserialize_with = "crate::uuid_serde::deserialize_uuid")]
    pub request_id: Uuid,
    pub events: Vec<EventEnvelope>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct IngestEventsResponse {
    pub request_id: Uuid,
    pub accepted: usize,
    pub rejected: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}
