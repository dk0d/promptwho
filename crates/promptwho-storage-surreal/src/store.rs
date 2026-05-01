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

use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Map as JsonMap, Value as JsonValue};
use surrealdb::types::{
    Array, Error as SurrealTypeError, Kind, Number, Object, SurrealValue, Value,
};
use surrealdb::{
    Surreal,
    engine::any::{Any, connect},
    opt::auth::Root,
};

struct Stored<T>(T);

// fn json_value_kind() -> Kind {
//     Kind::either(vec![
//         Kind::Null,
//         Kind::Bool,
//         Kind::Int,
//         Kind::Float,
//         Kind::Decimal,
//         Kind::String,
//         Kind::Array(Box::new(Kind::Any), None),
//         Kind::Object,
//     ])
// }

// fn stored_kind() -> Kind {
//     Kind::Literal(KindLiteral::Object(BTreeMap::from([(
//         "value".to_string(),
//         json_value_kind(),
//     )])))
// }

fn json_to_surreal_value(value: serde_json::Value) -> Value {
    match value {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(value) => Value::Bool(value),
        serde_json::Value::Number(value) => {
            if let Some(value) = value.as_i64() {
                Value::Number(Number::from(value))
            } else if let Some(value) = value.as_u64() {
                match i64::try_from(value) {
                    Ok(value) => Value::Number(Number::from(value)),
                    Err(_) => {
                        let value = surrealdb::types::Decimal::from_str_exact(&value.to_string())
                            .expect("failed to convert u64 JSON number to decimal");
                        Value::Number(Number::from(value))
                    }
                }
            } else if let Some(value) = value.as_f64() {
                Value::Number(Number::from(value))
            } else {
                panic!("unsupported JSON number representation")
            }
        }
        serde_json::Value::String(value) => Value::String(value),
        serde_json::Value::Array(values) => Value::Array(Array::from(
            values
                .into_iter()
                .map(json_to_surreal_value)
                .collect::<Vec<_>>(),
        )),
        serde_json::Value::Object(values) => Value::Object(Object::from_iter(
            values
                .into_iter()
                .map(|(key, value)| (key, json_to_surreal_value(value))),
        )),
    }
}

fn surreal_to_json_value(value: Value) -> serde_json::Value {
    match value {
        Value::None | Value::Null => serde_json::Value::Null,
        Value::Bool(value) => serde_json::Value::Bool(value),
        Value::Number(value) => match value {
            Number::Int(value) => serde_json::Value::Number(value.into()),
            Number::Float(value) => serde_json::Number::from_f64(value)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null),
            Number::Decimal(value) => serde_json::Value::String(value.to_string()),
        },
        Value::String(value) => serde_json::Value::String(value),
        Value::Array(values) => serde_json::Value::Array(
            values
                .into_iter()
                .map(surreal_to_json_value)
                .collect::<Vec<_>>(),
        ),
        Value::Object(values) => serde_json::Value::Object(
            values
                .into_iter()
                .map(|(key, value)| (key, surreal_to_json_value(value)))
                .collect(),
        ),
        value => value.into_json_value(),
    }
}

