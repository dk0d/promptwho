use async_trait::async_trait;
use promptwho_storage::capabilities::{SupportsSyncMetadata, SupportsVectors};
use promptwho_storage::*;

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
use uuid::Uuid;

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

const RAW_EVENT_TABLE: &str = "raw_event";
const PROJECT_TABLE: &str = "project";
const PROJECT_FOREIGN_ID_TABLE: &str = "project_foreign_id";
const SESSION_TABLE: &str = "session";
const MESSAGE_TABLE: &str = "message";
const TOOL_CALL_TABLE: &str = "tool_call";
const GIT_SNAPSHOT_TABLE: &str = "git_snapshot";
const GIT_COMMIT_TABLE: &str = "git_commit";
const GIT_COMMIT_FILE_TABLE: &str = "git_commit_file";
const GIT_COMMIT_HUNK_TABLE: &str = "git_commit_hunk";
const SESSION_CODE_CHANGE_TABLE: &str = "session_code_change";
const SESSION_CHANGE_HUNK_TABLE: &str = "session_change_hunk";
const PATCH_ATTRIBUTION_TABLE: &str = "patch_attribution";

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
                DEFINE TABLE raw_event SCHEMALESS;
                DEFINE TABLE project SCHEMALESS;
                DEFINE TABLE project_foreign_id SCHEMALESS;
                DEFINE TABLE session SCHEMALESS;
                DEFINE TABLE message SCHEMALESS;
                DEFINE TABLE tool_call SCHEMALESS;
                DEFINE TABLE git_snapshot SCHEMALESS;
                DEFINE TABLE git_commit SCHEMALESS;
                DEFINE TABLE git_commit_file SCHEMALESS;
                DEFINE TABLE git_commit_hunk SCHEMALESS;
                DEFINE TABLE session_code_change SCHEMALESS;
                DEFINE TABLE session_change_hunk SCHEMALESS;
                DEFINE TABLE execution_trace SCHEMALESS;
                DEFINE TABLE trace_frame SCHEMALESS;
                DEFINE TABLE code_location SCHEMALESS;
                DEFINE TABLE patch_attribution SCHEMALESS;
                DEFINE TABLE embedding SCHEMALESS;

                DEFINE INDEX raw_event_id ON TABLE raw_event FIELDS id UNIQUE;
                DEFINE INDEX project_id ON TABLE project FIELDS id UNIQUE;
                DEFINE INDEX project_foreign_id_id ON TABLE project_foreign_id FIELDS id UNIQUE;
                DEFINE INDEX session_id ON TABLE session FIELDS id UNIQUE;
                DEFINE INDEX message_id ON TABLE message FIELDS id UNIQUE;
                DEFINE INDEX tool_call_id ON TABLE tool_call FIELDS id UNIQUE;
                DEFINE INDEX git_snapshot_id ON TABLE git_snapshot FIELDS id UNIQUE;
                DEFINE INDEX git_commit_oid ON TABLE git_commit FIELDS oid UNIQUE;
                DEFINE INDEX git_commit_file_id ON TABLE git_commit_file FIELDS id UNIQUE;
                DEFINE INDEX git_commit_hunk_id ON TABLE git_commit_hunk FIELDS id UNIQUE;
                DEFINE INDEX session_code_change_id ON TABLE session_code_change FIELDS id UNIQUE;
                DEFINE INDEX session_change_hunk_id ON TABLE session_change_hunk FIELDS id UNIQUE;
                DEFINE INDEX execution_trace_trace_id ON TABLE execution_trace FIELDS trace_id UNIQUE;
                DEFINE INDEX trace_frame_id ON TABLE trace_frame FIELDS id UNIQUE;
                DEFINE INDEX code_location_id ON TABLE code_location FIELDS id UNIQUE;
                DEFINE INDEX patch_attribution_id ON TABLE patch_attribution FIELDS id UNIQUE;
                DEFINE INDEX embedding_id ON TABLE embedding FIELDS id UNIQUE;
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

    async fn create_record<T>(&self, table: &str, id: &str, value: T) -> Result<T, StoreError>
    where
        T: Serialize + DeserializeOwned,
    {
        let created: Option<Stored<T>> = self
            .db
            .create((table, id))
            .content(Stored(value))
            .await
            .map_err(|err| StoreError::Message(err.to_string()))?;

        created
            .map(|record| record.0)
            .ok_or_else(|| StoreError::Message(format!("failed to create record in {table}: {id}")))
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

    fn offset_from_pagination(pagination: Option<Pagination>) -> Result<Option<u32>, StoreError> {
        match pagination {
            None => Ok(None),
            Some(Pagination::Offset(offset)) => Ok(Some(offset)),
            Some(Pagination::Cursor(_)) => Err(StoreError::Message(
                "cursor pagination is not supported by SurrealStore yet".to_string(),
            )),
        }
    }

    fn todo<T>() -> Result<T, StoreError> {
        Err(StoreError::Message(
            "surreal storage methods are scaffolded but not yet implemented".to_string(),
        ))
    }

    fn project_record_key(project_id: &str) -> &str {
        project_id.strip_prefix("project:").unwrap_or(project_id)
    }

    async fn get_project_by_canonical_id(
        &self,
        project_id: String,
    ) -> Result<Option<Project>, StoreError> {
        let mut projects = self
            .list_projects(Some(ProjectQuery {
                id: Some(project_id),
                limit: Some(1),
                ..Default::default()
            }))
            .await?;
        Ok(projects.pop())
    }
}

