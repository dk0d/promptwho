use promptwho_protocol::*;
use promptwho_storage::*;
use promptwho_storage_surreal::{SurrealConfig, SurrealStore};
use serde_json::json;
use tempfile::TempDir;

use uuid::Uuid;

fn plus_seconds(ts: TimestampUtc, seconds: i64) -> TimestampUtc {
    ts + chrono::TimeDelta::seconds(seconds)
}

fn test_project_ref() -> ProjectRef {
    ProjectRef {
        id: "project-a".to_string(),
        root: "/tmp/project-a".to_string(),
        name: Some("project-a".to_string()),
        repository_fingerprint: None,
    }
}

fn test_session_ref() -> SessionRef {
    SessionRef {
        id: "session-a".to_string(),
    }
}

fn test_source() -> PluginSource {
    PluginSource {
        plugin: "opencode".to_string(),
        plugin_version: "0.1.0".to_string(),
        runtime: "bun".to_string(),
    }
}

async fn test_store() -> (TempDir, SurrealStore) {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let endpoint = format!(
        "surrealkv://{}",
        temp_dir.path().join("promptwho.db").display()
    );

    let store = SurrealStore::connect(SurrealConfig {
        endpoint,
        namespace: "promptwho_storage_test".to_string(),
        database: "promptwho_storage_test".to_string(),
        username: None,
        password: None,
        vector_enabled: false,
        sync_enabled: false,
    })
    .await
    .expect("store should connect");

    (temp_dir, store)
}

fn stored_event(
    id: Uuid,
    occurred_at: TimestampUtc,
    action: &str,
    project_id: &str,
    session_id: Option<&str>,
) -> StoredEvent {
    StoredEvent {
        id,
        project_id: project_id.to_string(),
        session_id: session_id.map(ToString::to_string),
        occurred_at,
        action: action.to_string(),
        envelope: EventEnvelope {
            id,
            version: ProtocolVersion::V1,
            occurred_at,
            project: ProjectRef {
                id: project_id.to_string(),
                root: format!("/tmp/{project_id}"),
                name: Some(project_id.to_string()),
                repository_fingerprint: None,
            },
            session: session_id.map(|session_id| SessionRef {
                id: session_id.to_string(),
            }),
            source: test_source(),
            payload: EventPayload::MessageUpdated(MessageUpdatedPayload {
                message_id: format!("message-{id}"),
                role: "user".to_string(),
                content: Some(format!("event {action}")),
                token_count: Some(1),
            }),
        },
        ingested_at: plus_seconds(occurred_at, 1),
    }
}

#[tokio::test]
async fn event_store_round_trips_and_filters_events() {
    let (_temp_dir, store) = test_store().await;
    let now = chrono::DateTime::UNIX_EPOCH;

    let session_event = stored_event(
        Uuid::new_v4(),
        now,
        "message.updated",
        "project-a",
        Some("session-a"),
    );
    let other_project_event = stored_event(
        Uuid::new_v4(),
        plus_seconds(now, 10),
        "tool.execute.before",
        "project-b",
        Some("session-b"),
    );

    let summary = store
        .append_events(vec![session_event.clone(), other_project_event.clone()])
        .await
        .expect("events should be appended");
    assert_eq!(summary.inserted, 2);
    assert_eq!(summary.skipped, 0);

    let fetched = store
        .get_event(session_event.id)
        .await
        .expect("event lookup should succeed")
        .expect("event should exist");
    assert_eq!(fetched.id, session_event.id);
    assert_eq!(fetched.project_id, "project-a");

    let filtered = store
        .list_events(Some(EventQuery {
            project_id: Some("project-a".to_string()),
            session_id: Some("session-a".to_string()),
            action: Some("message.updated".to_string()),
            occurred_after: Some(plus_seconds(now, -1)),
            occurred_before: Some(plus_seconds(now, 1)),
            limit: Some(10),
            ..Default::default()
        }))
        .await
        .expect("filtered event listing should succeed");
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].id, session_event.id);

    let limited = store
        .list_events(Some(EventQuery {
            limit: Some(1),
            ..Default::default()
        }))
        .await
        .expect("limited event listing should succeed");
    assert_eq!(limited.len(), 1);
    assert_eq!(limited[0].id, session_event.id);
}