impl<T> SurrealValue for Stored<T>
where
    T: Serialize + DeserializeOwned,
{
    fn kind_of() -> surrealdb::types::Kind {
        // stored_kind()
        Kind::Object
    }

    fn into_value(self) -> Value {
        let json = serde_json::to_value(self.0).expect("failed to serialize stored value");
        let mut object = Object::new();
        object.insert("value".to_string(), json_to_surreal_value(json));
        Value::Object(object)
    }

    fn from_value(value: Value) -> Result<Self, surrealdb::types::Error> {
        let mut object = value.into_object()?;
        let value = object
            .remove("value")
            .ok_or_else(|| SurrealTypeError::thrown("missing stored value envelope".to_string()))?;
        let inner = serde_json::from_value(surreal_to_json_value(value))
            .map_err(|err| SurrealTypeError::thrown(err.to_string()))?;
        Ok(Self(inner))
    }
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

        let username = config.username.as_deref().unwrap_or("root");
        let password = config.password.as_deref().unwrap_or("secret");

        db.query(format!(
            "DEFINE USER {} ON ROOT PASSWORD '{}' ROLES OWNER",
            username, password
        ))
        .await
        .map_err(|err| StoreError::Message(err.to_string()))?;

        db.signin(Root {
            username: username.to_string(),
            password: password.to_string(),
        })
        .await
        .map_err(|err| StoreError::Message(err.to_string()))?;

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
        T: Serialize + DeserializeOwned,
    {
        let _: Option<Stored<T>> = self
            .db
            .upsert((table, id))
            .content(Stored(value))
            .await
            .map_err(|err| StoreError::Message(err.to_string()))?;

        Ok(())
    }

    async fn select_record<T>(&self, table: &str, id: &str) -> Result<Option<T>, StoreError>
    where
        T: Serialize + DeserializeOwned,
    {
        let record: Option<Stored<T>> = self
            .db
            .select((table, id))
            .await
            .map_err(|err| StoreError::Message(err.to_string()))?;

        Ok(record.map(|record| record.0))
    }

    async fn query_table<T>(
        &self,
        sql: &str,
        bindings: JsonMap<String, JsonValue>,
    ) -> Result<Vec<T>, StoreError>
    where
        T: Serialize + DeserializeOwned,
    {
        let mut response = self
            .db
            .query(sql.to_string())
            .bind(JsonValue::Object(bindings))
            .await
            .map_err(|err| StoreError::Message(err.to_string()))?;
        let records: Vec<Stored<T>> = response
            .take(0)
            .map_err(|err| StoreError::Message(err.to_string()))?;

        Ok(records.into_iter().map(|record| record.0).collect())
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
        let created: Option<Stored<StoredEvent>> = match self
            .db
            .create(("raw_events", event_id.as_str()))
            .content(Stored(event))
            .await
        {
            Ok(created) => created,
            Err(err) => {
                let message = err.to_string();
                if message.contains("already exists") {
                    return Ok(AppendOutcome { inserted: false });
                }
                return Err(StoreError::Message(message));
            }
        };

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
        let mut conditions = Vec::new();
        let mut bindings = JsonMap::new();

        if let Some(project_id) = query.project_id {
            conditions.push("value.project_id = $project_id".to_string());
            bindings.insert("project_id".to_string(), JsonValue::String(project_id));
        }

        if let Some(session_id) = query.session_id {
            conditions.push("value.session_id = $session_id".to_string());
            bindings.insert("session_id".to_string(), JsonValue::String(session_id));
        }

        if let Some(action) = query.action {
            conditions.push("value.action = $action".to_string());
            bindings.insert("action".to_string(), JsonValue::String(action));
        }

        if let Some(occurred_after) = query.occurred_after {
            conditions.push("value.occurred_at >= $occurred_after".to_string());
            bindings.insert(
                "occurred_after".to_string(),
                serde_json::to_value(occurred_after)
                    .map_err(|err| StoreError::Message(err.to_string()))?,
            );
        }

        if let Some(occurred_before) = query.occurred_before {
            conditions.push("value.occurred_at <= $occurred_before".to_string());
            bindings.insert(
                "occurred_before".to_string(),
                serde_json::to_value(occurred_before)
                    .map_err(|err| StoreError::Message(err.to_string()))?,
            );
        }

        let mut sql = "SELECT * FROM raw_events".to_string();
        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }
        sql.push_str(" ORDER BY value.occurred_at ASC, value.id ASC");
        if let Some(limit) = query.limit {
            sql.push_str(" LIMIT $limit");
            bindings.insert("limit".to_string(), JsonValue::from(limit));
        }

        self.query_table(&sql, bindings).await
    }
}

#[async_trait]
impl ConversationStore for SurrealStore {
    async fn upsert_project(&self, project: Project) -> Result<(), StoreError> {
        self.upsert_record("projects", &project.id.clone(), project)
            .await
    }

