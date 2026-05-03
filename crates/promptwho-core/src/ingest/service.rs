use promptwho_protocol::{EventEnvelope, EventPayload, ProjectRefId};
use promptwho_storage::{
    AppendOutcome, GitCommit, GitCommitFile, GitCommitHunk, GitSnapshot, Message as StoredMessage,
    Project, ProjectForeignId, ProjectQuery, Session, StoreError, StoredEvent, ToolCall,
};
use promptwho_storage::{ConversationStore, EventStore, GitStore};
use serde_json::{Value, json};
use tracing::warn;

use promptwho_protocol::TimestampUtc;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum IngestError {
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error("session-scoped event missing session reference")]
    MissingSession,
    #[error("invalid event payload: {0}")]
    InvalidEvent(&'static str),
}

async fn upsert_message_content<S>(
    store: &S,
    message_id: String,
    session_id: String,
    role: String,
    content: String,
    token_count: Option<u32>,
    occurred_at: TimestampUtc,
    metadata: Value,
) -> Result<(), IngestError>
where
    S: ConversationStore + ?Sized,
{
    let existing = store.get_message(message_id.clone()).await?;

    let message = if let Some(existing) = existing {
        StoredMessage {
            id: existing.id,
            session_id: existing.session_id,
            role,
            content,
            token_count: token_count.or(existing.token_count),
            created_at: existing.created_at,
            metadata,
        }
    } else {
        StoredMessage {
            id: message_id,
            session_id,
            role,
            content,
            token_count,
            created_at: occurred_at,
            metadata,
        }
    };

    store.append_message(message).await?;
    Ok(())
}

async fn ensure_session_exists<S>(
    store: &S,
    envelope: &EventEnvelope,
    occurred_at: TimestampUtc,
) -> Result<(), IngestError>
where
    S: ConversationStore + ?Sized,
{
    let Some(session) = envelope.session.as_ref() else {
        return Ok(());
    };

    if store.get_session(session.id.clone()).await?.is_some() {
        return Ok(());
    }

    let project_id = match &envelope.project.id {
        ProjectRefId::Id { id } => id.clone(),
        ProjectRefId::Ext { .. } => {
            return Err(IngestError::InvalidEvent(
                "project missing canonical id after normalization",
            ));
        }
    };

    store
        .upsert_session(Session {
            id: session.id.clone(),
            project_id,
            provider: "unknown".to_string(),
            model: "unknown".to_string(),
            started_on_branch: None,
            started_on_head: None,
            started_at: occurred_at,
            ended_at: None,
            metadata: serde_json::Value::Object(Default::default()),
        })
        .await?;

    Ok(())
}

fn project_foreign_id(envelope: &EventEnvelope) -> Option<ProjectForeignId> {
    match &envelope.project.id {
        ProjectRefId::Ext { src, id } if !src.is_empty() && !id.is_empty() => {
            Some(ProjectForeignId {
                pid: String::new(),
                fid: id.clone(),
                source: src.clone(),
            })
        }
        ProjectRefId::Id { .. } | ProjectRefId::Ext { .. } => None,
    }
}

fn canonical_project_id(project_id: &ProjectRefId) -> Result<String, IngestError> {
    match project_id {
        ProjectRefId::Id { id } if !id.is_empty() => Ok(id.clone()),
        ProjectRefId::Id { .. } | ProjectRefId::Ext { .. } => Err(IngestError::InvalidEvent(
            "project missing canonical id after normalization",
        )),
    }
}

fn generated_project_id(envelope: &EventEnvelope) -> String {
    let seed =
        if let Some(repository_fingerprint) = envelope.project.repository_fingerprint.as_deref() {
            format!("fingerprint:{repository_fingerprint}")
        } else if let Some(foreign_id) = project_foreign_id(envelope) {
            format!("foreign:{}:{}", foreign_id.source, foreign_id.fid)
        } else {
            format!("root:{}", envelope.project.root)
        };

    format!(
        "project:{}",
        Uuid::new_v5(&Uuid::NAMESPACE_URL, seed.as_bytes())
    )
}

