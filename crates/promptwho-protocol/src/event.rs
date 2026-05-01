use std::fmt::Display;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::TimestampUtc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolVersion {
    V1,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ProjectRef {
    pub id: String,
    pub root: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SessionRef {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PluginSource {
    pub plugin: String,
    pub plugin_version: String,
    pub runtime: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum EventAction {
    #[serde(rename = "session.started")]
    SessionStarted,
    #[serde(rename = "session.ended")]
    SessionEnded,
    #[serde(rename = "message.added")]
    MessageAdded,
    #[serde(rename = "tool.called")]
    ToolCalled,
    #[serde(rename = "tool.result")]
    ToolResult,
    #[serde(rename = "git.snapshot")]
    GitSnapshot,
    #[serde(rename = "trace.linked")]
    TraceLinked,
}

impl Display for EventAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        serde_json::to_string(self)
            .map_err(|_| std::fmt::Error)?
            .trim_matches('"')
            .fmt(f)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "action", content = "payload")]
pub enum EventPayload {
    #[serde(rename = "session.started")]
    SessionStarted(SessionStartedPayload),
    #[serde(rename = "session.ended")]
    SessionEnded(SessionEndedPayload),
    #[serde(rename = "message.added")]
    MessageAdded(MessageAddedPayload),
    #[serde(rename = "tool.called")]
    ToolCalled(ToolCalledPayload),
    #[serde(rename = "tool.result")]
    ToolResult(ToolResultPayload),
    #[serde(rename = "git.snapshot")]
    GitSnapshot(GitSnapshotPayload),
    #[serde(rename = "trace.linked")]
    TraceLinked(TraceLinkedPayload),
}
impl EventPayload {
    pub fn action(&self) -> EventAction {
        match self {
            EventPayload::SessionStarted(_) => EventAction::SessionStarted,
            EventPayload::SessionEnded(_) => EventAction::SessionEnded,
            EventPayload::MessageAdded(_) => EventAction::MessageAdded,
            EventPayload::ToolCalled(_) => EventAction::ToolCalled,
            EventPayload::ToolResult(_) => EventAction::ToolResult,
            EventPayload::GitSnapshot(_) => EventAction::GitSnapshot,
            EventPayload::TraceLinked(_) => EventAction::TraceLinked,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EventEnvelope {
    pub id: Uuid,
    pub version: ProtocolVersion,
    #[schema(value_type = String, format = "date-time")]
    pub occurred_at: TimestampUtc,
    pub project: ProjectRef,
    pub session: Option<SessionRef>,
    pub source: PluginSource,
    #[serde(flatten)]
    pub payload: EventPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SessionStartedPayload {
    pub provider: String,
    pub model: String,
    pub branch: Option<String>,
    pub head_commit: Option<String>,
    #[serde(default)]
    #[schema(value_type = Object)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SessionEndedPayload {
    pub reason: Option<String>,
    #[serde(default)]
    #[schema(value_type = Object)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MessageAddedPayload {
    pub message_id: String,
    pub role: String,
    pub content: String,
    pub token_count: Option<u32>,
    #[serde(default)]
    #[schema(value_type = Object)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ToolCalledPayload {
    pub tool_call_id: String,
    pub tool_name: String,
    #[schema(value_type = Object)]
    pub input: Value,
    #[serde(default)]
    #[schema(value_type = Object)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ToolResultPayload {
    pub tool_call_id: String,
    pub success: bool,
    #[schema(value_type = Object)]
    pub output: Value,
    #[serde(default)]
    #[schema(value_type = Object)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GitSnapshotPayload {
    pub branch: Option<String>,
    pub head_commit: Option<String>,
    pub dirty: bool,
    #[serde(default)]
    pub staged_files: Vec<String>,
    #[serde(default)]
    pub unstaged_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TraceLinkedPayload {
    pub trace_id: String,
    pub span_id: Option<String>,
    pub file_path: Option<String>,
    pub symbol: Option<String>,
    #[serde(default)]
    #[schema(value_type = Object)]
    pub metadata: Value,
}