#[tokio::test]
async fn event_store_treats_duplicate_event_ids_as_skipped() {
    let (_temp_dir, store) = test_store().await;
    let now = chrono::DateTime::UNIX_EPOCH;
    let event = stored_event(
        Uuid::new_v4(),
        now,
        "message.updated",
        "project-a",
        Some("session-a"),
    );

    let first = store
        .append_event(event.clone())
        .await
        .expect("first append should succeed");
    assert!(first.inserted);

    let second = store
        .append_event(event.clone())
        .await
        .expect("duplicate append should be treated as a skip");
    assert!(!second.inserted);

    let events = store
        .list_events(None)
        .await
        .expect("event listing should succeed");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].id, event.id);
}

#[tokio::test]
async fn conversation_store_round_trips_sessions_and_messages() {
    let (_temp_dir, store) = test_store().await;
    let started_at = chrono::DateTime::UNIX_EPOCH;

    store
        .upsert_project(Project {
            id: "project-a".to_string(),
            root: "/tmp/project-a".to_string(),
            name: Some("project-a".to_string()),
            repository_fingerprint: None,
            created_at: started_at,
        })
        .await
        .expect("project upsert should succeed");

    store
        .upsert_session(Session {
            id: "session-a".to_string(),
            project_id: "project-a".to_string(),
            provider: "openai".to_string(),
            model: "gpt-5.4".to_string(),
            started_on_branch: Some("main".to_string()),
            started_on_head: Some("abc123".to_string()),
            started_at,
            ended_at: None,
            metadata: json!({"editor": "vscode"}),
        })
        .await
        .expect("session upsert should succeed");

    store
        .append_message(Message {
            id: "message-1".to_string(),
            session_id: "session-a".to_string(),
            role: "user".to_string(),
            content: "first message".to_string(),
            token_count: Some(10),
            created_at: started_at,
            metadata: json!({}),
        })
        .await
        .expect("first message append should succeed");

    store
        .append_message(Message {
            id: "message-2".to_string(),
            session_id: "session-a".to_string(),
            role: "assistant".to_string(),
            content: "second message".to_string(),
            token_count: Some(20),
            created_at: plus_seconds(started_at, 5),
            metadata: json!({}),
        })
        .await
        .expect("second message append should succeed");

    let session = store
        .get_session("session-a".to_string())
        .await
        .expect("session lookup should succeed")
        .expect("session should exist");
    assert_eq!(session.provider, "openai");
    assert_eq!(session.started_on_branch.as_deref(), Some("main"));

    let sessions = store
        .list_sessions(Some(SessionQuery {
            project_id: Some("project-a".to_string()),
            started_after: Some(plus_seconds(started_at, -1)),
            started_before: Some(plus_seconds(started_at, 1)),
            limit: Some(10),
            ..Default::default()
        }))
        .await
        .expect("session listing should succeed");
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].id, "session-a");
    assert_eq!(sessions[0].model, "gpt-5.4");

    let messages = store
        .list_messages("session-a".to_string(), None)
        .await
        .expect("message listing should succeed");
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].id, "message-1");
    assert_eq!(messages[1].id, "message-2");
}

