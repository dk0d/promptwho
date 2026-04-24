use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

use super::shared::{ApiErrorData, OpencodeError, SnapshotFileDiff};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type")]
pub enum OutputFormat {
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "json_schema")]
    JsonSchema {
        schema: Value,
        retry_count: Option<i64>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserMessageSummary {
    pub title: Option<String>,
    pub body: Option<String>,
    #[serde(default)]
    pub diffs: Vec<SnapshotFileDiff>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ModelRef {
    #[serde(rename = "providerID", alias = "providerId")]
    pub provider_id: String,
    #[serde(rename = "modelID", alias = "modelId")]
    pub model_id: String,
    pub variant: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MessageTime {
    pub created: i64,
    pub completed: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TokenCache {
    pub read: i64,
    pub write: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TokenUsage {
    pub total: Option<i64>,
    pub input: i64,
    pub output: i64,
    pub reasoning: i64,
    pub cache: TokenCache,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AssistantPath {
    pub cwd: String,
    pub root: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "role")]
pub enum Message {
    #[serde(rename = "user")]
    User {
        id: String,
        #[serde(rename = "sessionID", alias = "sessionId")]
        session_id: String,
        time: MessageTime,
        format: Option<OutputFormat>,
        summary: Option<UserMessageSummary>,
        agent: String,
        model: ModelRef,
        system: Option<String>,
        tools: Option<std::collections::BTreeMap<String, bool>>,
    },
    #[serde(rename = "assistant")]
    Assistant {
        id: String,
        #[serde(rename = "sessionID", alias = "sessionId")]
        session_id: String,
        time: MessageTime,
        error: Option<OpencodeError>,
        #[serde(rename = "parentID", alias = "parentId")]
        parent_id: String,
        #[serde(rename = "modelID", alias = "modelId")]
        model_id: String,
        #[serde(rename = "providerID", alias = "providerId")]
        provider_id: String,
        mode: String,
        agent: String,
        path: AssistantPath,
        summary: Option<bool>,
        cost: f64,
        tokens: TokenUsage,
        structured: Option<Value>,
        variant: Option<String>,
        finish: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PartTimeRange {
    pub start: i64,
    pub end: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FilePartSourceText {
    pub value: String,
    pub start: i64,
    pub end: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RangePoint {
    pub line: i64,
    pub character: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Range {
    pub start: RangePoint,
    pub end: RangePoint,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type")]
pub enum FilePartSource {
    #[serde(rename = "file")]
    File {
        text: FilePartSourceText,
        path: String,
    },
    #[serde(rename = "symbol")]
    Symbol {
        text: FilePartSourceText,
        path: String,
        range: Range,
        name: String,
        kind: i64,
    },
    #[serde(rename = "resource")]
    Resource {
        text: FilePartSourceText,
        client_name: String,
        uri: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RetryTime {
    pub created: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "status")]
pub enum ToolState {
    #[serde(rename = "pending")]
    Pending { input: Value, raw: String },
    #[serde(rename = "running")]
    Running {
        input: Value,
        title: Option<String>,
        metadata: Option<Value>,
        time: PartTimeRange,
    },
    #[serde(rename = "completed")]
    Completed {
        input: Value,
        output: String,
        title: String,
        metadata: Value,
        time: PartTimeRange,
        attachments: Option<Vec<Part>>,
    },
    #[serde(rename = "error")]
    Error {
        input: Value,
        error: String,
        metadata: Option<Value>,
        time: PartTimeRange,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type")]
pub enum Part {
    #[serde(rename = "text")]
    Text {
        id: String,
        #[serde(rename = "sessionID", alias = "sessionId")]
        session_id: String,
        #[serde(rename = "messageID", alias = "messageId")]
        message_id: String,
        text: String,
        synthetic: Option<bool>,
        ignored: Option<bool>,
        time: Option<PartTimeRange>,
        metadata: Option<Value>,
    },
    #[serde(rename = "subtask")]
    Subtask {
        id: String,
        #[serde(rename = "sessionID", alias = "sessionId")]
        session_id: String,
        #[serde(rename = "messageID", alias = "messageId")]
        message_id: String,
        prompt: String,
        description: String,
        agent: String,
        model: Option<ModelRef>,
        command: Option<String>,
    },
    #[serde(rename = "reasoning")]
    Reasoning {
        id: String,
        #[serde(rename = "sessionID", alias = "sessionId")]
        session_id: String,
        #[serde(rename = "messageID", alias = "messageId")]
        message_id: String,
        text: String,
        metadata: Option<Value>,
        time: PartTimeRange,
    },
    #[serde(rename = "file")]
    File {
        id: String,
        #[serde(rename = "sessionID", alias = "sessionId")]
        session_id: String,
        #[serde(rename = "messageID", alias = "messageId")]
        message_id: String,
        mime: String,
        filename: Option<String>,
        url: String,
        source: Option<FilePartSource>,
    },
    #[serde(rename = "tool")]
    Tool {
        id: String,
        #[serde(rename = "sessionID", alias = "sessionId")]
        session_id: String,
        #[serde(rename = "messageID", alias = "messageId")]
        message_id: String,
        #[serde(rename = "callID", alias = "callId")]
        call_id: String,
        tool: String,
        state: ToolState,
        metadata: Option<Value>,
    },
    #[serde(rename = "step-start")]
    StepStart {
        id: String,
        #[serde(rename = "sessionID", alias = "sessionId")]
        session_id: String,
        #[serde(rename = "messageID", alias = "messageId")]
        message_id: String,
        snapshot: Option<String>,
    },
    #[serde(rename = "step-finish")]
    StepFinish {
        id: String,
        #[serde(rename = "sessionID", alias = "sessionId")]
        session_id: String,
        #[serde(rename = "messageID", alias = "messageId")]
        message_id: String,
        reason: String,
        snapshot: Option<String>,
        cost: f64,
        tokens: TokenUsage,
    },
    #[serde(rename = "snapshot")]
    Snapshot {
        id: String,
        #[serde(rename = "sessionID", alias = "sessionId")]
        session_id: String,
        #[serde(rename = "messageID", alias = "messageId")]
        message_id: String,
        snapshot: String,
    },
    #[serde(rename = "patch")]
    Patch {
        id: String,
        #[serde(rename = "sessionID", alias = "sessionId")]
        session_id: String,
        #[serde(rename = "messageID", alias = "messageId")]
        message_id: String,
        hash: String,
        #[serde(default)]
        files: Vec<String>,
    },
    #[serde(rename = "agent")]
    Agent {
        id: String,
        #[serde(rename = "sessionID", alias = "sessionId")]
        session_id: String,
        #[serde(rename = "messageID", alias = "messageId")]
        message_id: String,
        name: String,
        source: Option<FilePartSourceText>,
    },
    #[serde(rename = "retry")]
    Retry {
        id: String,
        #[serde(rename = "sessionID", alias = "sessionId")]
        session_id: String,
        #[serde(rename = "messageID", alias = "messageId")]
        message_id: String,
        attempt: i64,
        error: ApiErrorData,
        time: RetryTime,
    },
    #[serde(rename = "compaction")]
    Compaction {
        id: String,
        #[serde(rename = "sessionID", alias = "sessionId")]
        session_id: String,
        #[serde(rename = "messageID", alias = "messageId")]
        message_id: String,
        auto: bool,
        overflow: Option<bool>,
        #[serde(rename = "tail_start_id", alias = "tailStartId")]
        tail_start_id: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MessageUpdatedProperties {
    #[serde(rename = "sessionID", alias = "sessionId")]
    pub session_id: String,
    pub info: Message,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MessageRemovedProperties {
    #[serde(rename = "sessionID", alias = "sessionId")]
    pub session_id: String,
    #[serde(rename = "messageID", alias = "messageId")]
    pub message_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MessagePartUpdatedProperties {
    #[serde(rename = "sessionID", alias = "sessionId")]
    pub session_id: String,
    pub part: Part,
    pub time: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MessagePartRemovedProperties {
    #[serde(rename = "sessionID", alias = "sessionId")]
    pub session_id: String,
    #[serde(rename = "messageID", alias = "messageId")]
    pub message_id: String,
    #[serde(rename = "partID", alias = "partId")]
    pub part_id: String,
}
