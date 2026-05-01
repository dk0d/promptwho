use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use promptwho_protocol::{EventEnvelope, TimestampUtc};

pub type ProjectId = String;
pub type SessionId = String;
pub type MessageId = String;
pub type ToolCallId = String;
pub type GitOid = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEvent {
    pub id: Uuid,
    pub project_id: ProjectId,
    pub session_id: Option<SessionId>,
    pub occurred_at: TimestampUtc,
    pub action: String,
    pub envelope: EventEnvelope,
    pub ingested_at: TimestampUtc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendOutcome {
    pub inserted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendSummary {
    pub inserted: usize,
    pub skipped: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: ProjectId,
    pub root: String,
    pub name: Option<String>,
    pub created_at: TimestampUtc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub project_id: ProjectId,
    pub provider: String,
    pub model: String,
    pub branch: Option<String>,
    pub head_commit: Option<String>,
    pub started_at: TimestampUtc,
    pub ended_at: Option<TimestampUtc>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub id: SessionId,
    pub project_id: ProjectId,
    pub provider: String,
    pub model: String,
    pub started_at: TimestampUtc,
    pub ended_at: Option<TimestampUtc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub session_id: SessionId,
    pub role: String,
    pub content: String,
    pub token_count: Option<u32>,
    pub created_at: TimestampUtc,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: ToolCallId,
    pub session_id: SessionId,
    pub tool_name: String,
    pub input: Value,
    pub created_at: TimestampUtc,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_call_id: ToolCallId,
    pub success: bool,
    pub output: Value,
    pub created_at: TimestampUtc,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitSnapshot {
    pub id: Uuid,
    pub project_id: ProjectId,
    pub session_id: Option<SessionId>,
    pub branch: Option<String>,
    pub head_commit: Option<GitOid>,
    pub dirty: bool,
    pub staged_files: Vec<String>,
    pub unstaged_files: Vec<String>,
    pub created_at: TimestampUtc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitCommit {
    pub oid: GitOid,
    pub project_id: ProjectId,
    pub parent_oid: Option<GitOid>,
    pub author_name: Option<String>,
    pub author_email: Option<String>,
    pub message: String,
    pub committed_at: TimestampUtc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitCommitFile {
    pub commit_oid: GitOid,
    pub path: String,
    pub old_path: Option<String>,
    pub change_kind: String,
    pub hunk_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitCommitHunk {
    pub id: Uuid,
    pub commit_oid: GitOid,
    pub file_path: String,
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub hunk_header: Option<String>,
    pub added_line_count: u32,
    pub removed_line_count: u32,
    pub context_before_hash: Option<String>,
    pub context_after_hash: Option<String>,
    pub added_lines_fingerprint: Option<String>,
    pub removed_lines_fingerprint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitFileHistoryRow {
    pub commit_oid: GitOid,
    pub path: String,
    pub committed_at: TimestampUtc,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCodeChange {
    pub id: Uuid,
    pub session_id: SessionId,
    pub project_id: ProjectId,
    pub tool_call_id: Option<ToolCallId>,
    pub source: String,
    pub captured_at: TimestampUtc,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionChangeHunk {
    pub id: Uuid,
    pub change_id: Uuid,
    pub session_id: SessionId,
    pub project_id: ProjectId,
    pub file_path: String,
    pub base_commit_oid: Option<GitOid>,
    pub old_start: Option<u32>,
    pub old_lines: Option<u32>,
    pub new_start: Option<u32>,
    pub new_lines: Option<u32>,
    pub symbol: Option<String>,
    pub added_line_count: u32,
    pub removed_line_count: u32,
    pub added_lines_fingerprint: Option<String>,
    pub removed_lines_fingerprint: Option<String>,
    pub captured_at: TimestampUtc,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTrace {
    pub trace_id: String,
    pub project_id: ProjectId,
    pub service_name: Option<String>,
    pub root_span_name: Option<String>,
    pub started_at: TimestampUtc,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceFrame {
    pub id: Uuid,
    pub trace_id: String,
    pub span_id: Option<String>,
    pub parent_span_id: Option<String>,
    pub project_id: ProjectId,
    pub session_id: Option<SessionId>,
    pub name: Option<String>,
    pub started_at: Option<TimestampUtc>,
    pub ended_at: Option<TimestampUtc>,
    pub metadata: Value,
    pub created_at: TimestampUtc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeLocation {
    pub id: Uuid,
    pub trace_frame_id: Uuid,
    pub project_id: ProjectId,
    pub file_path: String,
    pub symbol: Option<String>,
    pub start_line: Option<u32>,
    pub end_line: Option<u32>,
    pub confidence: f32,
    pub metadata: Value,
    pub resolved_at: TimestampUtc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchAttribution {
    pub id: Uuid,
    pub project_id: ProjectId,
    pub commit_oid: GitOid,
    pub commit_hunk_id: Uuid,
    pub session_id: SessionId,
    pub session_change_hunk_id: Uuid,
    pub score: f32,
    pub algorithm_version: String,
    pub reasons: Vec<String>,
    pub created_at: TimestampUtc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitSessionSummary {
    pub id: Uuid,
    pub project_id: ProjectId,
    pub commit_oid: GitOid,
    pub session_id: SessionId,
    pub patch_count: u32,
    pub score: f32,
    pub summary: Vec<String>,
    pub created_at: TimestampUtc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingRecord {
    pub id: Uuid,
    pub project_id: ProjectId,
    pub content_type: String,
    pub content_id: String,
    pub embedding: Vec<f32>,
    pub metadata: Value,
    pub created_at: TimestampUtc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorHit {
    pub content_type: String,
    pub content_id: String,
    pub score: f32,
    pub metadata: Value,
}
