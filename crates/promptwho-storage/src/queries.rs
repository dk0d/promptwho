use crate::models::ProjectId;
use promptwho_protocol::TimestampUtc;

#[derive(Debug, Clone, Default)]
pub struct EventQuery {
    pub project_id: Option<ProjectId>,
    pub session_id: Option<String>,
    pub action: Option<String>,
    pub occurred_after: Option<TimestampUtc>,
    pub occurred_before: Option<TimestampUtc>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct SessionQuery {
    pub project_id: Option<ProjectId>,
    pub started_after: Option<TimestampUtc>,
    pub started_before: Option<TimestampUtc>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct CommitQuery {
    pub committed_after: Option<TimestampUtc>,
    pub committed_before: Option<TimestampUtc>,
    pub limit: Option<u32>,
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