fn merge_message_part(
    existing: Option<StoredMessage>,
    message_id: String,
    session_id: String,
    part_id: String,
    text: String,
    occurred_at: TimestampUtc,
) -> Result<StoredMessage, IngestError> {
    let mut message = existing.unwrap_or(StoredMessage {
        id: message_id,
        session_id,
        role: "assistant".to_string(),
        content: String::new(),
        token_count: None,
        created_at: occurred_at,
        metadata: Value::Object(Default::default()),
    });

    let metadata = message
        .metadata
        .as_object_mut()
        .ok_or(IngestError::InvalidEvent(
            "message metadata is not an object",
        ))?;

    {
        let part_order = metadata
            .entry("part_order".to_string())
            .or_insert_with(|| Value::Array(Vec::new()));
        let part_order = part_order.as_array_mut().ok_or(IngestError::InvalidEvent(
            "message part_order is not an array",
        ))?;

        let seen_part = part_order
            .iter()
            .any(|value| value.as_str().is_some_and(|value| value == part_id));
        if !seen_part {
            part_order.push(Value::String(part_id.clone()));
        }
    }

    {
        let parts = metadata
            .entry("parts".to_string())
            .or_insert_with(|| Value::Object(Default::default()));
        let parts = parts
            .as_object_mut()
            .ok_or(IngestError::InvalidEvent("message parts is not an object"))?;
        parts.insert(part_id.clone(), Value::String(text));
    }

    let part_order =
        metadata
            .get("part_order")
            .and_then(Value::as_array)
            .ok_or(IngestError::InvalidEvent(
                "message part_order is not an array",
            ))?;
    let parts = metadata
        .get("parts")
        .and_then(Value::as_object)
        .ok_or(IngestError::InvalidEvent("message parts is not an object"))?;
    let content = part_order
        .iter()
        .filter_map(Value::as_str)
        .filter_map(|part_id| parts.get(part_id))
        .filter_map(Value::as_str)
        .collect::<String>();

    message.content = content;
    Ok(message)
}

#[async_trait::async_trait]
pub trait Ingest {
    type Store: EventStore + ConversationStore + GitStore;
    type IngestEventEnvelope: Send + Sync;

    fn store(&self) -> &Self::Store;

    /// FIXME: use type-state pattern to enforce normalization at
    /// compile time instead of runtime of key importance
    /// is the project reference id - so we don't fail here if the project id
    /// has not been resolved
    async fn normalize_event(
        &self,
        envelope: &Self::IngestEventEnvelope,
    ) -> Result<EventEnvelope, IngestError>;

    async fn ingest_protocol_event(
        &self,
        envelope: &Self::IngestEventEnvelope,
    ) -> Result<AppendOutcome, IngestError> {
        let envelope = self.normalize_event(envelope).await?;
        let occurred_at = envelope.occurred_at;
        let session_id = envelope.session.as_ref().map(|session| session.id.clone());
        let project_id = canonical_project_id(&envelope.project.id)?;
        let stored = StoredEvent {
            id: envelope.id,
            project_id,
            session_id: session_id.clone(),
            occurred_at,
            action: envelope.payload.action().to_string(),
            envelope: envelope.clone(),
            ingested_at: chrono::Utc::now(),
        };
        let outcome = self.store().append_event(stored).await?;
        if !outcome.inserted {
            return Ok(outcome);
        }

        self.project_event(envelope, occurred_at).await?;
        Ok(outcome)
    }