    async fn list_projects(&self) -> Result<Vec<Project>, StoreError> {
        self.query_table(
            "SELECT * FROM projects ORDER BY value.id ASC",
            JsonMap::new(),
        )
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
        let mut conditions = Vec::new();
        let mut bindings = JsonMap::new();

        if let Some(project_id) = query.project_id {
            conditions.push("value.project_id = $project_id".to_string());
            bindings.insert("project_id".to_string(), JsonValue::String(project_id));
        }

        if let Some(started_after) = query.started_after {
            conditions.push("value.started_at >= $started_after".to_string());
            bindings.insert(
                "started_after".to_string(),
                serde_json::to_value(started_after)
                    .map_err(|err| StoreError::Message(err.to_string()))?,
            );
        }

        if let Some(started_before) = query.started_before {
            conditions.push("value.started_at <= $started_before".to_string());
            bindings.insert(
                "started_before".to_string(),
                serde_json::to_value(started_before)
                    .map_err(|err| StoreError::Message(err.to_string()))?,
            );
        }

        let mut sql = "SELECT * FROM sessions".to_string();
        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }
        sql.push_str(" ORDER BY value.started_at ASC, value.id ASC");
        if let Some(limit) = query.limit {
            sql.push_str(" LIMIT $limit");
            bindings.insert("limit".to_string(), JsonValue::from(limit));
        }

        let sessions = self.query_table::<Session>(&sql, bindings).await?;

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
        let mut bindings = JsonMap::new();
        bindings.insert("session_id".to_string(), JsonValue::String(session_id));
        self.query_table(
            "SELECT * FROM messages WHERE value.session_id = $session_id ORDER BY value.created_at ASC, value.id ASC",
            bindings,
        )
        .await
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
        let mut conditions = vec!["value.project_id = $project_id".to_string()];
        let mut bindings = JsonMap::new();
        bindings.insert("project_id".to_string(), JsonValue::String(project_id));

        if let Some(committed_after) = query.committed_after {
            conditions.push("value.committed_at >= $committed_after".to_string());
            bindings.insert(
                "committed_after".to_string(),
                serde_json::to_value(committed_after)
                    .map_err(|err| StoreError::Message(err.to_string()))?,
            );
        }

        if let Some(committed_before) = query.committed_before {
            conditions.push("value.committed_at <= $committed_before".to_string());
            bindings.insert(
                "committed_before".to_string(),
                serde_json::to_value(committed_before)
                    .map_err(|err| StoreError::Message(err.to_string()))?,
            );
        }

        let mut sql = "SELECT * FROM git_commits WHERE ".to_string();
        sql.push_str(&conditions.join(" AND "));
        sql.push_str(" ORDER BY value.committed_at ASC, value.oid ASC");
        if let Some(limit) = query.limit {
            sql.push_str(" LIMIT $limit");
            bindings.insert("limit".to_string(), JsonValue::from(limit));
        }

