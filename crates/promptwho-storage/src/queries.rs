use crate::models::ProjectId;
use promptwho_protocol::TimestampUtc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum Pagination {
    Cursor(String),
    Offset(u32),
}

#[derive(Debug, Clone, Default)]
pub struct ProjectQuery {
    pub id: Option<ProjectId>,
    pub root: Option<String>,
    pub name: Option<String>,
    pub repository_fingerprint: Option<String>,
    pub created_after: Option<TimestampUtc>,
    pub created_before: Option<TimestampUtc>,
    pub limit: Option<u32>,
    pub pagination: Option<Pagination>,
}

#[derive(Debug, Clone, Default)]
pub struct EventQuery {
    pub project_id: Option<ProjectId>,
    pub session_id: Option<String>,
    pub action: Option<String>,
    pub occurred_after: Option<TimestampUtc>,
    pub occurred_before: Option<TimestampUtc>,
    pub limit: Option<u32>,
    pub pagination: Option<Pagination>,
}

#[derive(Debug, Clone, Default)]
pub struct SessionQuery {
    pub id: Option<String>,
    pub project_id: Option<ProjectId>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub started_after: Option<TimestampUtc>,
    pub started_before: Option<TimestampUtc>,
    pub limit: Option<u32>,
    pub pagination: Option<Pagination>,
}

#[derive(Debug, Clone, Default)]
pub struct MessageQuery {
    pub role: Option<String>,
    pub created_after: Option<TimestampUtc>,
    pub created_before: Option<TimestampUtc>,
    pub limit: Option<u32>,
    pub pagination: Option<Pagination>,
}

#[derive(Debug, Clone, Default)]
pub struct CommitQuery {
    pub oid: Option<String>,
    pub author_email: Option<String>,
    pub committed_after: Option<TimestampUtc>,
    pub committed_before: Option<TimestampUtc>,
    pub limit: Option<u32>,
    pub pagination: Option<Pagination>,
}

#[derive(Debug, Clone, Default)]
pub struct FileHistoryQuery {
    pub commit_oid: Option<String>,
    pub committed_after: Option<TimestampUtc>,
    pub committed_before: Option<TimestampUtc>,
    pub limit: Option<u32>,
    pub pagination: Option<Pagination>,
}

#[derive(Debug, Clone, Default)]
pub struct CommitHunkQuery {
    pub file_path: Option<String>,
    pub hunk_header: Option<String>,
    pub limit: Option<u32>,
    pub pagination: Option<Pagination>,
}

#[derive(Debug, Clone, Default)]
pub struct TraceLinkQuery {
    pub project_id: Option<ProjectId>,
    pub file_path: Option<String>,
    pub trace_id: Option<String>,
    pub symbol: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct TraceFrameQuery {
    pub span_id: Option<String>,
    pub parent_span_id: Option<String>,
    pub project_id: Option<ProjectId>,
    pub session_id: Option<String>,
    pub name: Option<String>,
    pub started_after: Option<TimestampUtc>,
    pub started_before: Option<TimestampUtc>,
    pub limit: Option<u32>,
    pub pagination: Option<Pagination>,
}

#[derive(Debug, Clone, Default)]
pub struct SessionChangeHunkQuery {
    pub project_id: Option<ProjectId>,
    pub file_path: Option<String>,
    pub base_commit_oid: Option<String>,
    pub symbol: Option<String>,
    pub captured_after: Option<TimestampUtc>,
    pub captured_before: Option<TimestampUtc>,
    pub limit: Option<u32>,
    pub pagination: Option<Pagination>,
}

#[derive(Debug, Clone, Default)]
pub struct CommitAttributionQuery {
    pub commit_oid: Option<String>,
    pub session_id: Option<String>,
    pub minimum_score: Option<f32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct TextSearchQuery {
    pub text: String,
    pub project_id: Option<ProjectId>,
    pub limit: u32,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub kind: String,
    pub id: String,
    pub title: String,
    pub snippet: Option<String>,
    pub score: f32,
}

#[derive(Debug, Clone, Default)]
pub struct SearchResults {
    pub hits: Vec<SearchResult>,
}

#[derive(Debug, Clone)]
pub struct VectorSearchQuery {
    pub embedding: Vec<f32>,
    pub project_id: Option<ProjectId>,
    pub limit: u32,
}