    async fn project_event(
        &self,
        envelope: EventEnvelope,
        occurred_at: TimestampUtc,
    ) -> Result<(), IngestError> {
        let project = self
            .store()
            .upsert_project(Project {
                id: canonical_project_id(&envelope.project.id)?,
                root: envelope.project.root.clone(),
                name: envelope.project.name.clone(),
                repository_fingerprint: envelope.project.repository_fingerprint.clone(),
                created_at: occurred_at,
            })
            .await?;

        if let Some(mut foreign_id) = project_foreign_id(&envelope) {
            foreign_id.pid = project.id.clone();
            self.store().upsert_project_foreign_id(foreign_id).await?;
        }

        ensure_session_exists(self.store(), &envelope, occurred_at).await?;

        match envelope.payload {
            EventPayload::SessionCreated => {
                let session = envelope.session.ok_or(IngestError::MissingSession)?;
                let project_id = canonical_project_id(&envelope.project.id)?;
                self.store()
                    .upsert_session(Session {
                        id: session.id,
                        project_id,
                        provider: "unknown".to_string(),
                        model: "unknown".to_string(),
                        started_on_branch: None,
                        started_on_head: None,
                        started_at: occurred_at,
                        ended_at: None,
                        metadata: serde_json::Value::Object(Default::default()),
                    })
                    .await?;
            }
            EventPayload::SessionUpdated => {
                let session = envelope.session.ok_or(IngestError::MissingSession)?;
                let project_id = canonical_project_id(&envelope.project.id)?;
                self.store()
                    .upsert_session(Session {
                        id: session.id,
                        project_id,
                        provider: "unknown".to_string(),
                        model: "unknown".to_string(),
                        started_on_branch: None,
                        started_on_head: None,
                        started_at: occurred_at,
                        ended_at: None,
                        metadata: serde_json::Value::Object(Default::default()),
                    })
                    .await?;
            }
            EventPayload::SessionDeleted => {
                let session = envelope.session.ok_or(IngestError::MissingSession)?;
                let project_id = canonical_project_id(&envelope.project.id)?;
                let session = Session {
                    id: session.id,
                    project_id,
                    provider: "unknown".to_string(),
                    model: "unknown".to_string(),
                    started_on_branch: None,
                    started_on_head: None,
                    started_at: occurred_at,
                    ended_at: Some(occurred_at),
                    metadata: serde_json::Value::Object(Default::default()),
                };
                self.store().upsert_session(session).await?;
            }
            EventPayload::SessionDiff(payload) => {
                warn!(
                    file_count = payload.diff.len(),
                    "session diff projection not implemented yet"
                );
                let _ = json!(payload);
            }
            EventPayload::MessageUpdated(payload) => {
                let session = envelope.session.ok_or(IngestError::MissingSession)?;
                if let Some(content) = payload.content {
                    upsert_message_content(
                        self.store(),
                        payload.message_id,
                        session.id,
                        payload.role,
                        content,
                        payload.token_count,
                        occurred_at,
                        serde_json::Value::Object(Default::default()),
                    )
                    .await?;
                }
            }
            EventPayload::MessagePartUpdated(payload) => {
                let session = envelope.session.ok_or(IngestError::MissingSession)?;
                if payload.part_type == "text"
                    && let Some(content) = payload.text
                {
                    let existing = self.store().get_message(payload.message_id.clone()).await?;
                    let message = merge_message_part(
                        existing,
                        payload.message_id,
                        session.id,
                        payload.part_id,
                        content,
                        occurred_at,
                    )?;
                    self.store().append_message(message).await?;
                }
            }
            EventPayload::ToolExecuteBefore(payload) => {
                let session = envelope.session.ok_or(IngestError::MissingSession)?;
                self.store()
                    .record_tool_call(ToolCall {
                        id: payload.tool_call_id,
                        session_id: session.id,
                        tool_name: payload.tool_name,
                        input: payload.input,
                        created_at: occurred_at,
                        completed_at: None,
                        success: None,
                        output: None,
                        metadata: serde_json::Value::Object(Default::default()),
                    })
                    .await?;
            }
            EventPayload::ToolExecuteAfter(payload) => {
                self.store()
                    .complete_tool_call(
                        payload.tool_call_id,
                        payload.success,
                        payload.output,
                        occurred_at,
                        serde_json::Value::Object(Default::default()),
                    )
                    .await?;
            }
            EventPayload::FileEdited(_) => {
                warn!("file edited projection not implemented yet");
            }
            EventPayload::GitSnapshot(payload) => {
                let project_id = canonical_project_id(&envelope.project.id)?;
                self.store()
                    .record_git_snapshot(GitSnapshot {
                        id: envelope.id,
                        project_id,
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
            EventPayload::GitCommit(payload) => {
                let commit_oid = payload
                    .head_commit
                    .clone()
                    .ok_or(IngestError::InvalidEvent("git.commit missing head_commit"))?;

                let message = payload.message.clone().unwrap_or_else(|| {
                    let title = payload.commit_title.clone().unwrap_or_default();
                    match payload.commit_body.as_deref() {
                        Some(body) if !body.is_empty() && !title.is_empty() => {
                            format!("{title}\n\n{body}")
                        }
                        Some(body) if !body.is_empty() => body.to_string(),
                        _ => title,
                    }
                });

                let project_id = canonical_project_id(&envelope.project.id)?;
                self.store()
                    .record_commit(
                        GitCommit {
                            oid: commit_oid.clone(),
                            project_id,
                            parent_oid: payload.parent_commit,
                            author_name: payload.commit_author_name,
                            author_email: payload.commit_author_email,
                            message,
                            committed_at: payload.commit_timestamp.unwrap_or(occurred_at),
                        },
                        payload
                            .files
                            .into_iter()
                            .map(|file| GitCommitFile {
                                id: format!("{commit_oid}::{}", file.path),
                                commit_oid: commit_oid.clone(),
                                path: file.path,
                                old_path: file.old_path,
                                change_kind: file.change_kind,
                            })
                            .collect(),
                        payload
                            .hunks
                            .into_iter()
                            .map(|hunk| GitCommitHunk {
                                id: hunk.id,
                                commit_file_id: format!("{commit_oid}::{}", hunk.file_path),
                                file_path: hunk.file_path,
                                old_start: hunk.old_start,
                                old_lines: hunk.old_lines,
                                new_start: hunk.new_start,
                                new_lines: hunk.new_lines,
                                hunk_header: hunk.hunk_header,
                                added_line_count: hunk.added_line_count,
                                removed_line_count: hunk.removed_line_count,
                                context_before_hash: hunk.context_before_hash,
                                context_after_hash: hunk.context_after_hash,
                                added_lines_fingerprint: hunk.added_lines_fingerprint,
                                removed_lines_fingerprint: hunk.removed_lines_fingerprint,
                            })
                            .collect(),
                    )
                    .await?;
            }
            EventPayload::ServerInstanceDisposed(_)
            | EventPayload::ServerConnected
            | EventPayload::InstallationUpdated(_)
            | EventPayload::InstallationUpdateAvailable(_)
            | EventPayload::LspClientDiagnostics(_)
            | EventPayload::LspUpdated
            | EventPayload::MessageRemoved(_)
            | EventPayload::MessagePartRemoved(_)
            | EventPayload::PermissionUpdated(_)
            | EventPayload::PermissionReplied(_)
            | EventPayload::SessionStatus(_)
            | EventPayload::SessionIdle
            | EventPayload::SessionCompacted
            | EventPayload::TodoUpdated(_)
            | EventPayload::CommandExecuted(_)
            | EventPayload::SessionError(_)
            | EventPayload::FileWatcherUpdated(_)
            | EventPayload::VcsBranchUpdated(_)
            | EventPayload::TuiPromptAppend(_)
            | EventPayload::TuiCommandExecute(_)
            | EventPayload::TuiToastShow(_)
            | EventPayload::PtyCreated(_)
            | EventPayload::PtyUpdated(_)
            | EventPayload::PtyExited(_)
            | EventPayload::PtyDeleted(_)
            | EventPayload::ShellEnv(_) => {
                warn!(action = %envelope.payload.action(), "projection not implemented yet");
            }
            EventPayload::TraceLinked(payload) => {
                warn!(trace_id = %payload.trace_id, "trace projection not implemented yet");
                let _ = json!(payload);
            }
        }

        Ok(())
    }
}

/// Default implementation of the Ingest trait that can be used with any store that implements
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

    async fn resolve_project(&self, envelope: &EventEnvelope) -> Result<Project, IngestError> {
        if let ProjectRefId::Id { id } = &envelope.project.id {
            let mut projects = self
                .store
                .list_projects(Some(ProjectQuery {
                    id: Some(id.clone()),
                    limit: Some(1),
                    ..Default::default()
                }))
                .await?;
            if let Some(project) = projects.pop() {
                return Ok(project);
            }
        }

        if let Some(foreign_id) = project_foreign_id(envelope)
            && let Some(project) = self
                .store
                .get_project_by_foreign_id(foreign_id.source, foreign_id.fid)
                .await?
        {
            return Ok(project);
        }

        if let Some(repository_fingerprint) = envelope.project.repository_fingerprint.clone()
            && let Some(project) = self
                .store
                .get_project_by_fingerprint(repository_fingerprint)
                .await?
        {
            return Ok(project);
        }

        let mut root_matches = self
            .store
            .list_projects(Some(ProjectQuery {
                root: Some(envelope.project.root.clone()),
                limit: Some(2),
                ..Default::default()
            }))
            .await?;

        if root_matches.len() == 1 {
            return Ok(root_matches.pop().expect("root match length checked"));
        }

        self.store
            .create_project(Project {
                id: generated_project_id(envelope),
                root: envelope.project.root.clone(),
                name: envelope.project.name.clone(),
                repository_fingerprint: envelope.project.repository_fingerprint.clone(),
                created_at: envelope.occurred_at,
            })
            .await
            .map_err(IngestError::from)
    }
}

#[async_trait::async_trait]
impl<S> Ingest for IngestService<S>
where
    S: EventStore + ConversationStore + GitStore,
{
    type Store = S;
    type IngestEventEnvelope = EventEnvelope;

    fn store(&self) -> &Self::Store {
        &self.store
    }

    async fn normalize_event(
        &self,
        envelope: &EventEnvelope,
    ) -> Result<EventEnvelope, IngestError> {
        let project = self.resolve_project(envelope).await?;
        if let Some(mut foreign_id) = project_foreign_id(envelope) {
            foreign_id.pid = project.id.clone();
            self.store.upsert_project_foreign_id(foreign_id).await?;
        }
        let mut normalized = envelope.clone();
        normalized.project.id = ProjectRefId::Id { id: project.id };
        normalized.project.root = project.root;
        normalized.project.name = normalized.project.name.or(project.name);
        normalized.project.repository_fingerprint = project
            .repository_fingerprint
            .or(envelope.project.repository_fingerprint.clone());
        Ok(normalized)
    }
}
