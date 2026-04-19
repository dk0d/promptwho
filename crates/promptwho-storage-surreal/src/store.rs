use async_trait::async_trait;
use promptwho_storage::capabilities::{SupportsSyncMetadata, SupportsVectors};
use promptwho_storage::{
    AppendOutcome, AppendSummary, AttributionStore, ChangeStore, CodeLocation,
    CommitAttributionQuery, CommitQuery, CommitSessionSummary, ConversationStore, EmbeddingRecord,
    EventQuery, EventStore, ExecutionTrace, GitCommit, GitCommitFile, GitCommitHunk,
    GitFileHistoryRow, GitOid, GitSnapshot, GitStore, Message, PatchAttribution, Project,
    ProjectId, SearchResult, SearchResults, SearchStore, Session, SessionChangeHunk,
    SessionCodeChange, SessionId, SessionQuery, SessionSummary, StoreError, StoredEvent,
    TextSearchQuery, ToolCall, ToolResult, TraceFrame, TraceLinkQuery, TraceStore, VectorHit,
    VectorSearchQuery, VectorSearchStore,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use surrealdb::{
    Surreal,
    engine::any::{Any, connect},
    opt::auth::Root,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SurrealRecord<T> {
    value: T,
}

#[derive(Debug, Clone)]
pub struct SurrealConfig {
    pub endpoint: String,
    pub namespace: String,
    pub database: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub vector_enabled: bool,
    pub sync_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct SurrealStore {
    db: Surreal<Any>,
    vector_enabled: bool,
    sync_enabled: bool,
}

impl SurrealStore {
    pub async fn connect(config: SurrealConfig) -> Result<Self, StoreError> {
        let db: Surreal<Any> = connect(config.endpoint.as_str())
            .await
            .map_err(|err| StoreError::Message(err.to_string()))?;

        if let (Some(username), Some(password)) =
            (config.username.as_deref(), config.password.as_deref())
        {
            db.signin(Root { username, password })
                .await
                .map_err(|err| StoreError::Message(err.to_string()))?;
        }

        db.use_ns(&config.namespace)
            .use_db(&config.database)
            .await
            .map_err(|err| StoreError::Message(err.to_string()))?;

        let store = Self {
            db,
            vector_enabled: config.vector_enabled,
            sync_enabled: config.sync_enabled,
        };

        store.initialize().await?;
        Ok(store)
    }

    async fn initialize(&self) -> Result<(), StoreError> {
        self.db
            .query(
                r#"
                DEFINE TABLE raw_events SCHEMALESS;
                DEFINE TABLE projects SCHEMALESS;
                DEFINE TABLE sessions SCHEMALESS;
                DEFINE TABLE messages SCHEMALESS;
                DEFINE TABLE tool_calls SCHEMALESS;
                DEFINE TABLE tool_results SCHEMALESS;
                DEFINE TABLE git_snapshots SCHEMALESS;
                DEFINE TABLE git_commits SCHEMALESS;
                DEFINE TABLE git_commit_files SCHEMALESS;
                DEFINE TABLE git_commit_hunks SCHEMALESS;
                DEFINE TABLE session_code_changes SCHEMALESS;
                DEFINE TABLE session_change_hunks SCHEMALESS;
                DEFINE TABLE execution_traces SCHEMALESS;
                DEFINE TABLE trace_frames SCHEMALESS;
                DEFINE TABLE code_locations SCHEMALESS;
                DEFINE TABLE patch_attributions SCHEMALESS;
                DEFINE TABLE commit_session_summaries SCHEMALESS;

                DEFINE INDEX raw_events_id ON TABLE raw_events FIELDS id UNIQUE;
                DEFINE INDEX projects_id ON TABLE projects FIELDS id UNIQUE;
                DEFINE INDEX sessions_id ON TABLE sessions FIELDS id UNIQUE;
                DEFINE INDEX messages_id ON TABLE messages FIELDS id UNIQUE;
                DEFINE INDEX tool_calls_id ON TABLE tool_calls FIELDS id UNIQUE;
                DEFINE INDEX git_commits_oid ON TABLE git_commits FIELDS oid UNIQUE;
                DEFINE INDEX git_commit_hunks_id ON TABLE git_commit_hunks FIELDS id UNIQUE;
                DEFINE INDEX session_code_changes_id ON TABLE session_code_changes FIELDS id UNIQUE;
                DEFINE INDEX session_change_hunks_id ON TABLE session_change_hunks FIELDS id UNIQUE;
                DEFINE INDEX trace_frames_id ON TABLE trace_frames FIELDS id UNIQUE;
                DEFINE INDEX code_locations_id ON TABLE code_locations FIELDS id UNIQUE;
                DEFINE INDEX patch_attributions_id ON TABLE patch_attributions FIELDS id UNIQUE;
                DEFINE INDEX commit_session_summaries_id ON TABLE commit_session_summaries FIELDS id UNIQUE;
                "#,
            )
            .await
            .map_err(|err| StoreError::Message(err.to_string()))?;

        Ok(())
    }

    async fn upsert_record<T>(&self, table: &str, id: &str, value: T) -> Result<(), StoreError>
    where
        T: Serialize + DeserializeOwned + 'static,
    {
        let _: Option<SurrealRecord<T>> = self
            .db
            .upsert((table, id))
            .content(SurrealRecord { value })
            .await
            .map_err(|err| StoreError::Message(err.to_string()))?;

        Ok(())
    }

    async fn select_record<T>(&self, table: &str, id: &str) -> Result<Option<T>, StoreError>
    where
        T: DeserializeOwned,
    {
        let record: Option<SurrealRecord<T>> = self
            .db
            .select((table, id))
            .await
            .map_err(|err| StoreError::Message(err.to_string()))?;

        Ok(record.map(|record| record.value))
    }

    async fn select_table<T>(&self, table: &str) -> Result<Vec<T>, StoreError>
    where
        T: DeserializeOwned,
    {
        let records: Vec<SurrealRecord<T>> = self
            .db
            .select(table)
            .await
            .map_err(|err| StoreError::Message(err.to_string()))?;

        Ok(records.into_iter().map(|record| record.value).collect())
    }

    fn table_scoped_id(parts: &[&str]) -> String {
        parts.join("::")
    }

    fn todo<T>() -> Result<T, StoreError> {
        Err(StoreError::Message(
            "surreal storage methods are scaffolded but not yet implemented".to_string(),
        ))
    }
}

impl SupportsVectors for SurrealStore {
    fn vector_enabled(&self) -> bool {
        self.vector_enabled
    }
}

impl SupportsSyncMetadata for SurrealStore {
    fn sync_enabled(&self) -> bool {
        self.sync_enabled
    }
}

#[async_trait]
impl EventStore for SurrealStore {
    async fn append_event(&self, event: StoredEvent) -> Result<AppendOutcome, StoreError> {
        let event_id = event.id.to_string();
        let created: Option<SurrealRecord<StoredEvent>> = self
            .db
            .create(("raw_events", event_id.as_str()))
            .content(SurrealRecord { value: event })
            .await
            .map_err(|err| StoreError::Message(err.to_string()))?;

        Ok(AppendOutcome {
            inserted: created.is_some(),
        })
    }

    async fn append_events(&self, events: Vec<StoredEvent>) -> Result<AppendSummary, StoreError> {
        let mut inserted = 0usize;
        let mut skipped = 0usize;

        for event in events {
            let outcome = self.append_event(event).await?;
            if outcome.inserted {
                inserted += 1;
            } else {
                skipped += 1;
            }
        }

        Ok(AppendSummary { inserted, skipped })
    }

    async fn get_event(&self, id: uuid::Uuid) -> Result<Option<StoredEvent>, StoreError> {
        self.select_record("raw_events", &id.to_string()).await
    }

    async fn list_events(&self, query: EventQuery) -> Result<Vec<StoredEvent>, StoreError> {
        let mut events = self.select_table::<StoredEvent>("raw_events").await?;

        events.retain(|event| {
            query
                .project_id
                .as_ref()
                .is_none_or(|project_id| &event.project_id == project_id)
                && query
                    .session_id
                    .as_ref()
                    .is_none_or(|session_id| event.session_id.as_ref() == Some(session_id))
                && query
                    .action
                    .as_ref()
                    .is_none_or(|action| &event.action == action)
                && query
                    .occurred_after
                    .is_none_or(|occurred_after| event.occurred_at >= occurred_after)
                && query
                    .occurred_before
                    .is_none_or(|occurred_before| event.occurred_at <= occurred_before)
        });

        events.sort_by(|left, right| {
            left.occurred_at
                .cmp(&right.occurred_at)
                .then_with(|| left.id.cmp(&right.id))
        });

        if let Some(limit) = query.limit {
            events.truncate(limit as usize);
        }

        Ok(events)
    }
}

#[async_trait]
impl ConversationStore for SurrealStore {
    async fn upsert_project(&self, project: Project) -> Result<(), StoreError> {
        self.upsert_record("projects", &project.id.clone(), project)
            .await
    }

    async fn upsert_session(&self, session: Session) -> Result<(), StoreError> {
        self.upsert_record("sessions", &session.id.clone(), session)
            .await
    }

    async fn append_message(&self, message: Message) -> Result<(), StoreError> {
        self.upsert_record("messages", &message.id.clone(), message)
            .await
    }

    async fn record_tool_call(&self, call: ToolCall) -> Result<(), StoreError> {
        self.upsert_record("tool_calls", &call.id.clone(), call)
            .await
    }

    async fn record_tool_result(&self, result: ToolResult) -> Result<(), StoreError> {
        self.upsert_record("tool_results", &result.tool_call_id.clone(), result)
            .await
    }

    async fn get_session(&self, id: SessionId) -> Result<Option<Session>, StoreError> {
        self.select_record("sessions", &id).await
    }

    async fn list_sessions(&self, query: SessionQuery) -> Result<Vec<SessionSummary>, StoreError> {
        let mut sessions = self.select_table::<Session>("sessions").await?;

        sessions.retain(|session| {
            query
                .project_id
                .as_ref()
                .is_none_or(|project_id| &session.project_id == project_id)
                && query
                    .started_after
                    .is_none_or(|started_after| session.started_at >= started_after)
                && query
                    .started_before
                    .is_none_or(|started_before| session.started_at <= started_before)
        });

        sessions.sort_by(|left, right| {
            left.started_at
                .cmp(&right.started_at)
                .then_with(|| left.id.cmp(&right.id))
        });

        if let Some(limit) = query.limit {
            sessions.truncate(limit as usize);
        }

        Ok(sessions
            .into_iter()
            .map(|session| SessionSummary {
                id: session.id,
                project_id: session.project_id,
                provider: session.provider,
                model: session.model,
                started_at: session.started_at,
                ended_at: session.ended_at,
            })
            .collect())
    }

    async fn list_messages(&self, session_id: SessionId) -> Result<Vec<Message>, StoreError> {
        let mut messages = self.select_table::<Message>("messages").await?;
        messages.retain(|message| message.session_id == session_id);
        messages.sort_by(|left, right| {
            left.created_at
                .cmp(&right.created_at)
                .then_with(|| left.id.cmp(&right.id))
        });

        Ok(messages)
    }
}

#[async_trait]
impl GitStore for SurrealStore {
    async fn record_git_snapshot(&self, snapshot: GitSnapshot) -> Result<(), StoreError> {
        self.upsert_record("git_snapshots", &snapshot.id.to_string(), snapshot)
            .await
    }

    async fn record_commit(
        &self,
        commit: GitCommit,
        files: Vec<GitCommitFile>,
        hunks: Vec<GitCommitHunk>,
    ) -> Result<(), StoreError> {
        self.upsert_record("git_commits", &commit.oid.clone(), commit)
            .await?;

        for file in files {
            let record_id = Self::table_scoped_id(&[file.commit_oid.as_str(), file.path.as_str()]);
            self.upsert_record("git_commit_files", &record_id, file)
                .await?;
        }

        for hunk in hunks {
            self.upsert_record("git_commit_hunks", &hunk.id.to_string(), hunk)
                .await?;
        }

        Ok(())
    }

    async fn get_commit(&self, oid: GitOid) -> Result<Option<GitCommit>, StoreError> {
        self.select_record("git_commits", &oid).await
    }

    async fn list_commits_for_project(
        &self,
        project_id: ProjectId,
        query: CommitQuery,
    ) -> Result<Vec<GitCommit>, StoreError> {
        let mut commits = self.select_table::<GitCommit>("git_commits").await?;

        commits.retain(|commit| {
            commit.project_id == project_id
                && query
                    .committed_after
                    .is_none_or(|committed_after| commit.committed_at >= committed_after)
                && query
                    .committed_before
                    .is_none_or(|committed_before| commit.committed_at <= committed_before)
        });

        commits.sort_by(|left, right| {
            left.committed_at
                .cmp(&right.committed_at)
                .then_with(|| left.oid.cmp(&right.oid))
        });

        if let Some(limit) = query.limit {
            commits.truncate(limit as usize);
        }

        Ok(commits)
    }

    async fn list_file_history(
        &self,
        project_id: ProjectId,
        path: &str,
    ) -> Result<Vec<GitFileHistoryRow>, StoreError> {
        let commits = self
            .list_commits_for_project(project_id, CommitQuery::default())
            .await?;
        let commit_files = self
            .select_table::<GitCommitFile>("git_commit_files")
            .await?;

        let mut history = commits
            .into_iter()
            .filter(|commit| {
                commit_files.iter().any(|file| {
                    file.commit_oid == commit.oid
                        && (file.path == path || file.old_path.as_deref() == Some(path))
                })
            })
            .map(|commit| GitFileHistoryRow {
                commit_oid: commit.oid,
                path: path.to_string(),
                committed_at: commit.committed_at,
                message: commit.message,
            })
            .collect::<Vec<_>>();

        history.sort_by(|left, right| {
            left.committed_at
                .cmp(&right.committed_at)
                .then_with(|| left.commit_oid.cmp(&right.commit_oid))
        });

        Ok(history)
    }

    async fn list_commit_hunks(&self, oid: GitOid) -> Result<Vec<GitCommitHunk>, StoreError> {
        let mut hunks = self
            .select_table::<GitCommitHunk>("git_commit_hunks")
            .await?;
        hunks.retain(|hunk| hunk.commit_oid == oid);
        hunks.sort_by(|left, right| {
            left.file_path
                .cmp(&right.file_path)
                .then_with(|| left.new_start.cmp(&right.new_start))
                .then_with(|| left.old_start.cmp(&right.old_start))
                .then_with(|| left.id.cmp(&right.id))
        });

        Ok(hunks)
    }
}

#[async_trait]
impl TraceStore for SurrealStore {
    async fn upsert_execution_trace(&self, _trace: ExecutionTrace) -> Result<(), StoreError> {
        Self::todo()
    }

    async fn record_trace_frames(&self, _frames: Vec<TraceFrame>) -> Result<(), StoreError> {
        Self::todo()
    }

    async fn write_code_locations(&self, _locations: Vec<CodeLocation>) -> Result<(), StoreError> {
        Self::todo()
    }

    async fn get_trace(&self, _trace_id: &str) -> Result<Option<ExecutionTrace>, StoreError> {
        Self::todo()
    }

    async fn list_trace_frames(&self, _trace_id: &str) -> Result<Vec<TraceFrame>, StoreError> {
        Self::todo()
    }

    async fn find_code_locations(
        &self,
        _query: TraceLinkQuery,
    ) -> Result<Vec<CodeLocation>, StoreError> {
        Self::todo()
    }
}

#[async_trait]
impl ChangeStore for SurrealStore {
    async fn record_session_change(
        &self,
        _change: SessionCodeChange,
        _hunks: Vec<SessionChangeHunk>,
    ) -> Result<(), StoreError> {
        Self::todo()
    }

    async fn list_session_change_hunks(
        &self,
        _session_id: SessionId,
    ) -> Result<Vec<SessionChangeHunk>, StoreError> {
        Self::todo()
    }
}

#[async_trait]
impl AttributionStore for SurrealStore {
    async fn write_patch_attributions(
        &self,
        _attributions: Vec<PatchAttribution>,
    ) -> Result<(), StoreError> {
        Self::todo()
    }

    async fn write_commit_session_summaries(
        &self,
        _summaries: Vec<CommitSessionSummary>,
    ) -> Result<(), StoreError> {
        Self::todo()
    }

    async fn find_patch_attributions(
        &self,
        _query: CommitAttributionQuery,
    ) -> Result<Vec<PatchAttribution>, StoreError> {
        Self::todo()
    }

    async fn find_commit_contributors(
        &self,
        _oid: GitOid,
    ) -> Result<Vec<CommitSessionSummary>, StoreError> {
        Self::todo()
    }

    async fn find_file_contributors(
        &self,
        _project_id: ProjectId,
        _path: &str,
    ) -> Result<Vec<CommitSessionSummary>, StoreError> {
        Self::todo()
    }
}

#[async_trait]
impl SearchStore for SurrealStore {
    async fn search_text(&self, _query: TextSearchQuery) -> Result<SearchResults, StoreError> {
        Ok(SearchResults {
            hits: vec![SearchResult {
                kind: "todo".to_string(),
                id: "search-not-implemented".to_string(),
                title: "Search storage scaffolded".to_string(),
                snippet: Some(
                    "Implement text and semantic search in promptwho-storage-surreal".to_string(),
                ),
                score: 0.0,
            }],
        })
    }
}

#[async_trait]
impl VectorSearchStore for SurrealStore {
    async fn upsert_embedding(&self, _embedding: EmbeddingRecord) -> Result<(), StoreError> {
        Self::todo()
    }

    async fn search_similar(
        &self,
        _query: VectorSearchQuery,
    ) -> Result<Vec<VectorHit>, StoreError> {
        Self::todo()
    }
}