async fn filter_patch_attributions_by_commit(
    store: &SurrealStore,
    rows: Vec<PatchAttribution>,
    commit_oid: &str,
) -> Result<Vec<PatchAttribution>, StoreError> {
    let mut filtered = Vec::new();

    for row in rows {
        let Some(hunk) = store
            .select_record::<GitCommitHunk>(GIT_COMMIT_HUNK_TABLE, &row.commit_hunk_id.to_string())
            .await?
        else {
            continue;
        };
        let Some(file) = store
            .select_record::<GitCommitFile>(GIT_COMMIT_FILE_TABLE, &hunk.commit_file_id)
            .await?
        else {
            continue;
        };

        if file.commit_oid == commit_oid {
            filtered.push(row);
        }
    }

    Ok(filtered)
}

async fn summarize_patch_attributions(
    store: &SurrealStore,
    commit_oid: &str,
    attributions: Vec<PatchAttribution>,
) -> Result<Vec<CommitSessionSummary>, StoreError> {
    let mut by_session = std::collections::BTreeMap::<SessionId, Vec<PatchAttribution>>::new();

    for attribution in attributions {
        let Some(change_hunk) = store
            .select_record::<SessionChangeHunk>(
                SESSION_CHANGE_HUNK_TABLE,
                &attribution.session_change_hunk_id.to_string(),
            )
            .await?
        else {
            continue;
        };
        let Some(change) = store
            .select_record::<SessionCodeChange>(
                SESSION_CODE_CHANGE_TABLE,
                &change_hunk.change_id.to_string(),
            )
            .await?
        else {
            continue;
        };

        by_session
            .entry(change.session_id)
            .or_default()
            .push(attribution);
    }

    Ok(by_session
        .into_iter()
        .map(|(session_id, rows)| {
            let patch_count = rows.len() as u32;
            let score = rows.iter().map(|row| row.score).sum::<f32>();
            let summary = rows
                .iter()
                .flat_map(|row| row.reasons.iter().cloned())
                .collect::<Vec<_>>();

            CommitSessionSummary {
                commit_oid: commit_oid.to_string(),
                session_id,
                patch_count,
                score,
                summary,
            }
        })
        .collect())
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
            .create((RAW_EVENT_TABLE, event_id.as_str()))
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
        self.select_record(RAW_EVENT_TABLE, &id.to_string()).await
    }

    async fn list_events(&self, query: Option<EventQuery>) -> Result<Vec<StoredEvent>, StoreError> {
        let query = query.unwrap_or_default();
        let offset = Self::offset_from_pagination(query.pagination.clone())?;
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

        let mut sql = format!("SELECT * FROM {RAW_EVENT_TABLE}");
        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }

        match query.sort {
            Some(SortOrder::Ascending(by)) => {
                let by = by.unwrap_or("occurred_at".to_string());
                sql.push_str(&format!(" ORDER BY value.{by} ASC"));
            }
            Some(SortOrder::Descending(by)) => {
                let by = by.unwrap_or("occurred_at".to_string());
                sql.push_str(&format!(" ORDER BY value.{by} DESC"));
            }
            None => {
                sql.push_str(" ORDER BY value.occurred_at ASC, value.id ASC");
            }
        }
        if let Some(limit) = query.limit {
            sql.push_str(" LIMIT $limit");
            bindings.insert("limit".to_string(), JsonValue::from(limit));
        }
        if let Some(offset) = offset {
            sql.push_str(" START $offset");
            bindings.insert("offset".to_string(), JsonValue::from(offset));
        }

        self.query_table(&sql, bindings).await
    }
}

