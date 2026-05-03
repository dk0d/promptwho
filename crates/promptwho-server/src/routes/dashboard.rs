use axum::{
    Json,
    extract::{Path, Query, State},
};
use promptwho_protocol::TimestampUtc;
use promptwho_storage::{
    ConversationStore, EventQuery, EventStore, SearchStore, SessionQuery, TextSearchQuery,
};
use serde::{Deserialize, Serialize};

use crate::{errors::ServerError, state::AppState};

#[derive(Debug, Deserialize)]
pub struct SessionListParams {
    pub project_id: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct EventListParams {
    pub project_id: Option<String>,
    pub session_id: Option<String>,
    pub action: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub q: Option<String>,
    pub project_id: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct DashboardProject {
    pub id: String,
    pub name: Option<String>,
    pub root: String,
    pub created_at: TimestampUtc,
}

#[derive(Debug, Serialize)]
pub struct DashboardSession {
    pub id: String,
    pub project_id: String,
    pub provider: String,
    pub model: String,
    pub started_at: TimestampUtc,
    pub ended_at: Option<TimestampUtc>,
}

#[derive(Debug, Serialize)]
pub struct DashboardMessage {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub token_count: Option<u32>,
    pub created_at: TimestampUtc,
}

#[derive(Debug, Serialize)]
pub struct DashboardEvent {
    pub id: uuid::Uuid,
    pub project_id: String,
    pub session_id: Option<String>,
    pub occurred_at: TimestampUtc,
    pub action: String,
}

#[derive(Debug, Serialize)]
pub struct DashboardSearchHit {
    pub kind: String,
    pub id: String,
    pub title: String,
    pub snippet: Option<String>,
    pub score: f32,
}

#[utoipa::path(get, path = "/v1/projects", responses((status = 200, description = "List projects")))]
pub async fn list_projects(
    State(state): State<AppState>,
) -> Result<Json<Vec<DashboardProject>>, ServerError> {
    let projects = state
        .store
        .list_projects(None)
        .await
        .map_err(ServerError::query)?
        .into_iter()
        .map(|project| DashboardProject {
            id: project.id,
            name: project.name,
            root: project.root,
            created_at: project.created_at,
        })
        .collect();

    Ok(Json(projects))
}

#[utoipa::path(get, path = "/v1/sessions", params(("project_id" = Option<String>, Query), ("limit" = Option<u32>, Query)), responses((status = 200, description = "List sessions")))]
pub async fn list_sessions(
    State(state): State<AppState>,
    Query(params): Query<SessionListParams>,
) -> Result<Json<Vec<DashboardSession>>, ServerError> {
    let sessions: Vec<DashboardSession> = state
        .store
        .list_sessions(Some(SessionQuery {
            project_id: params.project_id,
            limit: Some(params.limit.unwrap_or(50).min(200)),
            ..Default::default()
        }))
        .await
        .map_err(ServerError::query)?
        .into_iter()
        .map(|session| DashboardSession {
            id: session.id,
            project_id: session.project_id,
            provider: session.provider,
            model: session.model,
            started_at: session.started_at,
            ended_at: session.ended_at,
        })
        .collect();

    Ok(Json(sessions))
}

#[utoipa::path(get, path = "/v1/sessions/{session_id}/messages", params(("session_id" = String, Path)), responses((status = 200, description = "List messages for a session")))]
pub async fn list_messages(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Json<Vec<DashboardMessage>>, ServerError> {
    let messages = state
        .store
        .list_messages(session_id, None)
        .await
        .map_err(ServerError::query)?
        .into_iter()
        .map(|message| DashboardMessage {
            id: message.id,
            session_id: message.session_id,
            role: message.role,
            content: message.content,
            token_count: message.token_count,
            created_at: message.created_at,
        })
        .collect();

    Ok(Json(messages))
}

#[utoipa::path(get, path = "/v1/events/query", params(("project_id" = Option<String>, Query), ("session_id" = Option<String>, Query), ("action" = Option<String>, Query), ("limit" = Option<u32>, Query)), responses((status = 200, description = "List events")))]
pub async fn list_events(
    State(state): State<AppState>,
    Query(params): Query<EventListParams>,
) -> Result<Json<Vec<DashboardEvent>>, ServerError> {
    let events = state
        .store
        .list_events(Some(EventQuery {
            project_id: params.project_id,
            session_id: params.session_id,
            action: params.action,
            limit: Some(params.limit.unwrap_or(100).min(500)),
            ..Default::default()
        }))
        .await
        .map_err(ServerError::query)?
        .into_iter()
        .map(|event| DashboardEvent {
            id: event.id,
            project_id: event.project_id,
            session_id: event.session_id,
            occurred_at: event.occurred_at,
            action: event.action,
        })
        .collect();

    Ok(Json(events))
}

#[utoipa::path(get, path = "/v1/search", params(("q" = Option<String>, Query), ("project_id" = Option<String>, Query), ("limit" = Option<u32>, Query)), responses((status = 200, description = "Search across promptwho data")))]
pub async fn search(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<Vec<DashboardSearchHit>>, ServerError> {
    let query = params.q.unwrap_or_default();
    if query.trim().is_empty() {
        return Ok(Json(Vec::new()));
    }

    let hits = state
        .store
        .search_text(TextSearchQuery {
            text: query,
            project_id: params.project_id,
            limit: params.limit.unwrap_or(25).min(100),
        })
        .await
        .map_err(ServerError::query)?
        .hits
        .into_iter()
        .map(|hit| DashboardSearchHit {
            kind: hit.kind,
            id: hit.id,
            title: hit.title,
            snippet: hit.snippet,
            score: hit.score,
        })
        .collect();

    Ok(Json(hits))
}
