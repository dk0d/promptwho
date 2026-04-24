use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OpencodeProjectIcon {
    pub url: Option<String>,
    pub override_value: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OpencodeProjectCommands {
    pub start: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OpencodeProjectTime {
    pub created: i64,
    pub updated: i64,
    pub initialized: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OpencodeProject {
    pub id: String,
    pub worktree: String,
    pub vcs: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    pub icon: Option<OpencodeProjectIcon>,
    pub commands: Option<OpencodeProjectCommands>,
    pub time: Option<OpencodeProjectTime>,
    #[serde(default)]
    pub sandboxes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OpencodeContext {
    pub project: OpencodeProject,
    pub directory: String,
    pub worktree: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotFileDiff {
    pub file: String,
    pub patch: String,
    pub additions: i64,
    pub deletions: i64,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProviderAuthErrorData {
    #[serde(rename = "providerID", alias = "providerId")]
    pub provider_id: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UnknownErrorData {
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StructuredOutputErrorData {
    pub message: String,
    pub retries: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ContextOverflowErrorData {
    pub message: String,
    pub response_body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ApiErrorData {
    pub message: String,
    pub status_code: Option<i64>,
    pub is_retryable: bool,
    pub response_headers: Option<std::collections::BTreeMap<String, String>>,
    pub response_body: Option<String>,
    pub metadata: Option<std::collections::BTreeMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "name", content = "data")]
pub enum OpencodeError {
    ProviderAuthError(ProviderAuthErrorData),
    UnknownError(UnknownErrorData),
    MessageOutputLengthError(Value),
    MessageAbortedError(UnknownErrorData),
    StructuredOutputError(StructuredOutputErrorData),
    ContextOverflowError(ContextOverflowErrorData),
    APIError(ApiErrorData),
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct QuestionOption {
    pub label: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct QuestionInfo {
    pub question: String,
    pub header: String,
    #[serde(default)]
    pub options: Vec<QuestionOption>,
    pub multiple: Option<bool>,
    pub custom: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct QuestionTool {
    #[serde(rename = "messageID", alias = "messageId")]
    pub message_id: String,
    #[serde(rename = "callID", alias = "callId")]
    pub call_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct QuestionRequest {
    pub id: String,
    #[serde(rename = "sessionID", alias = "sessionId")]
    pub session_id: String,
    #[serde(default)]
    pub questions: Vec<QuestionInfo>,
    pub tool: Option<QuestionTool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct QuestionReplied {
    #[serde(rename = "sessionID", alias = "sessionId")]
    pub session_id: String,
    #[serde(rename = "requestID", alias = "requestId")]
    pub request_id: String,
    #[serde(default)]
    pub answers: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct QuestionRejected {
    #[serde(rename = "sessionID", alias = "sessionId")]
    pub session_id: String,
    #[serde(rename = "requestID", alias = "requestId")]
    pub request_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TodoItem {
    pub content: String,
    pub status: String,
    pub priority: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type")]
pub enum SessionStatus {
    #[serde(rename = "idle")]
    Idle,
    #[serde(rename = "retry")]
    Retry {
        attempt: i64,
        message: String,
        next: i64,
    },
    #[serde(rename = "busy")]
    Busy,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PtyInfo {
    pub id: String,
    pub title: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    pub cwd: String,
    pub status: String,
    pub pid: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryProperties {
    pub directory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FileProperties {
    pub file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FileWatcherProperties {
    pub file: String,
    pub event: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LspClientDiagnosticsProperties {
    #[serde(rename = "serverID", alias = "serverId")]
    pub server_id: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct VersionProperties {
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MessagePartDeltaProperties {
    #[serde(rename = "sessionID", alias = "sessionId")]
    pub session_id: String,
    #[serde(rename = "messageID", alias = "messageId")]
    pub message_id: String,
    #[serde(rename = "partID", alias = "partId")]
    pub part_id: String,
    pub field: String,
    pub delta: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PermissionTool {
    #[serde(rename = "messageID", alias = "messageId")]
    pub message_id: String,
    #[serde(rename = "callID", alias = "callId")]
    pub call_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PermissionRequest {
    pub id: String,
    #[serde(rename = "sessionID", alias = "sessionId")]
    pub session_id: String,
    pub permission: String,
    #[serde(default)]
    pub patterns: Vec<String>,
    pub metadata: Value,
    #[serde(default)]
    pub always: Vec<String>,
    pub tool: Option<PermissionTool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PermissionRepliedProperties {
    #[serde(rename = "sessionID", alias = "sessionId")]
    pub session_id: String,
    #[serde(rename = "requestID", alias = "requestId")]
    pub request_id: String,
    pub reply: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SessionDiffProperties {
    #[serde(rename = "sessionID", alias = "sessionId")]
    pub session_id: String,
    #[serde(default)]
    pub diff: Vec<SnapshotFileDiff>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SessionErrorProperties {
    #[serde(rename = "sessionID", alias = "sessionId")]
    pub session_id: Option<String>,
    pub error: Option<OpencodeError>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TodoUpdatedProperties {
    #[serde(rename = "sessionID", alias = "sessionId")]
    pub session_id: String,
    #[serde(default)]
    pub todos: Vec<TodoItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SessionStatusProperties {
    #[serde(rename = "sessionID", alias = "sessionId")]
    pub session_id: String,
    pub status: SessionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SessionIdProperties {
    #[serde(rename = "sessionID", alias = "sessionId")]
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TextProperties {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CommandProperties {
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ToastProperties {
    pub title: Option<String>,
    pub message: String,
    pub variant: String,
    pub duration: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ServerProperties {
    pub server: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct McpBrowserOpenFailedProperties {
    #[serde(rename = "mcpName", alias = "mcpname")]
    pub mcp_name: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CommandExecutedProperties {
    pub name: String,
    #[serde(rename = "sessionID", alias = "sessionId")]
    pub session_id: String,
    pub arguments: String,
    #[serde(rename = "messageID", alias = "messageId")]
    pub message_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BranchProperties {
    pub branch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorktreeReadyProperties {
    pub name: String,
    pub branch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MessageProperties {
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PtyInfoProperties {
    pub info: PtyInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PtyExitedProperties {
    pub id: String,
    pub exit_code: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct IdProperties {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NameProperties {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceRestoreProperties {
    #[serde(rename = "workspaceID", alias = "workspaceId")]
    pub workspace_id: String,
    #[serde(rename = "sessionID", alias = "sessionId")]
    pub session_id: String,
    pub total: i64,
    pub step: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceStatusProperties {
    #[serde(rename = "workspaceID", alias = "workspaceId")]
    pub workspace_id: String,
    pub status: String,
}
