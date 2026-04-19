use async_trait::async_trait;

use crate::models::*;
use crate::queries::*;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("storage error: {0}")]
    Message(String),
}

#[async_trait]
pub trait EventStore: Send + Sync {
    async fn append_event(&self, event: StoredEvent) -> Result<AppendOutcome, StoreError>;
    async fn append_events(&self, events: Vec<StoredEvent>) -> Result<AppendSummary, StoreError>;
    async fn get_event(&self, id: uuid::Uuid) -> Result<Option<StoredEvent>, StoreError>;
    async fn list_events(&self, query: EventQuery) -> Result<Vec<StoredEvent>, StoreError>;
}

#[async_trait]
pub trait ConversationStore: Send + Sync {
    async fn upsert_project(&self, project: Project) -> Result<(), StoreError>;
    async fn upsert_session(&self, session: Session) -> Result<(), StoreError>;
    async fn append_message(&self, message: Message) -> Result<(), StoreError>;
    async fn record_tool_call(&self, call: ToolCall) -> Result<(), StoreError>;
    async fn record_tool_result(&self, result: ToolResult) -> Result<(), StoreError>;

    async fn get_session(&self, id: SessionId) -> Result<Option<Session>, StoreError>;
    async fn list_sessions(&self, query: SessionQuery) -> Result<Vec<SessionSummary>, StoreError>;
    async fn list_messages(&self, session_id: SessionId) -> Result<Vec<Message>, StoreError>;
}

#[async_trait]
pub trait GitStore: Send + Sync {
    async fn record_git_snapshot(&self, snapshot: GitSnapshot) -> Result<(), StoreError>;
    async fn record_commit(
        &self,
        commit: GitCommit,
        files: Vec<GitCommitFile>,
        hunks: Vec<GitCommitHunk>,
    ) -> Result<(), StoreError>;

    async fn get_commit(&self, oid: GitOid) -> Result<Option<GitCommit>, StoreError>;
    async fn list_commits_for_project(
        &self,
        project_id: ProjectId,
        query: CommitQuery,
    ) -> Result<Vec<GitCommit>, StoreError>;
    async fn list_file_history(
        &self,
        project_id: ProjectId,
        path: &str,
    ) -> Result<Vec<GitFileHistoryRow>, StoreError>;
    async fn list_commit_hunks(&self, oid: GitOid) -> Result<Vec<GitCommitHunk>, StoreError>;
}

#[async_trait]
pub trait TraceStore: Send + Sync {
    async fn upsert_execution_trace(&self, trace: ExecutionTrace) -> Result<(), StoreError>;
    async fn record_trace_frames(&self, frames: Vec<TraceFrame>) -> Result<(), StoreError>;
    async fn write_code_locations(&self, locations: Vec<CodeLocation>) -> Result<(), StoreError>;

    async fn get_trace(&self, trace_id: &str) -> Result<Option<ExecutionTrace>, StoreError>;
    async fn list_trace_frames(&self, trace_id: &str) -> Result<Vec<TraceFrame>, StoreError>;
    async fn find_code_locations(
        &self,
        query: TraceLinkQuery,
    ) -> Result<Vec<CodeLocation>, StoreError>;
}

#[async_trait]
pub trait ChangeStore: Send + Sync {
    async fn record_session_change(
        &self,
        change: SessionCodeChange,
        hunks: Vec<SessionChangeHunk>,
    ) -> Result<(), StoreError>;
    async fn list_session_change_hunks(
        &self,
        session_id: SessionId,
    ) -> Result<Vec<SessionChangeHunk>, StoreError>;
}

#[async_trait]
pub trait AttributionStore: Send + Sync {
    async fn write_patch_attributions(
        &self,
        attributions: Vec<PatchAttribution>,
    ) -> Result<(), StoreError>;
    async fn write_commit_session_summaries(
        &self,
        summaries: Vec<CommitSessionSummary>,
    ) -> Result<(), StoreError>;
    async fn find_patch_attributions(
        &self,
        query: CommitAttributionQuery,
    ) -> Result<Vec<PatchAttribution>, StoreError>;
    async fn find_commit_contributors(
        &self,
        oid: GitOid,
    ) -> Result<Vec<CommitSessionSummary>, StoreError>;
    async fn find_file_contributors(
        &self,
        project_id: ProjectId,
        path: &str,
    ) -> Result<Vec<CommitSessionSummary>, StoreError>;
}

#[async_trait]
pub trait SearchStore: Send + Sync {
    async fn search_text(&self, query: TextSearchQuery) -> Result<SearchResults, StoreError>;
}

#[async_trait]
pub trait VectorSearchStore: Send + Sync {
    async fn upsert_embedding(&self, embedding: EmbeddingRecord) -> Result<(), StoreError>;
    async fn search_similar(&self, query: VectorSearchQuery) -> Result<Vec<VectorHit>, StoreError>;
}

pub trait Storage:
    EventStore
    + ConversationStore
    + GitStore
    + TraceStore
    + ChangeStore
    + AttributionStore
    + SearchStore
{
}

impl<T> Storage for T where
    T: EventStore
        + ConversationStore
        + GitStore
        + TraceStore
        + ChangeStore
        + AttributionStore
        + SearchStore
{
}

#[async_trait]
impl<T> EventStore for &T
where
    T: EventStore + Sync,
{
    async fn append_event(&self, event: StoredEvent) -> Result<AppendOutcome, StoreError> {
        (**self).append_event(event).await
    }

    async fn append_events(&self, events: Vec<StoredEvent>) -> Result<AppendSummary, StoreError> {
        (**self).append_events(events).await
    }

    async fn get_event(&self, id: uuid::Uuid) -> Result<Option<StoredEvent>, StoreError> {
        (**self).get_event(id).await
    }

    async fn list_events(&self, query: EventQuery) -> Result<Vec<StoredEvent>, StoreError> {
        (**self).list_events(query).await
    }
}

