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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository_fingerprint: Option<String>,
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
    #[serde(rename = "server.instance.disposed")]
    ServerInstanceDisposed,
    #[serde(rename = "server.connected")]
    ServerConnected,
    #[serde(rename = "installation.updated")]
    InstallationUpdated,
    #[serde(rename = "installation.update-available")]
    InstallationUpdateAvailable,
    #[serde(rename = "lsp.client.diagnostics")]
    LspClientDiagnostics,
    #[serde(rename = "lsp.updated")]
    LspUpdated,
    #[serde(rename = "message.updated")]
    MessageUpdated,
    #[serde(rename = "message.removed")]
    MessageRemoved,
    #[serde(rename = "message.part.updated")]
    MessagePartUpdated,
    #[serde(rename = "message.part.removed")]
    MessagePartRemoved,
    #[serde(rename = "permission.updated")]
    PermissionUpdated,
    #[serde(rename = "permission.replied")]
    PermissionReplied,
    #[serde(rename = "session.status")]
    SessionStatus,
    #[serde(rename = "session.idle")]
    SessionIdle,
    #[serde(rename = "session.compacted")]
    SessionCompacted,
    #[serde(rename = "file.edited")]
    FileEdited,
    #[serde(rename = "todo.updated")]
    TodoUpdated,
    #[serde(rename = "command.executed")]
    CommandExecuted,
    #[serde(rename = "session.created")]
    SessionCreated,
    #[serde(rename = "session.updated")]
    SessionUpdated,
    #[serde(rename = "session.deleted")]
    SessionDeleted,
    #[serde(rename = "session.diff")]
    SessionDiff,
    #[serde(rename = "session.error")]
    SessionError,
    #[serde(rename = "file.watcher.updated")]
    FileWatcherUpdated,
    #[serde(rename = "vcs.branch.updated")]
    VcsBranchUpdated,
    #[serde(rename = "tui.prompt.append")]
    TuiPromptAppend,
    #[serde(rename = "tui.command.execute")]
    TuiCommandExecute,
    #[serde(rename = "tui.toast.show")]
    TuiToastShow,
    #[serde(rename = "pty.created")]
    PtyCreated,
    #[serde(rename = "pty.updated")]
    PtyUpdated,
    #[serde(rename = "pty.exited")]
    PtyExited,
    #[serde(rename = "pty.deleted")]
    PtyDeleted,
    #[serde(rename = "tool.execute.before")]
    ToolExecuteBefore,
    #[serde(rename = "tool.execute.after")]
    ToolExecuteAfter,
    #[serde(rename = "shell.env")]
    ShellEnv,
    #[serde(rename = "git.snapshot")]
    GitSnapshot,
    #[serde(rename = "git.commit")]
    GitCommit,
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
    #[serde(rename = "server.instance.disposed")]
    ServerInstanceDisposed(ServerInstanceDisposedPayload),
    #[serde(rename = "server.connected")]
    ServerConnected,
    #[serde(rename = "installation.updated")]
    InstallationUpdated(VersionPayload),
    #[serde(rename = "installation.update-available")]
    InstallationUpdateAvailable(VersionPayload),
    #[serde(rename = "lsp.client.diagnostics")]
    LspClientDiagnostics(LspClientDiagnosticsPayload),
    #[serde(rename = "lsp.updated")]
    LspUpdated,
    #[serde(rename = "message.updated")]
    MessageUpdated(MessageUpdatedPayload),
    #[serde(rename = "message.removed")]
    MessageRemoved(MessageRemovedPayload),
    #[serde(rename = "message.part.updated")]
    MessagePartUpdated(MessagePartUpdatedPayload),
    #[serde(rename = "message.part.removed")]
    MessagePartRemoved(MessagePartRemovedPayload),
    #[serde(rename = "permission.updated")]
    PermissionUpdated(PermissionUpdatedPayload),
    #[serde(rename = "permission.replied")]
    PermissionReplied(PermissionRepliedPayload),
    #[serde(rename = "session.status")]
    SessionStatus(SessionStatusPayload),
    #[serde(rename = "session.idle")]
    SessionIdle,
    #[serde(rename = "session.compacted")]
    SessionCompacted,
    #[serde(rename = "file.edited")]
    FileEdited(FileEditedPayload),
    #[serde(rename = "todo.updated")]
    TodoUpdated(TodoUpdatedPayload),
    #[serde(rename = "command.executed")]
    CommandExecuted(CommandExecutedPayload),
    #[serde(rename = "session.created")]
    SessionCreated,
    #[serde(rename = "session.updated")]
    SessionUpdated,
    #[serde(rename = "session.deleted")]
    SessionDeleted,
    #[serde(rename = "session.diff")]
    SessionDiff(SessionDiffPayload),
    #[serde(rename = "session.error")]
    SessionError(SessionErrorPayload),
    #[serde(rename = "file.watcher.updated")]
    FileWatcherUpdated(FileWatcherUpdatedPayload),
    #[serde(rename = "vcs.branch.updated")]
    VcsBranchUpdated(VcsBranchUpdatedPayload),
    #[serde(rename = "tui.prompt.append")]
    TuiPromptAppend(TuiPromptAppendPayload),
    #[serde(rename = "tui.command.execute")]
    TuiCommandExecute(TuiCommandExecutePayload),
    #[serde(rename = "tui.toast.show")]
    TuiToastShow(TuiToastShowPayload),
    #[serde(rename = "pty.created")]
    PtyCreated(PtyPayload),
    #[serde(rename = "pty.updated")]
    PtyUpdated(PtyPayload),
    #[serde(rename = "pty.exited")]
    PtyExited(PtyPayload),
    #[serde(rename = "pty.deleted")]
    PtyDeleted(PtyPayload),
    #[serde(rename = "tool.execute.before")]
    ToolExecuteBefore(ToolExecuteBeforePayload),
    #[serde(rename = "tool.execute.after")]
    ToolExecuteAfter(ToolExecuteAfterPayload),
    #[serde(rename = "shell.env")]
    ShellEnv(ShellEnvPayload),
    #[serde(rename = "git.snapshot")]
    GitSnapshot(GitSnapshotPayload),
    #[serde(rename = "git.commit")]
    GitCommit(GitCommitPayload),
    #[serde(rename = "trace.linked")]
    TraceLinked(TraceLinkedPayload),
}

