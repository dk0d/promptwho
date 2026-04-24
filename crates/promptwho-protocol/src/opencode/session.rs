use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::shared::SnapshotFileDiff;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PermissionRule {
    pub permission: String,
    pub pattern: String,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SessionSummary {
    pub additions: i64,
    pub deletions: i64,
    pub files: i64,
    pub diffs: Option<Vec<SnapshotFileDiff>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SessionShare {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SessionRevert {
    #[serde(rename = "messageID", alias = "messageId")]
    pub message_id: String,
    #[serde(rename = "partID", alias = "partId")]
    pub part_id: Option<String>,
    pub snapshot: Option<String>,
    pub diff: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SessionTime {
    pub created: i64,
    pub updated: i64,
    pub compacting: Option<i64>,
    pub archived: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: String,
    pub slug: String,
    #[serde(rename = "projectID", alias = "projectId")]
    pub project_id: String,
    #[serde(rename = "workspaceID", alias = "workspaceId")]
    pub workspace_id: Option<String>,
    pub directory: String,
    #[serde(rename = "parentID", alias = "parentId")]
    pub parent_id: Option<String>,
    pub summary: Option<SessionSummary>,
    pub share: Option<SessionShare>,
    pub title: String,
    pub version: String,
    pub time: SessionTime,
    pub permission: Option<Vec<PermissionRule>>,
    pub revert: Option<SessionRevert>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PartialSessionShare {
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PartialSessionTime {
    pub created: Option<i64>,
    pub updated: Option<i64>,
    pub compacting: Option<i64>,
    pub archived: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PartialSession {
    pub id: Option<String>,
    pub slug: Option<String>,
    #[serde(rename = "projectID", alias = "projectId")]
    pub project_id: Option<String>,
    #[serde(rename = "workspaceID", alias = "workspaceId")]
    pub workspace_id: Option<String>,
    pub directory: Option<String>,
    #[serde(rename = "parentID", alias = "parentId")]
    pub parent_id: Option<String>,
    pub summary: Option<SessionSummary>,
    pub share: Option<PartialSessionShare>,
    pub title: Option<String>,
    pub version: Option<String>,
    pub time: Option<PartialSessionTime>,
    pub permission: Option<Vec<PermissionRule>>,
    pub revert: Option<SessionRevert>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfoProperties {
    #[serde(rename = "sessionID", alias = "sessionId")]
    pub session_id: String,
    pub info: Session,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PartialSessionInfoProperties {
    #[serde(rename = "sessionID", alias = "sessionId")]
    pub session_id: String,
    pub info: PartialSession,
}
