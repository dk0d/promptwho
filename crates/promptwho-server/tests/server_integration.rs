use std::sync::Arc;

use axum_test::{TestResponse, TestServer, http::StatusCode};
use promptwho_protocol::{
    ErrorResponse, EventEnvelope, EventPayload, GitSnapshotPayload, IngestEventsRequest,
    IngestEventsResponse, MessageAddedPayload, PluginSource, ProjectRef, ProtocolVersion,
    SessionRef, SessionStartedPayload, ToolCalledPayload, ToolResultPayload,
};
use promptwho_server::{AppState, build_router};
use promptwho_storage::{ConversationStore, EventQuery, EventStore};
use promptwho_storage_surreal::{SurrealConfig, SurrealStore};
use serde_json::json;
use tempfile::TempDir;
use time::OffsetDateTime;
use uuid::Uuid;

fn test_project() -> ProjectRef {
    ProjectRef {
        id: "project-test".to_string(),
        root: "/tmp/promptwho-test".to_string(),
        name: Some("promptwho-test".to_string()),
    }
}

fn test_session() -> SessionRef {
    SessionRef {
        id: "session-test".to_string(),
    }
}

fn test_source() -> PluginSource {
    PluginSource {
        plugin: "opencode".to_string(),
        plugin_version: "0.1.0".to_string(),
        runtime: "bun".to_string(),
    }
}

async fn test_store() -> (TempDir, Arc<SurrealStore>) {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let endpoint = format!(
        "surrealkv://{}",
        temp_dir.path().join("promptwho.db").display()
    );

    let store = SurrealStore::connect(SurrealConfig {
        endpoint,
        namespace: "promptwho_test".to_string(),
        database: "promptwho_test".to_string(),
        username: None,
        password: None,
        vector_enabled: false,
        sync_enabled: false,
    })
    .await
    .expect("store should connect");

    (temp_dir, Arc::new(store))
}

