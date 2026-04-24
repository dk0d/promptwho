use promptwho_protocol::{
    EventEnvelope, EventPayload, IngestOpencodeEventsRequest, Message, MessageAddedPayload,
    MessagePartUpdatedProperties, MessageUpdatedProperties, OpencodeContext, OpencodeEvent,
    OpencodeEventEnvelope, Part, PluginSource, ProjectRef, ProtocolVersion, SessionInfoProperties,
    SessionRef, SessionStartedPayload, ToolCalledPayload, ToolResultPayload, ToolState,
};
use promptwho_storage::{
    AppendOutcome, GitSnapshot, Message as StoredMessage, Project, Session, StoreError,
    StoredEvent, ToolCall, ToolResult,
};
use promptwho_storage::{ConversationStore, EventStore, GitStore};
use serde_json::{Value, json};
use time::OffsetDateTime;
use tracing::warn;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum IngestError {
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error("session-scoped event missing session reference")]
    MissingSession,
    #[error("invalid opencode event payload: {0}")]
    InvalidOpencodeEvent(&'static str),
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

    pub async fn ingest_opencode_event(
        &self,
        envelope: OpencodeEventEnvelope,
    ) -> Result<Option<AppendOutcome>, IngestError> {
        let Some(normalized) = normalize_opencode_event(envelope)? else {
            return Ok(None);
        };

        self.ingest_protocol_event(normalized).await.map(Some)
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
                    .append_message(StoredMessage {
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

pub fn normalize_opencode_request(
    request: IngestOpencodeEventsRequest,
) -> Result<Vec<EventEnvelope>, IngestError> {
    let mut normalized = Vec::new();

    for event in request.events {
        if let Some(event) = normalize_opencode_event(event)? {
            normalized.push(event);
        }
    }

    Ok(normalized)
}

fn normalize_opencode_event(
    envelope: OpencodeEventEnvelope,
) -> Result<Option<EventEnvelope>, IngestError> {
    match envelope.event {
        OpencodeEvent::SessionCreated { properties } => {
            Ok(Some(normalize_session_created(envelope.context, properties)?))
        }
        OpencodeEvent::MessageUpdated { properties } => {
            normalize_message_updated(envelope.context, properties)
        }
        OpencodeEvent::MessagePartUpdated { properties } => {
            normalize_message_part_updated(envelope.context, properties)
        }
        _ => Ok(None),
    }
}

fn normalize_session_created(
    context: OpencodeContext,
    properties: SessionInfoProperties,
) -> Result<EventEnvelope, IngestError> {
    let raw_properties = properties.clone();
    let session_id = properties.session_id;
    let info = properties.info;
    let occurred_at = unix_millis_to_offset(info.time.created)?;
    let provider = info.version.clone();

    Ok(EventEnvelope {
        id: stable_event_id(&json!({ "type": "session.created", "properties": raw_properties }))?,
        version: ProtocolVersion::V1,
        occurred_at,
        project: project_ref(&context, Some(info.project_id.clone()), Some(info.directory.clone())),
        session: Some(SessionRef { id: session_id }),
        source: plugin_source(),
        payload: EventPayload::SessionStarted(SessionStartedPayload {
            provider,
            model: "unknown".to_string(),
            branch: None,
            head_commit: None,
            metadata: json!({
                "opencode": {
                    "type": "session.created",
                    "properties": raw_properties,
                },
            }),
        }),
    })
}

fn normalize_message_updated(
    context: OpencodeContext,
    properties: MessageUpdatedProperties,
) -> Result<Option<EventEnvelope>, IngestError> {
    let raw_properties = properties.clone();
    let session_id = properties.session_id;

    let (message_id, role, content, token_count, created_at, project_id, directory) = match properties
        .info
    {
        Message::User {
            id,
            session_id: _,
            time,
            summary,
            ..
        } => {
            let content = summary
                .and_then(|summary| summary.body.or(summary.title))
                .unwrap_or_default();

            (id, "user".to_string(), content, None, time.created, None, None)
        }
        Message::Assistant {
            id,
            time,
            tokens,
            ..
        } => {
            let token_count = tokens.total.and_then(|value| u32::try_from(value).ok());
            (id, "assistant".to_string(), String::new(), token_count, time.created, None, None)
        }
    };

    if content.is_empty() {
        return Ok(None);
    }

    Ok(Some(EventEnvelope {
        id: stable_event_id(&json!({ "type": "message.updated", "properties": raw_properties }))?,
        version: ProtocolVersion::V1,
        occurred_at: unix_millis_to_offset(created_at)?,
        project: project_ref(&context, project_id, directory),
        session: Some(SessionRef { id: session_id }),
        source: plugin_source(),
        payload: EventPayload::MessageAdded(MessageAddedPayload {
            message_id,
            role,
            content,
            token_count,
            metadata: json!({
                "opencode": {
                    "type": "message.updated",
                    "properties": raw_properties,
                },
            }),
        }),
    }))
}

fn normalize_message_part_updated(
    context: OpencodeContext,
    properties: MessagePartUpdatedProperties,
) -> Result<Option<EventEnvelope>, IngestError> {
    let raw_properties = properties.clone();
    let session_id = properties.session_id;
    let occurred_at = unix_millis_to_offset(properties.time)?;

    let Part::Tool {
        call_id,
        tool,
        state,
        ..
    } = properties.part
    else {
        return Ok(None);
    };

    match state {
        ToolState::Pending { input, .. } | ToolState::Running { input, .. } => Ok(Some(EventEnvelope {
            id: stable_event_id(&json!({ "type": "message.part.updated", "properties": raw_properties }))?,
            version: ProtocolVersion::V1,
            occurred_at,
            project: project_ref(&context, None, None),
            session: Some(SessionRef { id: session_id }),
            source: plugin_source(),
            payload: EventPayload::ToolCalled(ToolCalledPayload {
                tool_call_id: call_id,
                tool_name: tool,
                input,
                metadata: json!({
                    "opencode": {
                        "type": "message.part.updated",
                        "properties": raw_properties,
                    },
                }),
            }),
        })),
        ToolState::Completed { output, .. } => Ok(Some(EventEnvelope {
            id: stable_event_id(&json!({ "type": "message.part.updated", "properties": raw_properties }))?,
            version: ProtocolVersion::V1,
            occurred_at,
            project: project_ref(&context, None, None),
            session: Some(SessionRef { id: session_id }),
            source: plugin_source(),
            payload: EventPayload::ToolResult(ToolResultPayload {
                tool_call_id: call_id,
                success: true,
                output: json!(output),
                metadata: json!({
                    "opencode": {
                        "type": "message.part.updated",
                        "properties": raw_properties,
                    },
                }),
            }),
        })),
        ToolState::Error { error, .. } => Ok(Some(EventEnvelope {
            id: stable_event_id(&json!({ "type": "message.part.updated", "properties": raw_properties }))?,
            version: ProtocolVersion::V1,
            occurred_at,
            project: project_ref(&context, None, None),
            session: Some(SessionRef { id: session_id }),
            source: plugin_source(),
            payload: EventPayload::ToolResult(ToolResultPayload {
                tool_call_id: call_id,
                success: false,
                output: json!({ "error": error }),
                metadata: json!({
                    "opencode": {
                        "type": "message.part.updated",
                        "properties": raw_properties,
                    },
                }),
            }),
        })),
    }
}

fn unix_millis_to_offset(millis: i64) -> Result<OffsetDateTime, IngestError> {
    OffsetDateTime::from_unix_timestamp_nanos(i128::from(millis) * 1_000_000)
        .map_err(|_| IngestError::InvalidOpencodeEvent("invalid unix millis timestamp"))
}

fn project_ref(
    context: &OpencodeContext,
    project_id: Option<String>,
    directory: Option<String>,
) -> ProjectRef {
    let project_id = project_id.unwrap_or_else(|| context.project.id.clone());
    let root = directory.unwrap_or_else(|| context.directory.clone());

    ProjectRef {
        id: project_id,
        root,
        name: context.project.name.clone(),
    }
}

fn plugin_source() -> PluginSource {
    PluginSource {
        plugin: "opencode".to_string(),
        plugin_version: env!("CARGO_PKG_VERSION").to_string(),
        runtime: "bun".to_string(),
    }
}

fn stable_event_id(event: &Value) -> Result<Uuid, IngestError> {
    if let Some(id) = event.get("id").and_then(Value::as_str) {
        return Ok(Uuid::new_v5(&Uuid::NAMESPACE_OID, id.as_bytes()));
    }

    let serialized = serde_json::to_vec(event)
        .map_err(|_| IngestError::InvalidOpencodeEvent("event could not be serialized"))?;
    Ok(Uuid::new_v5(&Uuid::NAMESPACE_OID, &serialized))
}