        self.query_table(&sql, bindings).await
    }

    async fn list_file_history(
        &self,
        project_id: ProjectId,
        path: &str,
    ) -> Result<Vec<GitFileHistoryRow>, StoreError> {
        let mut bindings = JsonMap::new();
        bindings.insert(
            "project_id".to_string(),
            JsonValue::String(project_id.clone()),
        );
        bindings.insert("path".to_string(), JsonValue::String(path.to_string()));

        let commit_files = self
            .query_table::<GitCommitFile>(
                "SELECT * FROM git_commit_files WHERE value.path = $path OR value.old_path = $path ORDER BY value.commit_oid ASC, value.path ASC",
                bindings.clone(),
            )
            .await?;

        let commit_oids = commit_files
            .into_iter()
            .map(|file| file.commit_oid)
            .collect::<std::collections::BTreeSet<_>>();

        if commit_oids.is_empty() {
            return Ok(Vec::new());
        }

        let commits = self
            .list_commits_for_project(
                project_id,
                CommitQuery {
                    limit: None,
                    ..Default::default()
                },
            )
            .await?;

        Ok(commits
            .into_iter()
            .filter(|commit| commit_oids.contains(&commit.oid))
            .map(|commit| GitFileHistoryRow {
                commit_oid: commit.oid,
                path: path.to_string(),
                committed_at: commit.committed_at,
                message: commit.message,
            })
            .collect())
    }

    async fn list_commit_hunks(&self, oid: GitOid) -> Result<Vec<GitCommitHunk>, StoreError> {
        let mut bindings = JsonMap::new();
        bindings.insert("oid".to_string(), JsonValue::String(oid));
        self.query_table(
            "SELECT * FROM git_commit_hunks WHERE value.commit_oid = $oid ORDER BY value.file_path ASC, value.new_start ASC, value.old_start ASC, value.id ASC",
            bindings,
        )
        .await
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
    async fn search_text(&self, query: TextSearchQuery) -> Result<SearchResults, StoreError> {
        let search = query.text.trim().to_lowercase();
        if search.is_empty() {
            return Ok(SearchResults::default());
        }

        let project_name_by_id = self
            .query_table::<Project>(
                "SELECT * FROM projects ORDER BY value.id ASC",
                JsonMap::new(),
            )
            .await?
            .into_iter()
            .map(|project| (project.id, project.name))
            .collect::<std::collections::HashMap<_, _>>();

        let mut hits = Vec::new();

        for session in self
            .query_table::<Session>(
                "SELECT * FROM sessions ORDER BY value.id ASC",
                JsonMap::new(),
            )
            .await?
        {
            if query
                .project_id
                .as_ref()
                .is_some_and(|project_id| &session.project_id != project_id)
            {
                continue;
            }

            let session_text = format!(
                "{} {} {} {} {}",
                session.id,
                session.project_id,
                session.provider,
                session.model,
                session.branch.as_deref().unwrap_or_default()
            )
            .to_lowercase();

            if session_text.contains(&search) {
                hits.push(SearchResult {
                    kind: "session".to_string(),
                    id: session.id.clone(),
                    title: format!("{} / {}", session.provider, session.model,),
                    snippet: Some(format!(
                        "project={} branch={} started={}",
                        project_name_by_id
                            .get(&session.project_id)
                            .and_then(|name| name.as_deref())
                            .unwrap_or(session.project_id.as_str()),
                        session.branch.as_deref().unwrap_or("-"),
                        session.started_at,
                    )),
                    score: 1.0,
                });
            }
        }

        for message in self
            .query_table::<Message>(
                "SELECT * FROM messages ORDER BY value.id ASC",
                JsonMap::new(),
            )
            .await?
        {
            let session = match self.get_session(message.session_id.clone()).await? {
                Some(session) => session,
                None => continue,
            };

            if query
                .project_id
                .as_ref()
                .is_some_and(|project_id| &session.project_id != project_id)
            {
                continue;
            }

            let message_text =
                format!("{} {} {}", message.id, message.role, message.content).to_lowercase();
            if message_text.contains(&search) {
                let snippet = if message.content.len() > 180 {
                    format!("{}...", &message.content[..180])
                } else {
                    message.content.clone()
                };

                hits.push(SearchResult {
                    kind: "message".to_string(),
                    id: message.id.clone(),
                    title: format!("{} message in {}", message.role, session.id),
                    snippet: Some(snippet),
                    score: 2.0,
                });
            }
        }

        for event in self
            .list_events(EventQuery {
                project_id: query.project_id.clone(),
                limit: None,
                ..Default::default()
            })
            .await?
        {
            let event_text = serde_json::to_string(&event.envelope)
                .unwrap_or_default()
                .to_lowercase();

            if event_text.contains(&search) || event.action.to_lowercase().contains(&search) {
                hits.push(SearchResult {
                    kind: "event".to_string(),
                    id: event.id.to_string(),
                    title: format!("{} in {}", event.action, event.project_id),
                    snippet: Some(format!(
                        "occurred_at={} session={}",
                        event.occurred_at,
                        event.session_id.unwrap_or_default()
                    )),
                    score: 0.5,
                });
            }
        }

        hits.sort_by(|left, right| {
            right
                .score
                .partial_cmp(&left.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left.kind.cmp(&right.kind))
                .then_with(|| left.id.cmp(&right.id))
        });
        hits.truncate(query.limit as usize);

        Ok(SearchResults { hits })
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