#[tokio::test]
async fn direct_projection_writes_accept_tool_and_git_records() {
    let (_temp_dir, store) = test_store().await;
    let created_at = chrono::DateTime::UNIX_EPOCH;
    let project = test_project_ref();
    let session = test_session_ref();

    store
        .upsert_project(Project {
            id: project.id.clone(),
            root: project.root,
            name: project.name,
            repository_fingerprint: project.repository_fingerprint,
            created_at,
        })
        .await
        .expect("project upsert should succeed");

    store
        .upsert_session(Session {
            id: session.id.clone(),
            project_id: project.id.clone(),
            provider: "openai".to_string(),
            model: "gpt-5.4".to_string(),
            started_on_branch: Some("main".to_string()),
            started_on_head: Some("abc123".to_string()),
            started_at: created_at,
            ended_at: None,
            metadata: json!({}),
        })
        .await
        .expect("session upsert should succeed");

    store
        .record_tool_call(ToolCall {
            id: "tool-call-1".to_string(),
            session_id: session.id.clone(),
            tool_name: "bash".to_string(),
            input: json!({"command": "git status"}),
            created_at,
            completed_at: None,
            success: None,
            output: None,
            metadata: json!({"source": "test"}),
        })
        .await
        .expect("tool call record should succeed");

    store
        .complete_tool_call(
            "tool-call-1".to_string(),
            true,
            json!({"stdout": "clean"}),
            plus_seconds(created_at, 1),
            json!({"exit_code": 0}),
        )
        .await
        .expect("tool result record should succeed");

    store
        .record_git_snapshot(GitSnapshot {
            id: Uuid::new_v4(),
            project_id: project.id,
            session_id: Some(session.id),
            branch: Some("main".to_string()),
            head_commit: Some("abc123".to_string()),
            dirty: true,
            staged_files: vec!["src/lib.rs".to_string()],
            unstaged_files: vec!["README.md".to_string()],
            created_at: plus_seconds(created_at, 2),
        })
        .await
        .expect("git snapshot record should succeed");

    let stored_session = store
        .get_session("session-a".to_string())
        .await
        .expect("session lookup should succeed")
        .expect("session should still exist");
    assert_eq!(stored_session.id, "session-a");
}

#[tokio::test]
async fn git_store_round_trips_commits_files_and_hunks() {
    let (_temp_dir, store) = test_store().await;
    let committed_at = chrono::DateTime::UNIX_EPOCH;
    let commit_oid = "abc123".to_string();
    let first_hunk_id = Uuid::new_v4();
    let second_hunk_id = Uuid::new_v4();

    store
        .record_commit(
            GitCommit {
                oid: commit_oid.clone(),
                project_id: "project-a".to_string(),
                parent_oid: Some("def456".to_string()),
                author_name: Some("Daniel".to_string()),
                author_email: Some("daniel@example.com".to_string()),
                message: "Implement commit hunk persistence".to_string(),
                committed_at,
            },
            vec![
                GitCommitFile {
                    id: format!("{commit_oid}::src/lib.rs"),
                    commit_oid: commit_oid.clone(),
                    path: "src/lib.rs".to_string(),
                    old_path: None,
                    change_kind: "modified".to_string(),
                },
                GitCommitFile {
                    id: format!("{commit_oid}::src/old.rs"),
                    commit_oid: commit_oid.clone(),
                    path: "src/old.rs".to_string(),
                    old_path: Some("src/older.rs".to_string()),
                    change_kind: "renamed".to_string(),
                },
            ],
            vec![
                GitCommitHunk {
                    id: first_hunk_id,
                    commit_file_id: format!("{commit_oid}::src/lib.rs"),
                    file_path: "src/lib.rs".to_string(),
                    old_start: 10,
                    old_lines: 2,
                    new_start: 10,
                    new_lines: 4,
                    hunk_header: Some("fn build_router".to_string()),
                    added_line_count: 3,
                    removed_line_count: 1,
                    context_before_hash: Some("before-a".to_string()),
                    context_after_hash: Some("after-a".to_string()),
                    added_lines_fingerprint: Some("add-a".to_string()),
                    removed_lines_fingerprint: Some("remove-a".to_string()),
                },
                GitCommitHunk {
                    id: second_hunk_id,
                    commit_file_id: format!("{commit_oid}::src/lib.rs"),
                    file_path: "src/lib.rs".to_string(),
                    old_start: 30,
                    old_lines: 1,
                    new_start: 32,
                    new_lines: 2,
                    hunk_header: Some("fn ingest_events".to_string()),
                    added_line_count: 2,
                    removed_line_count: 1,
                    context_before_hash: Some("before-b".to_string()),
                    context_after_hash: Some("after-b".to_string()),
                    added_lines_fingerprint: Some("add-b".to_string()),
                    removed_lines_fingerprint: Some("remove-b".to_string()),
                },
            ],
        )
        .await
        .expect("commit record should succeed");

    let stored_commit = store
        .get_commit(commit_oid.clone())
        .await
        .expect("commit lookup should succeed")
        .expect("commit should exist");
    assert_eq!(stored_commit.project_id, "project-a");
    assert_eq!(stored_commit.parent_oid.as_deref(), Some("def456"));

    let commits = store
        .list_commits_for_project(
            "project-a".to_string(),
            Some(promptwho_storage::CommitQuery {
                committed_after: Some(plus_seconds(committed_at, -1)),
                committed_before: Some(plus_seconds(committed_at, 1)),
                limit: Some(10),
                ..Default::default()
            }),
        )
        .await
        .expect("commit listing should succeed");
    assert_eq!(commits.len(), 1);
    assert_eq!(commits[0].oid, commit_oid);

    let hunks = store
        .list_commit_hunks("abc123".to_string(), None)
        .await
        .expect("commit hunk listing should succeed");
    assert_eq!(hunks.len(), 2);
    assert_eq!(hunks[0].id, first_hunk_id);
    assert_eq!(hunks[1].id, second_hunk_id);

    let history = store
        .list_file_history("project-a".to_string(), "src/lib.rs", None)
        .await
        .expect("file history should succeed");
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].commit_oid, "abc123");
    assert_eq!(history[0].message, "Implement commit hunk persistence");

    let renamed_history = store
        .list_file_history("project-a".to_string(), "src/older.rs", None)
        .await
        .expect("renamed file history should succeed");
    assert_eq!(renamed_history.len(), 1);
    assert_eq!(renamed_history[0].commit_oid, "abc123");
}

