use std::sync::Arc;

use axum_test::{TestResponse, TestServer, http::StatusCode};
use promptwho_protocol::*;
use promptwho_server::{AppState, build_router};
use promptwho_storage::{ConversationStore, EventQuery, EventStore};
use promptwho_storage_surreal::{SurrealConfig, SurrealStore};
use serde_json::json;
use tempfile::TempDir;
use uuid::Uuid;

#[derive(Debug, serde::Deserialize)]
struct DashboardProjectResponse {
    id: String,
    name: Option<String>,
    root: String,
}

fn test_project() -> ProjectRef {
    ProjectRef {
        id: "project-test".to_string(),
        root: "/tmp/promptwho-test".to_string(),
        name: Some("promptwho-test".to_string()),
        repository_fingerprint: None,
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
    let occurred_at = chrono::DateTime::UNIX_EPOCH;

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
                    payload: EventPayload::SessionCreated,
                },
                EventEnvelope {
                    id: message_added_id,
                    version: ProtocolVersion::V1,
                    occurred_at,
                    project: project.clone(),
                    session: Some(session.clone()),
                    source: source.clone(),
                    payload: EventPayload::MessageUpdated(MessageUpdatedPayload {
                        message_id: "message-1".to_string(),
                        role: "user".to_string(),
                        content: Some("hello from axum_test".to_string()),
                        token_count: Some(12),
                    }),
                },
                EventEnvelope {
                    id: tool_called_id,
                    version: ProtocolVersion::V1,
                    occurred_at,
                    project: project.clone(),
                    session: Some(session.clone()),
                    source: source.clone(),
                    payload: EventPayload::ToolExecuteBefore(ToolExecuteBeforePayload {
                        tool_call_id: "tool-call-1".to_string(),
                        tool_name: "bash".to_string(),
                        input: json!({"command": "git status"}),
                    }),
                },
                EventEnvelope {
                    id: tool_result_id,
                    version: ProtocolVersion::V1,
                    occurred_at,
                    project: project.clone(),
                    session: Some(session.clone()),
                    source: source.clone(),
                    payload: EventPayload::ToolExecuteAfter(ToolExecuteAfterPayload {
                        tool_call_id: "tool-call-1".to_string(),
                        tool_name: "bash".to_string(),
                        success: true,
                        output: json!({"stdout": "On branch main"}),
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
    assert_eq!(stored_event.action, "session.created");

    let all_events = store
        .list_events(Some(EventQuery {
            project_id: Some("project-test".to_string()),
            ..Default::default()
        }))
        .await
        .expect("event listing should succeed");
    assert_eq!(all_events.len(), 5);
    assert!(
        all_events
            .iter()
            .any(|event| event.action == "tool.execute.before")
    );
    assert!(
        all_events
            .iter()
            .any(|event| event.action == "tool.execute.after")
    );
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
    assert_eq!(stored_session.provider, "unknown");
    assert_eq!(stored_session.model, "unknown");
    assert_eq!(stored_session.started_on_branch, None);

    let messages = store
        .list_messages("session-test".to_string(), None)
        .await
        .expect("message listing should succeed");
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].content, "hello from axum_test");
    assert_eq!(messages[0].role, "user");
}

#[tokio::test]
async fn ingest_events_merge_streamed_message_parts() {
    let (_temp_dir, store) = test_store().await;
    let state = AppState {
        store: store.clone(),
    };
    let server = TestServer::new(build_router(state));

    let request_id = Uuid::new_v4();
    let project = test_project();
    let session = test_session();
    let source = test_source();
    let occurred_at = chrono::DateTime::UNIX_EPOCH;

    let response: TestResponse = server
        .post("/v1/events")
        .msgpack(&IngestEventsRequest {
            request_id,
            events: vec![
                EventEnvelope {
                    id: Uuid::new_v4(),
                    version: ProtocolVersion::V1,
                    occurred_at,
                    project: project.clone(),
                    session: Some(session.clone()),
                    source: source.clone(),
                    payload: EventPayload::SessionCreated,
                },
                EventEnvelope {
                    id: Uuid::new_v4(),
                    version: ProtocolVersion::V1,
                    occurred_at,
                    project: project.clone(),
                    session: Some(session.clone()),
                    source: source.clone(),
                    payload: EventPayload::MessagePartUpdated(MessagePartUpdatedPayload {
                        message_id: "message-2".to_string(),
                        part_id: "part-1".to_string(),
                        part_type: "text".to_string(),
                        text: Some("Hel".to_string()),
                    }),
                },
                EventEnvelope {
                    id: Uuid::new_v4(),
                    version: ProtocolVersion::V1,
                    occurred_at,
                    project: project.clone(),
                    session: Some(session.clone()),
                    source: source.clone(),
                    payload: EventPayload::MessagePartUpdated(MessagePartUpdatedPayload {
                        message_id: "message-2".to_string(),
                        part_id: "part-1".to_string(),
                        part_type: "text".to_string(),
                        text: Some("Hello".to_string()),
                    }),
                },
                EventEnvelope {
                    id: Uuid::new_v4(),
                    version: ProtocolVersion::V1,
                    occurred_at,
                    project,
                    session: Some(session),
                    source,
                    payload: EventPayload::MessagePartUpdated(MessagePartUpdatedPayload {
                        message_id: "message-2".to_string(),
                        part_id: "part-2".to_string(),
                        part_type: "text".to_string(),
                        text: Some(" world".to_string()),
                    }),
                },
            ],
        })
        .await;

    response.assert_status(StatusCode::ACCEPTED);
    let body = response.msgpack::<IngestEventsResponse>();
    assert_eq!(body.request_id, request_id);
    assert_eq!(body.accepted, 4);
    assert_eq!(body.rejected, 0);

    let messages = store
        .list_messages("session-test".to_string(), None)
        .await
        .expect("message listing should succeed");
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].id, "message-2");
    assert_eq!(messages[0].role, "assistant");
    assert_eq!(messages[0].content, "Hello world");
}