#[async_trait]
impl ConversationStore for SurrealStore {
    async fn upsert_project(&self, project: Project) -> Result<Project, StoreError> {
        let project_id = project.id.clone();
        let record_key = if Self::project_record_key(&project_id) != project_id
            && self
                .select_record::<Project>(PROJECT_TABLE, &project_id)
                .await?
                .is_some()
        {
            project_id.as_str()
        } else {
            Self::project_record_key(&project_id)
        };

        self.upsert_record(PROJECT_TABLE, record_key, project).await?;
        self.get_project_by_canonical_id(project_id.clone())
            .await?
            .ok_or_else(|| StoreError::Message(format!("project not found after upsert: {project_id}")))
    }

    async fn create_project(&self, mut project: Project) -> Result<Project, StoreError> {
        if project.id.is_empty() {
            project.id = format!("project:{}", Uuid::new_v4());
        }
        let project_id = project.id.clone();
        self.create_record(PROJECT_TABLE, Self::project_record_key(&project_id), project)
            .await
    }

    async fn get_project_by_fingerprint(
        &self,
        repository_fingerprint: String,
    ) -> Result<Option<Project>, StoreError> {
        let mut projects = self
            .list_projects(Some(ProjectQuery {
                repository_fingerprint: Some(repository_fingerprint),
                limit: Some(1),
                ..Default::default()
            }))
            .await?;
        Ok(projects.pop())
    }

    async fn get_project_by_foreign_id(
        &self,
        source: String,
        foreign_id: String,
    ) -> Result<Option<Project>, StoreError> {
        let foreign_key = format!("{source}:{foreign_id}");
        let Some(mapping) = self
            .select_record::<ProjectForeignId>(PROJECT_FOREIGN_ID_TABLE, &foreign_key)
            .await?
        else {
            return Ok(None);
        };

        self.get_project_by_canonical_id(mapping.pid).await
    }

    async fn upsert_project_foreign_id(
        &self,
        foreign_id: ProjectForeignId,
    ) -> Result<ProjectForeignId, StoreError> {
        let foreign_key = format!("{}:{}", foreign_id.source, foreign_id.fid);
        self.upsert_record(PROJECT_FOREIGN_ID_TABLE, &foreign_key, foreign_id)
            .await?;
        self.select_record(PROJECT_FOREIGN_ID_TABLE, &foreign_key)
            .await?
            .ok_or_else(|| {
                StoreError::Message(format!("project foreign id not found after upsert: {foreign_key}"))
            })
    }

    async fn list_projects(&self, query: Option<ProjectQuery>) -> Result<Vec<Project>, StoreError> {
        let query = query.unwrap_or_default();
        let offset = Self::offset_from_pagination(query.pagination.clone())?;
        let mut conditions = Vec::new();
        let mut bindings = JsonMap::new();

        if let Some(id) = query.id {
            conditions.push("value.id = $id".to_string());
            bindings.insert("id".to_string(), JsonValue::String(id));
        }

        if let Some(root) = query.root {
            conditions.push("value.root = $root".to_string());
            bindings.insert("root".to_string(), JsonValue::String(root));
        }

        if let Some(name) = query.name {
            conditions.push("value.name = $name".to_string());
            bindings.insert("name".to_string(), JsonValue::String(name));
        }

        if let Some(repository_fingerprint) = query.repository_fingerprint {
            conditions.push("value.repository_fingerprint = $repository_fingerprint".to_string());
            bindings.insert(
                "repository_fingerprint".to_string(),
                JsonValue::String(repository_fingerprint),
            );
        }

        if let Some(created_after) = query.created_after {
            conditions.push("value.created_at >= $created_after".to_string());
            bindings.insert(
                "created_after".to_string(),
                serde_json::to_value(created_after)
                    .map_err(|err| StoreError::Message(err.to_string()))?,
            );
        }

        if let Some(created_before) = query.created_before {
            conditions.push("value.created_at <= $created_before".to_string());
            bindings.insert(
                "created_before".to_string(),
                serde_json::to_value(created_before)
                    .map_err(|err| StoreError::Message(err.to_string()))?,
            );
        }

        let mut sql = format!("SELECT * FROM {PROJECT_TABLE}");
        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }
        sql.push_str(" ORDER BY value.id ASC");
        if let Some(limit) = query.limit {
            sql.push_str(" LIMIT $limit");
            bindings.insert("limit".to_string(), JsonValue::from(limit));
        }
        if let Some(offset) = offset {
            sql.push_str(" START $offset");
            bindings.insert("offset".to_string(), JsonValue::from(offset));
        }