#[async_trait]
impl<T> ConversationStore for &T
where
    T: ConversationStore + Sync,
{
    async fn upsert_project(&self, project: Project) -> Result<(), StoreError> {
        (**self).upsert_project(project).await
    }

    async fn upsert_session(&self, session: Session) -> Result<(), StoreError> {
        (**self).upsert_session(session).await
    }

    async fn append_message(&self, message: Message) -> Result<(), StoreError> {
        (**self).append_message(message).await
    }

    async fn record_tool_call(&self, call: ToolCall) -> Result<(), StoreError> {
        (**self).record_tool_call(call).await
    }

    async fn record_tool_result(&self, result: ToolResult) -> Result<(), StoreError> {
        (**self).record_tool_result(result).await
    }

    async fn get_session(&self, id: SessionId) -> Result<Option<Session>, StoreError> {
        (**self).get_session(id).await
    }

    async fn list_sessions(&self, query: SessionQuery) -> Result<Vec<SessionSummary>, StoreError> {
        (**self).list_sessions(query).await
    }

    async fn list_messages(&self, session_id: SessionId) -> Result<Vec<Message>, StoreError> {
        (**self).list_messages(session_id).await
    }
}

#[async_trait]
impl<T> GitStore for &T
where
    T: GitStore + Sync,
{
    async fn record_git_snapshot(&self, snapshot: GitSnapshot) -> Result<(), StoreError> {
        (**self).record_git_snapshot(snapshot).await
    }

    async fn record_commit(
        &self,
        commit: GitCommit,
        files: Vec<GitCommitFile>,
        hunks: Vec<GitCommitHunk>,
    ) -> Result<(), StoreError> {
        (**self).record_commit(commit, files, hunks).await
    }

    async fn get_commit(&self, oid: GitOid) -> Result<Option<GitCommit>, StoreError> {
        (**self).get_commit(oid).await
    }

    async fn list_commits_for_project(
        &self,
        project_id: ProjectId,
        query: CommitQuery,
    ) -> Result<Vec<GitCommit>, StoreError> {
        (**self).list_commits_for_project(project_id, query).await
    }

    async fn list_file_history(
        &self,
        project_id: ProjectId,
        path: &str,
    ) -> Result<Vec<GitFileHistoryRow>, StoreError> {
        (**self).list_file_history(project_id, path).await
    }

    async fn list_commit_hunks(&self, oid: GitOid) -> Result<Vec<GitCommitHunk>, StoreError> {
        (**self).list_commit_hunks(oid).await
    }
}

#[async_trait]
impl<T> TraceStore for &T
where
    T: TraceStore + Sync,
{
    async fn upsert_execution_trace(&self, trace: ExecutionTrace) -> Result<(), StoreError> {
        (**self).upsert_execution_trace(trace).await
    }

    async fn record_trace_frames(&self, frames: Vec<TraceFrame>) -> Result<(), StoreError> {
        (**self).record_trace_frames(frames).await
    }

    async fn write_code_locations(&self, locations: Vec<CodeLocation>) -> Result<(), StoreError> {
        (**self).write_code_locations(locations).await
    }

    async fn get_trace(&self, trace_id: &str) -> Result<Option<ExecutionTrace>, StoreError> {
        (**self).get_trace(trace_id).await
    }

    async fn list_trace_frames(&self, trace_id: &str) -> Result<Vec<TraceFrame>, StoreError> {
        (**self).list_trace_frames(trace_id).await
    }

    async fn find_code_locations(
        &self,
        query: TraceLinkQuery,
    ) -> Result<Vec<CodeLocation>, StoreError> {
        (**self).find_code_locations(query).await
    }
}

#[async_trait]
impl<T> ChangeStore for &T
where
    T: ChangeStore + Sync,
{
    async fn record_session_change(
        &self,
        change: SessionCodeChange,
        hunks: Vec<SessionChangeHunk>,
    ) -> Result<(), StoreError> {
        (**self).record_session_change(change, hunks).await
    }

    async fn list_session_change_hunks(
        &self,
        session_id: SessionId,
    ) -> Result<Vec<SessionChangeHunk>, StoreError> {
        (**self).list_session_change_hunks(session_id).await
    }
}

#[async_trait]
impl<T> AttributionStore for &T
where
    T: AttributionStore + Sync,
{
    async fn write_patch_attributions(
        &self,
        attributions: Vec<PatchAttribution>,
    ) -> Result<(), StoreError> {
        (**self).write_patch_attributions(attributions).await
    }

    async fn write_commit_session_summaries(
        &self,
        summaries: Vec<CommitSessionSummary>,
    ) -> Result<(), StoreError> {
        (**self).write_commit_session_summaries(summaries).await
    }

    async fn find_patch_attributions(
        &self,
        query: CommitAttributionQuery,
    ) -> Result<Vec<PatchAttribution>, StoreError> {
        (**self).find_patch_attributions(query).await
    }

    async fn find_commit_contributors(
        &self,
        oid: GitOid,
    ) -> Result<Vec<CommitSessionSummary>, StoreError> {
        (**self).find_commit_contributors(oid).await
    }

    async fn find_file_contributors(
        &self,
        project_id: ProjectId,
        path: &str,
    ) -> Result<Vec<CommitSessionSummary>, StoreError> {
        (**self).find_file_contributors(project_id, path).await
    }
}