#[tokio::test]
async fn stored_events_can_round_trip_real_protocol_payloads() {
    let (_temp_dir, store) = test_store().await;
    let occurred_at = chrono::DateTime::UNIX_EPOCH;
    let id = Uuid::new_v4();

    let event = StoredEvent {
        id,
        project_id: "project-a".to_string(),
        session_id: Some("session-a".to_string()),
        occurred_at,
        action: "session.created".to_string(),
        envelope: EventEnvelope {
            id,
            version: ProtocolVersion::V1,
            occurred_at,
            project: test_project_ref(),
            session: Some(test_session_ref()),
            source: test_source(),
            payload: EventPayload::SessionCreated,
        },
        ingested_at: plus_seconds(occurred_at, 1),
    };

    let outcome = store
        .append_event(event.clone())
        .await
        .expect("event append should succeed");
    assert!(outcome.inserted);

    let fetched = store
        .get_event(id)
        .await
        .expect("event lookup should succeed")
        .expect("event should exist");

    match fetched.envelope.payload {
        EventPayload::SessionCreated => {}
        payload => panic!("unexpected payload stored: {payload:?}"),
    }

    let git_event = StoredEvent {
        id: Uuid::new_v4(),
        project_id: "project-a".to_string(),
        session_id: Some("session-a".to_string()),
        occurred_at: plus_seconds(occurred_at, 10),
        action: "git.snapshot".to_string(),
        envelope: EventEnvelope {
            id: Uuid::new_v4(),
            version: ProtocolVersion::V1,
            occurred_at: plus_seconds(occurred_at, 10),
            project: test_project_ref(),
            session: Some(test_session_ref()),
            source: test_source(),
            payload: EventPayload::GitSnapshot(GitSnapshotPayload {
                branch: Some("main".to_string()),
                head_commit: Some("abc123".to_string()),
                dirty: true,
                staged_files: vec!["src/lib.rs".to_string()],
                unstaged_files: vec!["README.md".to_string()],
            }),
        },
        ingested_at: plus_seconds(occurred_at, 11),
    };

    store
        .append_event(git_event.clone())
        .await
        .expect("git event append should succeed");

    let listed = store
        .list_events(Some(EventQuery {
            action: Some("git.snapshot".to_string()),
            ..Default::default()
        }))
        .await
        .expect("git event listing should succeed");
    assert_eq!(listed.len(), 1);

    match &listed[0].envelope.payload {
        EventPayload::GitSnapshot(payload) => {
            assert!(payload.dirty);
            assert_eq!(payload.staged_files, vec!["src/lib.rs".to_string()]);
        }
        payload => panic!("unexpected payload listed: {payload:?}"),
    }
}
