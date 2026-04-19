use promptwho_protocol::{EventEnvelope, EventPayload};
use promptwho_storage::{
    AppendOutcome, GitSnapshot, Message, Project, Session, StoreError, StoredEvent, ToolCall,
    ToolResult,
};
use promptwho_storage::{ConversationStore, EventStore, GitStore};
use serde_json::json;
use time::OffsetDateTime;
use tracing::warn;

#[derive(Debug, thiserror::Error)]
pub enum IngestError {
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error("session-scoped event missing session reference")]
    MissingSession,
}

pub struct IngestService<S> {
    pub store: S,
}

impl<S> IngestService<S>
where
    S: EventStore + ConversationStore + GitStore,
{
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub async fn ingest_protocol_event(
        &self,
        envelope: EventEnvelope,
    ) -> Result<AppendOutcome, IngestError> {
        let occurred_at = envelope.occurred_at;

        let session_id = envelope.session.as_ref().map(|session| session.id.clone());

        let stored = StoredEvent {
            id: envelope.id,
            project_id: envelope.project.id.clone(),
            session_id: session_id.clone(),
            occurred_at,
            action: envelope.payload.action().to_string(),
            envelope: envelope.clone(),
            ingested_at: OffsetDateTime::now_utc(),
        };

        let outcome = self.store.append_event(stored).await?;
        if !outcome.inserted {
            return Ok(outcome);
        }

        self.project_event(envelope, occurred_at).await?;
        Ok(outcome)
    }

    async fn project_event(
        &self,
        envelope: EventEnvelope,
        occurred_at: OffsetDateTime,
    ) -> Result<(), IngestError> {
        self.store
            .upsert_project(Project {
                id: envelope.project.id.clone(),
                root: envelope.project.root.clone(),
                name: envelope.project.name.clone(),
                created_at: occurred_at,
            })
            .await?;

        match envelope.payload {
            EventPayload::SessionStarted(payload) => {
                let session = envelope.session.ok_or(IngestError::MissingSession)?;
                self.store
                    .upsert_session(Session {
                        id: session.id,
                        project_id: envelope.project.id,
                        provider: payload.provider,
                        model: payload.model,
                        branch: payload.branch,
                        head_commit: payload.head_commit,
                        started_at: occurred_at,
                        ended_at: None,
                        metadata: payload.metadata,
                    })
                    .await?;
            }
            EventPayload::SessionEnded(payload) => {
                let session = envelope.session.ok_or(IngestError::MissingSession)?;
                let session = Session {
                    id: session.id,
                    project_id: envelope.project.id,
                    provider: "unknown".to_string(),
                    model: "unknown".to_string(),
                    branch: None,
                    head_commit: None,
                    started_at: occurred_at,
                    ended_at: Some(occurred_at),
                    metadata: payload.metadata,
                };
                self.store.upsert_session(session).await?;
            }
            EventPayload::MessageAdded(payload) => {
                let session = envelope.session.ok_or(IngestError::MissingSession)?;
                self.store
                    .append_message(Message {
                        id: payload.message_id,
                        session_id: session.id,
                        role: payload.role,
                        content: payload.content,
                        token_count: payload.token_count,
                        created_at: occurred_at,
                        metadata: payload.metadata,
                    })
                    .await?;
            }
            EventPayload::ToolCalled(payload) => {
                let session = envelope.session.ok_or(IngestError::MissingSession)?;
                self.store
                    .record_tool_call(ToolCall {
                        id: payload.tool_call_id,
                        session_id: session.id,
                        tool_name: payload.tool_name,
                        input: payload.input,
                        created_at: occurred_at,
                        metadata: payload.metadata,
                    })
                    .await?;
            }
            EventPayload::ToolResult(payload) => {
                self.store
                    .record_tool_result(ToolResult {
                        tool_call_id: payload.tool_call_id,
                        success: payload.success,
                        output: payload.output,
                        created_at: occurred_at,
                        metadata: payload.metadata,
                    })
                    .await?;
            }
            EventPayload::GitSnapshot(payload) => {
                self.store
                    .record_git_snapshot(GitSnapshot {
                        id: envelope.id,
                        project_id: envelope.project.id,
                        session_id: envelope.session.map(|session| session.id),
                        branch: payload.branch,
                        head_commit: payload.head_commit,
                        dirty: payload.dirty,
                        staged_files: payload.staged_files,
                        unstaged_files: payload.unstaged_files,
                        created_at: occurred_at,
                    })
                    .await?;
            }
            EventPayload::TraceLinked(payload) => {
                warn!(trace_id = %payload.trace_id, "trace projection not implemented yet");
                let _ = json!(payload);
            }
        }

        Ok(())
    }
}