#[tokio::test]
async fn list_projects_returns_persisted_projects() {
    let (_temp_dir, store) = test_store().await;
    store
        .upsert_project(promptwho_storage::Project {
            id: "project-test".to_string(),
            root: "/tmp/promptwho-test".to_string(),
            name: Some("promptwho-test".to_string()),
            repository_fingerprint: None,
            created_at: chrono::DateTime::UNIX_EPOCH,
        })
        .await
        .expect("project upsert should succeed");

    let state = AppState {
        store: store.clone(),
    };
    let server = TestServer::new(build_router(state));

    let response: TestResponse = server.get("/v1/projects").await;

    response.assert_status_ok();
    let body = response.json::<Vec<DashboardProjectResponse>>();
    assert_eq!(body.len(), 1);
    assert_eq!(body[0].id, "project-test");
    assert_eq!(body[0].name.as_deref(), Some("promptwho-test"));
    assert_eq!(body[0].root, "/tmp/promptwho-test");
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

#[tokio::test]
async fn ingest_events_rejects_valid_msgpack_with_wrong_shape() {
    let (_temp_dir, store) = test_store().await;
    let state = AppState {
        store: store.clone(),
    };
    let server = TestServer::new(build_router(state));

    let body = rmp_serde::encode::to_vec_named(&serde_json::json!({
        "flavor": "opencode",
        "request_id": Uuid::new_v4(),
        "events": [
            {
                "context": {
                    "project": {
                        "id": "project-test",
                        "worktree": "/tmp/promptwho-test",
                        "vcs": "git"
                    },
                    "directory": "/tmp/promptwho-test",
                    "worktree": "/tmp/promptwho-test"
                },
                "event": {
                    "type": "message.updated",
                    "properties": {
                        "sessionID": "session-test"
                    }
                }
            }
        ]
    }))
    .expect("msgpack body should serialize");

    let response: TestResponse = server
        .post("/v1/events")
        .bytes(body.into())
        .content_type("application/msgpack")
        .expect_failure()
        .await;

    response.assert_status_bad_request();
    let error = response.msgpack::<ErrorResponse>();
    assert_eq!(error.code, "invalid_msgpack");
    assert!(
        error
            .message
            .contains("unknown variant `opencode`, expected `core`")
            || error.message.contains("missing field")
            || error.message.contains("wrong msgpack marker"),
        "unexpected error message: {}",
        error.message
    );
}
