use promptwho_storage::{
    AttributionStore, CommitSessionSummary, GitOid, PatchAttribution, ProjectId, StoreError,
};

#[derive(Debug, thiserror::Error)]
pub enum AttributionError {
    #[error(transparent)]
    Store(#[from] StoreError),
}

pub struct AttributionService<S> {
    pub store: S,
}

impl<S> AttributionService<S>
where
    S: AttributionStore,
{
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub async fn attribution_for_commit(
        &self,
        commit_oid: GitOid,
    ) -> Result<Vec<CommitSessionSummary>, AttributionError> {
        self.store
            .find_commit_contributors(commit_oid)
            .await
            .map_err(Into::into)
    }

    pub async fn attribution_for_file(
        &self,
        project_id: ProjectId,
        path: &str,
    ) -> Result<Vec<CommitSessionSummary>, AttributionError> {
        self.store
            .find_file_contributors(project_id, path)
            .await
            .map_err(Into::into)
    }

    pub async fn patch_attribution_for_commit(
        &self,
        commit_oid: GitOid,
    ) -> Result<Vec<PatchAttribution>, AttributionError> {
        self.store
            .find_patch_attributions(promptwho_storage::CommitAttributionQuery {
                commit_oid: Some(commit_oid),
                ..Default::default()
            })
            .await
            .map_err(Into::into)
    }
}