impl EventPayload {
    pub fn action(&self) -> EventAction {
        match self {
            EventPayload::ServerInstanceDisposed(_) => EventAction::ServerInstanceDisposed,
            EventPayload::ServerConnected => EventAction::ServerConnected,
            EventPayload::InstallationUpdated(_) => EventAction::InstallationUpdated,
            EventPayload::InstallationUpdateAvailable(_) => {
                EventAction::InstallationUpdateAvailable
            }
            EventPayload::LspClientDiagnostics(_) => EventAction::LspClientDiagnostics,
            EventPayload::LspUpdated => EventAction::LspUpdated,
            EventPayload::MessageUpdated(_) => EventAction::MessageUpdated,
            EventPayload::MessageRemoved(_) => EventAction::MessageRemoved,
            EventPayload::MessagePartUpdated(_) => EventAction::MessagePartUpdated,
            EventPayload::MessagePartRemoved(_) => EventAction::MessagePartRemoved,
            EventPayload::PermissionUpdated(_) => EventAction::PermissionUpdated,
            EventPayload::PermissionReplied(_) => EventAction::PermissionReplied,
            EventPayload::SessionStatus(_) => EventAction::SessionStatus,
            EventPayload::SessionIdle => EventAction::SessionIdle,
            EventPayload::SessionCompacted => EventAction::SessionCompacted,
            EventPayload::FileEdited(_) => EventAction::FileEdited,
            EventPayload::TodoUpdated(_) => EventAction::TodoUpdated,
            EventPayload::CommandExecuted(_) => EventAction::CommandExecuted,
            EventPayload::SessionCreated => EventAction::SessionCreated,
            EventPayload::SessionUpdated => EventAction::SessionUpdated,
            EventPayload::SessionDeleted => EventAction::SessionDeleted,
            EventPayload::SessionDiff(_) => EventAction::SessionDiff,
            EventPayload::SessionError(_) => EventAction::SessionError,
            EventPayload::FileWatcherUpdated(_) => EventAction::FileWatcherUpdated,
            EventPayload::VcsBranchUpdated(_) => EventAction::VcsBranchUpdated,
            EventPayload::TuiPromptAppend(_) => EventAction::TuiPromptAppend,
            EventPayload::TuiCommandExecute(_) => EventAction::TuiCommandExecute,
            EventPayload::TuiToastShow(_) => EventAction::TuiToastShow,
            EventPayload::PtyCreated(_) => EventAction::PtyCreated,
            EventPayload::PtyUpdated(_) => EventAction::PtyUpdated,
            EventPayload::PtyExited(_) => EventAction::PtyExited,
            EventPayload::PtyDeleted(_) => EventAction::PtyDeleted,
            EventPayload::ToolExecuteBefore(_) => EventAction::ToolExecuteBefore,
            EventPayload::ToolExecuteAfter(_) => EventAction::ToolExecuteAfter,
            EventPayload::ShellEnv(_) => EventAction::ShellEnv,
            EventPayload::GitSnapshot(_) => EventAction::GitSnapshot,
            EventPayload::TraceLinked(_) => EventAction::TraceLinked,
            EventPayload::GitCommit(_) => EventAction::GitCommit,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EventEnvelope {
    #[serde(deserialize_with = "crate::uuid_serde::deserialize_uuid")]
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
pub struct VersionPayload {
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ServerInstanceDisposedPayload {
    pub directory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LspClientDiagnosticsPayload {
    pub server_id: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MessageUpdatedPayload {
    pub message_id: String,
    pub role: String,
    pub content: Option<String>,
    pub token_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MessageRemovedPayload {
    pub message_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MessagePartUpdatedPayload {
    pub message_id: String,
    pub part_id: String,
    pub part_type: String,
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MessagePartRemovedPayload {
    pub message_id: String,
    pub part_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PermissionUpdatedPayload {
    pub permission_id: String,
    pub permission_type: String,
    pub message_id: Option<String>,
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PermissionRepliedPayload {
    pub permission_id: String,
    pub response: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SessionStatusPayload {
    pub status: String,
    pub attempt: Option<u32>,
    pub message: Option<String>,
    pub next: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FileEditedPayload {
    pub file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TodoUpdatedPayload {
    pub todo_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CommandExecutedPayload {
    pub message_id: String,
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SessionDiffFile {
    pub file: String,
    pub patch: String,
    pub additions: i64,
    pub deletions: i64,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SessionDiffPayload {
    #[serde(default)]
    pub diff: Vec<SessionDiffFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SessionErrorPayload {
    pub error_name: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FileWatcherUpdatedPayload {
    pub file: String,
    pub event: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VcsBranchUpdatedPayload {
    pub branch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TuiPromptAppendPayload {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TuiCommandExecutePayload {
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TuiToastShowPayload {
    pub variant: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PtyPayload {
    pub pty_id: String,
    pub command: Option<String>,
    pub cwd: Option<String>,
    pub status: Option<String>,
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ToolExecuteBeforePayload {
    pub tool_call_id: String,
    pub tool_name: String,
    #[schema(value_type = Object)]
    pub input: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ToolExecuteAfterPayload {
    pub tool_call_id: String,
    pub tool_name: String,
    pub success: bool,
    #[schema(value_type = Object)]
    pub output: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ShellEnvPayload {
    pub cwd: String,
    pub tool_call_id: Option<String>,
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

// TODO: Not sure if this is  required or just reuse snapshot?
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GitCommitPayload {
    pub branch: Option<String>,
    pub head_commit: Option<String>,
    pub parent_commit: Option<String>,
    pub commit_author_name: Option<String>,
    pub commit_author_email: Option<String>,
    #[schema(value_type = String, format = "date-time")]
    pub commit_timestamp: Option<TimestampUtc>,
    pub commit_title: Option<String>,
    pub commit_body: Option<String>,
    pub message: Option<String>,
    #[serde(default)]
    pub files: Vec<GitCommitFilePayload>,
    #[serde(default)]
    pub hunks: Vec<GitCommitHunkPayload>,
    pub dirty: bool,
    #[serde(default)]
    pub staged_files: Vec<String>,
    #[serde(default)]
    pub unstaged_files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct GitCommitFilePayload {
    pub path: String,
    pub old_path: Option<String>,
    pub change_kind: String,
    pub hunk_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct GitCommitHunkPayload {
    #[serde(deserialize_with = "crate::uuid_serde::deserialize_uuid")]
    #[schema(value_type = String, format = "uuid")]
    pub id: Uuid,
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