#[tokio::test]
async fn ingest_events_persists_surreal_records() {
    let (_temp_dir, store) = test_store().await;
    let state = AppState {
        store: store.clone(),
    };
    let server = TestServer::new(build_router(state));

    let request_id = Uuid::new_v4();
    let session_started_id = Uuid::new_v4();
    let message_added_id = Uuid::new_v4();
    let tool_called_id = Uuid::new_v4();
    let tool_result_id = Uuid::new_v4();
    let git_snapshot_id = Uuid::new_v4();

    let project = test_project();
    let session = test_session();
    let source = test_source();
    let occurred_at = OffsetDateTime::UNIX_EPOCH;

    let response: TestResponse = server
        .post("/v1/events")
        .msgpack(&IngestEventsRequest {
            request_id,
            events: vec![
                EventEnvelope {
                    id: session_started_id,
                    version: ProtocolVersion::V1,
                    occurred_at,
                    project: project.clone(),
                    session: Some(session.clone()),
                    source: source.clone(),
                    payload: EventPayload::SessionStarted(SessionStartedPayload {
                        provider: "openai".to_string(),
                        model: "gpt-5.4".to_string(),
                        branch: Some("main".to_string()),
                        head_commit: Some("abc123".to_string()),
                        metadata: json!({"editor": "vscode"}),
                    }),
                },
                EventEnvelope {
                    id: message_added_id,
                    version: ProtocolVersion::V1,
                    occurred_at,
                    project: project.clone(),
                    session: Some(session.clone()),
                    source: source.clone(),
                    payload: EventPayload::MessageAdded(MessageAddedPayload {
                        message_id: "message-1".to_string(),
                        role: "user".to_string(),
                        content: "hello from axum_test".to_string(),
                        token_count: Some(12),
                        metadata: json!({"kind": "prompt"}),
                    }),
                },
                EventEnvelope {
                    id: tool_called_id,
                    version: ProtocolVersion::V1,
                    occurred_at,
                    project: project.clone(),
                    session: Some(session.clone()),
                    source: source.clone(),
                    payload: EventPayload::ToolCalled(ToolCalledPayload {
                        tool_call_id: "tool-call-1".to_string(),
                        tool_name: "bash".to_string(),
                        input: json!({"command": "git status"}),
                        metadata: json!({"source": "test"}),
                    }),
                },
                EventEnvelope {
                    id: tool_result_id,
                    version: ProtocolVersion::V1,
                    occurred_at,
                    project: project.clone(),
                    session: Some(session.clone()),
                    source: source.clone(),
                    payload: EventPayload::ToolResult(ToolResultPayload {
                        tool_call_id: "tool-call-1".to_string(),
                        success: true,
                        output: json!({"stdout": "On branch main"}),
                        metadata: json!({"exit_code": 0}),
                    }),
                },
                EventEnvelope {
                    id: git_snapshot_id,
                    version: ProtocolVersion::V1,
                    occurred_at,
                    project: project.clone(),
                    session: Some(session.clone()),
                    source,
                    payload: EventPayload::GitSnapshot(GitSnapshotPayload {
                        branch: Some("main".to_string()),
                        head_commit: Some("abc123".to_string()),
                        dirty: true,
                        staged_files: vec!["src/app.rs".to_string()],
                        unstaged_files: vec!["README.md".to_string()],
                    }),
                },
            ],
        })
        .await;

    response.assert_status(StatusCode::ACCEPTED);
    let body = response.msgpack::<IngestEventsResponse>();
    assert_eq!(body.request_id, request_id);
    assert_eq!(body.accepted, 5);
    assert_eq!(body.rejected, 0);

    let stored_event = store
        .get_event(session_started_id)
        .await
        .expect("event lookup should succeed")
        .expect("session started event should exist");
    assert_eq!(stored_event.project_id, project.id);
    assert_eq!(stored_event.action, "session.started");

    let all_events = store
        .list_events(EventQuery {
            project_id: Some("project-test".to_string()),
            ..Default::default()
        })
        .await
        .expect("event listing should succeed");
    assert_eq!(all_events.len(), 5);
    assert!(all_events.iter().any(|event| event.action == "tool.called"));
    assert!(all_events.iter().any(|event| event.action == "tool.result"));
    assert!(
        all_events
            .iter()
            .any(|event| event.action == "git.snapshot")
    );

    let stored_session = store
        .get_session("session-test".to_string())
        .await
        .expect("session lookup should succeed")
        .expect("session should exist");
    assert_eq!(stored_session.provider, "openai");
    assert_eq!(stored_session.model, "gpt-5.4");
    assert_eq!(stored_session.branch.as_deref(), Some("main"));

    let messages = store
        .list_messages("session-test".to_string())
        .await
        .expect("message listing should succeed");
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].content, "hello from axum_test");
    assert_eq!(messages[0].role, "user");
}

#[tokio::test]
async fn ingest_events_rejects_invalid_msgpack() {
    let (_temp_dir, store) = test_store().await;
    let state = AppState {
        store: store.clone(),
    };
    let server = TestServer::new(build_router(state));

    let response: TestResponse = server
        .post("/v1/events")
        .bytes(vec![0xc1].into())
        .content_type("application/msgpack")
        .expect_failure()
        .await;

    response.assert_status_bad_request();
    let body = response.msgpack::<ErrorResponse>();
    assert_eq!(body.code, "invalid_msgpack");
}

#[tokio::test]
async fn ingest_events_returns_json_errors_when_requested() {
    let (_temp_dir, store) = test_store().await;
    let state = AppState {
        store: store.clone(),
    };
    let server = TestServer::new(build_router(state));

    let response: TestResponse = server
        .post("/v1/events")
        .bytes(vec![0xc1].into())
        .content_type("application/msgpack")
        .add_header("accept", "application/json")
        .expect_failure()
        .await;

    response.assert_status_bad_request();
    let body = response.json::<ErrorResponse>();
    assert_eq!(body.code, "invalid_msgpack");
}
