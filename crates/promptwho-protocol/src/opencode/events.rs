use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use uuid::Uuid;

use super::message::{
    MessagePartRemovedProperties, MessagePartUpdatedProperties, MessageRemovedProperties,
    MessageUpdatedProperties,
};
use super::session::{PartialSessionInfoProperties, SessionInfoProperties};
use super::shared::{
    BranchProperties, CommandExecutedProperties, CommandProperties, DirectoryProperties,
    FileProperties, FileWatcherProperties, IdProperties, LspClientDiagnosticsProperties,
    McpBrowserOpenFailedProperties, MessagePartDeltaProperties, MessageProperties,
    NameProperties, OpencodeContext, OpencodeProject, PermissionRepliedProperties,
    PermissionRequest, PtyExitedProperties, PtyInfoProperties, QuestionRejected,
    QuestionReplied, QuestionRequest, ServerProperties, SessionDiffProperties,
    SessionErrorProperties, SessionIdProperties, SessionStatusProperties, TextProperties,
    TodoUpdatedProperties, ToastProperties, VersionProperties, WorktreeReadyProperties,
    WorkspaceRestoreProperties, WorkspaceStatusProperties,
};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "name")]
pub enum OpencodeSyncEvent {
    #[serde(rename = "message.updated.1")]
    MessageUpdated {
        id: String,
        seq: i64,
        aggregate_id: String,
        data: MessageUpdatedProperties,
    },
    #[serde(rename = "message.removed.1")]
    MessageRemoved {
        id: String,
        seq: i64,
        aggregate_id: String,
        data: MessageRemovedProperties,
    },
    #[serde(rename = "message.part.updated.1")]
    MessagePartUpdated {
        id: String,
        seq: i64,
        aggregate_id: String,
        data: MessagePartUpdatedProperties,
    },
    #[serde(rename = "message.part.removed.1")]
    MessagePartRemoved {
        id: String,
        seq: i64,
        aggregate_id: String,
        data: MessagePartRemovedProperties,
    },
    #[serde(rename = "session.created.1")]
    SessionCreated {
        id: String,
        seq: i64,
        aggregate_id: String,
        data: SessionInfoProperties,
    },
    #[serde(rename = "session.updated.1")]
    SessionUpdated {
        id: String,
        seq: i64,
        aggregate_id: String,
        data: PartialSessionInfoProperties,
    },
    #[serde(rename = "session.deleted.1")]
    SessionDeleted {
        id: String,
        seq: i64,
        aggregate_id: String,
        data: SessionInfoProperties,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type")]
pub enum OpencodeEvent {
    #[serde(rename = "project.updated")]
    ProjectUpdated { properties: OpencodeProject },
    #[serde(rename = "server.instance.disposed")]
    ServerInstanceDisposed { properties: DirectoryProperties },
    #[serde(rename = "server.connected")]
    ServerConnected { properties: Value },
    #[serde(rename = "global.disposed")]
    GlobalDisposed { properties: Value },
    #[serde(rename = "file.edited")]
    FileEdited { properties: FileProperties },
    #[serde(rename = "file.watcher.updated")]
    FileWatcherUpdated { properties: FileWatcherProperties },
    #[serde(rename = "lsp.client.diagnostics")]
    LspClientDiagnostics {
        properties: LspClientDiagnosticsProperties,
    },
    #[serde(rename = "lsp.updated")]
    LspUpdated { properties: Value },
    #[serde(rename = "installation.updated")]
    InstallationUpdated { properties: VersionProperties },
    #[serde(rename = "installation.update-available")]
    InstallationUpdateAvailable { properties: VersionProperties },
    #[serde(rename = "message.part.delta")]
    MessagePartDelta {
        properties: MessagePartDeltaProperties,
    },
    #[serde(rename = "permission.asked")]
    PermissionAsked { properties: PermissionRequest },
    #[serde(rename = "permission.replied")]
    PermissionReplied {
        properties: PermissionRepliedProperties,
    },
    #[serde(rename = "session.diff")]
    SessionDiff { properties: SessionDiffProperties },
    #[serde(rename = "session.error")]
    SessionError { properties: SessionErrorProperties },
    #[serde(rename = "question.asked")]
    QuestionAsked { properties: QuestionRequest },
    #[serde(rename = "question.replied")]
    QuestionReplied { properties: QuestionReplied },
    #[serde(rename = "question.rejected")]
    QuestionRejected { properties: QuestionRejected },
    #[serde(rename = "todo.updated")]
    TodoUpdated { properties: TodoUpdatedProperties },
    #[serde(rename = "session.status")]
    SessionStatus { properties: SessionStatusProperties },
    #[serde(rename = "session.idle")]
    SessionIdle { properties: SessionIdProperties },
    #[serde(rename = "session.compacted")]
    SessionCompacted { properties: SessionIdProperties },
    #[serde(rename = "tui.prompt.append")]
    TuiPromptAppend { properties: TextProperties },
    #[serde(rename = "tui.command.execute")]
    TuiCommandExecute { properties: CommandProperties },
    #[serde(rename = "tui.toast.show")]
    TuiToastShow { properties: ToastProperties },
    #[serde(rename = "tui.session.select")]
    TuiSessionSelect { properties: SessionIdProperties },
    #[serde(rename = "mcp.tools.changed")]
    McpToolsChanged { properties: ServerProperties },
    #[serde(rename = "mcp.browser.open.failed")]
    McpBrowserOpenFailed {
        properties: McpBrowserOpenFailedProperties,
    },
    #[serde(rename = "command.executed")]
    CommandExecuted {
        properties: CommandExecutedProperties,
    },
    #[serde(rename = "vcs.branch.updated")]
    VcsBranchUpdated { properties: BranchProperties },
    #[serde(rename = "worktree.ready")]
    WorktreeReady { properties: WorktreeReadyProperties },
    #[serde(rename = "worktree.failed")]
    WorktreeFailed { properties: MessageProperties },
    #[serde(rename = "pty.created")]
    PtyCreated { properties: PtyInfoProperties },
    #[serde(rename = "pty.updated")]
    PtyUpdated { properties: PtyInfoProperties },
    #[serde(rename = "pty.exited")]
    PtyExited { properties: PtyExitedProperties },
    #[serde(rename = "pty.deleted")]
    PtyDeleted { properties: IdProperties },
    #[serde(rename = "workspace.ready")]
    WorkspaceReady { properties: NameProperties },
    #[serde(rename = "workspace.failed")]
    WorkspaceFailed { properties: MessageProperties },
    #[serde(rename = "workspace.restore")]
    WorkspaceRestore {
        properties: WorkspaceRestoreProperties,
    },
    #[serde(rename = "workspace.status")]
    WorkspaceStatus {
        properties: WorkspaceStatusProperties,
    },
    #[serde(rename = "message.updated")]
    MessageUpdated {
        properties: MessageUpdatedProperties,
    },
    #[serde(rename = "message.removed")]
    MessageRemoved {
        properties: MessageRemovedProperties,
    },
    #[serde(rename = "message.part.updated")]
    MessagePartUpdated {
        properties: MessagePartUpdatedProperties,
    },
    #[serde(rename = "message.part.removed")]
    MessagePartRemoved {
        properties: MessagePartRemovedProperties,
    },
    #[serde(rename = "session.created")]
    SessionCreated { properties: SessionInfoProperties },
    #[serde(rename = "session.updated")]
    SessionUpdated { properties: SessionInfoProperties },
    #[serde(rename = "session.deleted")]
    SessionDeleted { properties: SessionInfoProperties },
    #[serde(rename = "sync")]
    Sync(OpencodeSyncEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OpencodeEventEnvelope {
    pub context: OpencodeContext,
    pub event: OpencodeEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct IngestOpencodeEventsRequest {
    #[serde(alias = "requestId")]
    #[serde(deserialize_with = "crate::uuid_serde::deserialize_uuid")]
    pub request_id: Uuid,
    pub events: Vec<OpencodeEventEnvelope>,
}