        self.query_table(&sql, bindings).await
    }

    async fn upsert_session(&self, session: Session) -> Result<(), StoreError> {
        self.upsert_record(SESSION_TABLE, &session.id.clone(), session)
            .await
    }

    async fn append_message(&self, message: Message) -> Result<(), StoreError> {
        self.upsert_record(MESSAGE_TABLE, &message.id.clone(), message)
            .await
    }

    async fn record_tool_call(&self, call: ToolCall) -> Result<(), StoreError> {
        self.upsert_record(TOOL_CALL_TABLE, &call.id.clone(), call)
            .await
    }

    async fn complete_tool_call(
        &self,
        tool_call_id: ToolCallId,
        success: bool,
        output: JsonValue,
        completed_at: promptwho_protocol::TimestampUtc,
        metadata: JsonValue,
    ) -> Result<(), StoreError> {
        let Some(mut call) = self
            .select_record::<ToolCall>(TOOL_CALL_TABLE, &tool_call_id)
            .await?
        else {
            return Err(StoreError::Message(format!(
                "tool call not found for completion: {tool_call_id}"
            )));
        };

        call.success = Some(success);
        call.output = Some(output);
        call.completed_at = Some(completed_at);
        call.metadata = metadata;

        self.upsert_record(TOOL_CALL_TABLE, &tool_call_id, call)
            .await
    }

    async fn get_session(&self, id: SessionId) -> Result<Option<Session>, StoreError> {
        self.select_record(SESSION_TABLE, &id).await
    }

    async fn get_message(&self, id: MessageId) -> Result<Option<Message>, StoreError> {
        self.select_record(MESSAGE_TABLE, &id).await
    }

    async fn list_sessions(
        &self,
        query: Option<SessionQuery>,
    ) -> Result<Vec<SessionSummary>, StoreError> {
        let query = query.unwrap_or_default();
        let offset = Self::offset_from_pagination(query.pagination.clone())?;
        let mut conditions = Vec::new();
        let mut bindings = JsonMap::new();

        if let Some(id) = query.id {
            conditions.push("value.id = $id".to_string());
            bindings.insert("id".to_string(), JsonValue::String(id));
        }

        if let Some(project_id) = query.project_id {
            conditions.push("value.project_id = $project_id".to_string());
            bindings.insert("project_id".to_string(), JsonValue::String(project_id));
        }

        if let Some(provider) = query.provider {
            conditions.push("value.provider = $provider".to_string());
            bindings.insert("provider".to_string(), JsonValue::String(provider));
        }

        if let Some(model) = query.model {
            conditions.push("value.model = $model".to_string());
            bindings.insert("model".to_string(), JsonValue::String(model));
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

        let mut sql = format!("SELECT * FROM {SESSION_TABLE}");
        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }
        sql.push_str(" ORDER BY value.started_at ASC, value.id ASC");
        if let Some(limit) = query.limit {
            sql.push_str(" LIMIT $limit");
            bindings.insert("limit".to_string(), JsonValue::from(limit));
        }
        if let Some(offset) = offset {
            sql.push_str(" START $offset");
            bindings.insert("offset".to_string(), JsonValue::from(offset));
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

    async fn list_messages(
        &self,
        session_id: SessionId,
        query: Option<MessageQuery>,
    ) -> Result<Vec<Message>, StoreError> {
        let query = query.unwrap_or_default();
        let offset = Self::offset_from_pagination(query.pagination.clone())?;
        let mut conditions = vec!["value.session_id = $session_id".to_string()];
        let mut bindings = JsonMap::new();
        bindings.insert("session_id".to_string(), JsonValue::String(session_id));

        if let Some(role) = query.role {
            conditions.push("value.role = $role".to_string());
            bindings.insert("role".to_string(), JsonValue::String(role));
        }

        if let Some(created_after) = query.created_after {
            conditions.push("value.created_at >= $created_after".to_string());
            bindings.insert(
                "created_after".to_string(),
                serde_json::to_value(created_after)
                    .map_err(|err| StoreError::Message(err.to_string()))?,
            );
        }

        if let Some(created_before) = query.created_before {
            conditions.push("value.created_at <= $created_before".to_string());
            bindings.insert(
                "created_before".to_string(),
                serde_json::to_value(created_before)
                    .map_err(|err| StoreError::Message(err.to_string()))?,
            );
        }

        let mut sql = format!("SELECT * FROM {MESSAGE_TABLE} WHERE ");
        sql.push_str(&conditions.join(" AND "));
        sql.push_str(" ORDER BY value.created_at ASC, value.id ASC");
        if let Some(limit) = query.limit {
            sql.push_str(" LIMIT $limit");
            bindings.insert("limit".to_string(), JsonValue::from(limit));
        }
        if let Some(offset) = offset {
            sql.push_str(" START $offset");
            bindings.insert("offset".to_string(), JsonValue::from(offset));
        }

        self.query_table(&sql, bindings).await
    }
}

#[async_trait]
impl GitStore for SurrealStore {
    async fn record_git_snapshot(&self, snapshot: GitSnapshot) -> Result<(), StoreError> {
        self.upsert_record(GIT_SNAPSHOT_TABLE, &snapshot.id.to_string(), snapshot)
            .await
    }

    async fn record_commit(
        &self,
        commit: GitCommit,
        files: Vec<GitCommitFile>,
        hunks: Vec<GitCommitHunk>,
    ) -> Result<(), StoreError> {
        self.upsert_record(GIT_COMMIT_TABLE, &commit.oid.clone(), commit)
            .await?;

        for file in files {
            self.upsert_record(GIT_COMMIT_FILE_TABLE, &file.id.clone(), file)
                .await?;
        }

        for hunk in hunks {
            self.upsert_record(GIT_COMMIT_HUNK_TABLE, &hunk.id.to_string(), hunk)
                .await?;
        }

        Ok(())
    }

    async fn get_commit(&self, oid: GitOid) -> Result<Option<GitCommit>, StoreError> {
        self.select_record(GIT_COMMIT_TABLE, &oid).await
    }

    async fn list_commits_for_project(
        &self,
        project_id: ProjectId,
        query: Option<CommitQuery>,
    ) -> Result<Vec<GitCommit>, StoreError> {
        let query = query.unwrap_or_default();
        let offset = Self::offset_from_pagination(query.pagination.clone())?;
        let mut conditions = vec!["value.project_id = $project_id".to_string()];
        let mut bindings = JsonMap::new();
        bindings.insert("project_id".to_string(), JsonValue::String(project_id));

        if let Some(oid) = query.oid {
            conditions.push("value.oid = $oid".to_string());
            bindings.insert("oid".to_string(), JsonValue::String(oid));
        }

        if let Some(author_email) = query.author_email {
            conditions.push("value.author_email = $author_email".to_string());
            bindings.insert("author_email".to_string(), JsonValue::String(author_email));
        }

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

        let mut sql = format!("SELECT * FROM {GIT_COMMIT_TABLE} WHERE ");
        sql.push_str(&conditions.join(" AND "));
        sql.push_str(" ORDER BY value.committed_at ASC, value.oid ASC");
        if let Some(limit) = query.limit {
            sql.push_str(" LIMIT $limit");
            bindings.insert("limit".to_string(), JsonValue::from(limit));
        }
        if let Some(offset) = offset {
            sql.push_str(" START $offset");
            bindings.insert("offset".to_string(), JsonValue::from(offset));
        }

        self.query_table(&sql, bindings).await
    }

    async fn list_file_history(
        &self,
        project_id: ProjectId,
        path: &str,
        query: Option<FileHistoryQuery>,
    ) -> Result<Vec<GitFileHistoryRow>, StoreError> {
        let query = query.unwrap_or_default();
        let offset = Self::offset_from_pagination(query.pagination.clone())?;
        let mut bindings = JsonMap::new();
        bindings.insert(
            "project_id".to_string(),
            JsonValue::String(project_id.clone()),
        );
        bindings.insert("path".to_string(), JsonValue::String(path.to_string()));

        let commit_files = self
            .query_table::<GitCommitFile>(
                &format!(
                    "SELECT * FROM {GIT_COMMIT_FILE_TABLE} WHERE value.path = $path OR value.old_path = $path ORDER BY value.commit_oid ASC, value.path ASC"
                ),
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
                Some(CommitQuery {
                    committed_after: query.committed_after,
                    committed_before: query.committed_before,
                    limit: None,
                    ..Default::default()
                }),
            )
            .await?;

        let filtered_commits = commits
            .into_iter()
            .filter(|commit| commit_oids.contains(&commit.oid))
            .filter(|commit| {
                query
                    .commit_oid
                    .as_ref()
                    .is_none_or(|commit_oid| commit.oid == *commit_oid)
            })
            .map(|commit| GitFileHistoryRow {
                commit_oid: commit.oid,
                path: path.to_string(),
                committed_at: commit.committed_at,
                message: commit.message,
            })
            .collect::<Vec<_>>();

        let offset = offset.unwrap_or(0) as usize;
        let limit = query.limit.map(|limit| limit as usize);

        Ok(match limit {
            Some(limit) => filtered_commits
                .into_iter()
                .skip(offset)
                .take(limit)
                .collect(),
            None => filtered_commits.into_iter().skip(offset).collect(),
        })
    }

    async fn list_commit_hunks(
        &self,
        oid: GitOid,
        query: Option<CommitHunkQuery>,
    ) -> Result<Vec<GitCommitHunk>, StoreError> {
        let query = query.unwrap_or_default();
        let offset = Self::offset_from_pagination(query.pagination.clone())?;
        let mut file_conditions = vec!["value.commit_oid = $oid".to_string()];
        let mut bindings = JsonMap::new();
        bindings.insert("oid".to_string(), JsonValue::String(oid));

        if let Some(file_path) = query.file_path {
            file_conditions.push("value.path = $file_path".to_string());
            bindings.insert("file_path".to_string(), JsonValue::String(file_path));
        }

        let mut file_sql = format!("SELECT * FROM {GIT_COMMIT_FILE_TABLE} WHERE ");
        file_sql.push_str(&file_conditions.join(" AND "));
        file_sql.push_str(" ORDER BY value.path ASC, value.id ASC");

        let files = self
            .query_table::<GitCommitFile>(&file_sql, bindings.clone())
            .await?;
        if files.is_empty() {
            return Ok(Vec::new());
        }

        let file_ids = files.into_iter().map(|file| file.id).collect::<Vec<_>>();

        let mut conditions = vec!["value.commit_file_id INSIDE $commit_file_ids".to_string()];
        bindings.insert(
            "commit_file_ids".to_string(),
            JsonValue::Array(file_ids.into_iter().map(JsonValue::String).collect()),
        );

        if let Some(hunk_header) = query.hunk_header {
            conditions.push("value.hunk_header = $hunk_header".to_string());
            bindings.insert("hunk_header".to_string(), JsonValue::String(hunk_header));
        }

        let mut sql = format!("SELECT * FROM {GIT_COMMIT_HUNK_TABLE} WHERE ");
        sql.push_str(&conditions.join(" AND "));
        sql.push_str(
            " ORDER BY value.file_path ASC, value.new_start ASC, value.old_start ASC, value.id ASC",
        );
        if let Some(limit) = query.limit {
            sql.push_str(" LIMIT $limit");
            bindings.insert("limit".to_string(), JsonValue::from(limit));
        }
        if let Some(offset) = offset {
            sql.push_str(" START $offset");
            bindings.insert("offset".to_string(), JsonValue::from(offset));
        }

        self.query_table(&sql, bindings).await
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

    async fn list_trace_frames(
        &self,
        _trace_id: &str,
        _query: Option<TraceFrameQuery>,
    ) -> Result<Vec<TraceFrame>, StoreError> {
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
        _query: Option<SessionChangeHunkQuery>,
    ) -> Result<Vec<SessionChangeHunk>, StoreError> {
        Self::todo()
    }
}

#[async_trait]
impl AttributionStore for SurrealStore {
    async fn write_patch_attributions(
        &self,
        attributions: Vec<PatchAttribution>,
    ) -> Result<(), StoreError> {
        for attribution in attributions {
            self.upsert_record(
                PATCH_ATTRIBUTION_TABLE,
                &attribution.id.to_string(),
                attribution,
            )
            .await?;
        }

        Ok(())
    }

    async fn find_patch_attributions(
        &self,
        query: CommitAttributionQuery,
    ) -> Result<Vec<PatchAttribution>, StoreError> {
        let mut conditions = Vec::new();
        let mut bindings = JsonMap::new();

        if let Some(minimum_score) = query.minimum_score {
            conditions.push("value.score >= $minimum_score".to_string());
            bindings.insert("minimum_score".to_string(), JsonValue::from(minimum_score));
        }

        if let Some(session_id) = query.session_id {
            let sql = r#"
                SELECT patch.value AS value
                FROM patch_attribution patch
                WHERE patch.value.session_change_hunk_id INSIDE (
                    SELECT VALUE value.id FROM session_change_hunk
                    WHERE value.change_id INSIDE (
                        SELECT VALUE value.id FROM session_code_change WHERE value.session_id = $session_id
                    )
                )
            "#;

            let mut rows = self
                .query_table::<PatchAttribution>(sql, {
                    let mut scoped = bindings.clone();
                    scoped.insert("session_id".to_string(), JsonValue::String(session_id));
                    scoped
                })
                .await?;

            if let Some(commit_oid) = query.commit_oid {
                rows = filter_patch_attributions_by_commit(self, rows, &commit_oid).await?;
            }

            if let Some(limit) = query.limit {
                rows.truncate(limit as usize);
            }

            return Ok(rows);
        }

        let mut sql = format!("SELECT * FROM {PATCH_ATTRIBUTION_TABLE}");
        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }
        sql.push_str(" ORDER BY value.created_at DESC, value.id ASC");
        if let Some(limit) = query.limit {
            sql.push_str(" LIMIT $limit");
            bindings.insert("limit".to_string(), JsonValue::from(limit));
        }

        let rows = self.query_table::<PatchAttribution>(&sql, bindings).await?;
        match query.commit_oid {
            Some(commit_oid) => filter_patch_attributions_by_commit(self, rows, &commit_oid).await,
            None => Ok(rows),
        }
    }

    async fn find_commit_contributors(
        &self,
        oid: GitOid,
    ) -> Result<Vec<CommitSessionSummary>, StoreError> {
        let attributions = self
            .find_patch_attributions(CommitAttributionQuery {
                commit_oid: Some(oid.clone()),
                limit: None,
                ..Default::default()
            })
            .await?;

        summarize_patch_attributions(self, &oid, attributions).await
    }

    async fn find_file_contributors(
        &self,
        project_id: ProjectId,
        path: &str,
    ) -> Result<Vec<CommitSessionSummary>, StoreError> {
        let history = self.list_file_history(project_id, path, None).await?;
        let mut summaries = Vec::new();

        for row in history {
            summaries.extend(self.find_commit_contributors(row.commit_oid).await?);
        }

        summaries.sort_by(|left, right| {
            right
                .score
                .partial_cmp(&left.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left.session_id.cmp(&right.session_id))
        });
        summaries.dedup_by(|left, right| {
            left.commit_oid == right.commit_oid && left.session_id == right.session_id
        });

        Ok(summaries)
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
                &format!("SELECT * FROM {PROJECT_TABLE} ORDER BY value.id ASC"),
                JsonMap::new(),
            )
            .await?
            .into_iter()
            .map(|project| (project.id, project.name))
            .collect::<std::collections::HashMap<_, _>>();

        let mut hits = Vec::new();

        for session in self
            .query_table::<Session>(
                &format!("SELECT * FROM {SESSION_TABLE} ORDER BY value.id ASC"),
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
                session.started_on_branch.as_deref().unwrap_or_default()
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
                        session.started_on_branch.as_deref().unwrap_or("-"),
                        session.started_at,
                    )),
                    score: 1.0,
                });
            }
        }

        for message in self
            .query_table::<Message>(
                &format!("SELECT * FROM {MESSAGE_TABLE} ORDER BY value.id ASC"),
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
            .list_events(Some(EventQuery {
                project_id: query.project_id.clone(),
                limit: None,
                ..Default::default()
            }))
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
