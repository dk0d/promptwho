use std::{
    collections::BTreeMap,
    fs,
    hash::{Hash, Hasher},
    io::Write,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::anyhow;
use chrono::{DateTime, Utc};
use gix::{ObjectId, Repository, diff::Options as DiffOptions, prelude::TreeDiffChangeExt};
use promptwho_protocol::{
    EventEnvelope, EventPayload, GitCommitFilePayload, GitCommitHunkPayload, GitCommitPayload,
    PluginSource, ProjectRef, ProtocolVersion,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::time;
use uuid::Uuid;

use crate::emitter::{EventEmitter, PublishError};

const DEFAULT_POLL_INTERVAL_SECS: u64 = 2;
const DEFAULT_CHECKPOINT_RELATIVE_PATH: &str = ".promptwho/git-watcher.json";

#[derive(Debug, Clone)]
pub struct WatcherConfig {
    pub repo_path: PathBuf,
    pub poll_interval: Duration,
    pub emit_existing_head: bool,
    pub project_id: Option<String>,
    pub project_name: Option<String>,
    pub checkpoint_path: Option<PathBuf>,
    pub source: PluginSource,
}

impl WatcherConfig {
    pub fn new(repo_path: impl Into<PathBuf>) -> Self {
        Self {
            repo_path: repo_path.into(),
            poll_interval: Duration::from_secs(DEFAULT_POLL_INTERVAL_SECS),
            emit_existing_head: false,
            project_id: None,
            project_name: None,
            checkpoint_path: None,
            source: PluginSource {
                plugin: "promptwho-watcher".to_string(),
                plugin_version: env!("CARGO_PKG_VERSION").to_string(),
                runtime: "rust".to_string(),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommitObservation {
    pub branch: Option<String>,
    pub head_commit: String,
    pub parent_commit: Option<String>,
    pub occurred_at: DateTime<Utc>,
    pub commit_author_name: Option<String>,
    pub commit_author_email: Option<String>,
    pub commit_timestamp: Option<DateTime<Utc>>,
    pub commit_title: Option<String>,
    pub commit_body: Option<String>,
    pub message: Option<String>,
    pub files: Vec<GitCommitFilePayload>,
    pub hunks: Vec<GitCommitHunkPayload>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct WatcherCheckpoint {
    head_commit: String,
}

#[derive(Debug, Default)]
struct HunkCollector {
    hunks: Vec<GitCommitHunkPayload>,
}

#[derive(Debug, thiserror::Error)]
pub enum WatcherError {
    #[error("failed to open git repository at {path}")]
    OpenRepository {
        path: PathBuf,
        #[source]
        source: anyhow::Error,
    },
    #[error("failed to read git HEAD state")]
    HeadState {
        #[source]
        source: anyhow::Error,
    },
    #[error(transparent)]
    Publish(#[from] PublishError),
}

pub struct GitWatcher<E> {
    repo: Repository,
    emitter: E,
    config: WatcherConfig,
    project: ProjectRef,
    checkpoint_path: PathBuf,
    last_seen_head: Option<String>,
}

impl<E> GitWatcher<E>
where
    E: EventEmitter,
{
    pub fn new(config: WatcherConfig, emitter: E) -> Result<Self, WatcherError> {
        let repo =
            gix::discover(&config.repo_path).map_err(|source| WatcherError::OpenRepository {
                path: config.repo_path.clone(),
                source: anyhow::Error::new(source),
            })?;

        let work_dir = repo
            .workdir()
            .unwrap_or(config.repo_path.as_path())
            .to_path_buf();
        let checkpoint_path = config
            .checkpoint_path
            .clone()
            .unwrap_or_else(|| work_dir.join(DEFAULT_CHECKPOINT_RELATIVE_PATH));
        let project = ProjectRef {
            id: config
                .project_id
                .clone()
                .unwrap_or_else(|| default_project_id(&work_dir)),
            root: work_dir.display().to_string(),
            name: config.project_name.clone().or_else(|| {
                work_dir
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
            }),
            repository_fingerprint: repository_fingerprint(&repo, &work_dir),
        };

        let persisted_head = read_checkpoint(&checkpoint_path)?;
        let baseline = if config.emit_existing_head {
            None
        } else {
            persisted_head.or_else(|| current_head(&repo).ok().flatten().map(|c| c.head_commit))
        };

        Ok(Self {
            repo,
            emitter,
            config,
            project,
            checkpoint_path,
            last_seen_head: baseline,
        })
    }

    pub async fn run(&mut self) -> Result<(), WatcherError> {
        let mut interval = time::interval(self.config.poll_interval);
        interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

        loop {
            interval.tick().await;
            self.tick().await?;
        }
    }

    pub async fn tick(&mut self) -> Result<Option<CommitObservation>, WatcherError> {
        let observation = match current_head(&self.repo)? {
            Some(observation) => observation,
            None => return Ok(None),
        };

        if self.last_seen_head.as_deref() == Some(observation.head_commit.as_str()) {
            return Ok(None);
        }

        let event = self.build_commit_event(&observation);
        self.emitter.publish(event).await?;
        write_checkpoint(&self.checkpoint_path, &observation.head_commit)?;
        self.last_seen_head = Some(observation.head_commit.clone());

        tracing::debug!(
            project_id = %self.project.id,
            head_commit = %observation.head_commit,
            branch = observation.branch.as_deref().unwrap_or("detached"),
            file_count = observation.files.len(),
            hunk_count = observation.hunks.len(),
            "published git commit event"
        );

        Ok(Some(observation))
    }

    fn build_commit_event(&self, observation: &CommitObservation) -> EventEnvelope {
        EventEnvelope {
            id: Uuid::new_v4(),
            version: ProtocolVersion::V1,
            occurred_at: observation.occurred_at,
            project: self.project.clone(),
            session: None,
            source: self.config.source.clone(),
            payload: EventPayload::GitCommit(GitCommitPayload {
                branch: observation.branch.clone(),
                head_commit: Some(observation.head_commit.clone()),
                parent_commit: observation.parent_commit.clone(),
                commit_author_name: observation.commit_author_name.clone(),
                commit_author_email: observation.commit_author_email.clone(),
                commit_timestamp: observation.commit_timestamp,
                commit_title: observation.commit_title.clone(),
                commit_body: observation.commit_body.clone(),
                message: observation.message.clone(),
                files: observation.files.clone(),
                hunks: observation.hunks.clone(),
                dirty: false,
                staged_files: Vec::new(),
                unstaged_files: Vec::new(),
            }),
        }
    }
}

fn current_head(repo: &Repository) -> Result<Option<CommitObservation>, WatcherError> {
    let mut head = repo.head().map_err(|source| WatcherError::HeadState {
        source: anyhow::Error::new(source),
    })?;

    let branch = head.referent_name().map(|name| name.shorten().to_string());

    let head_id = head
        .try_peel_to_id()
        .map_err(|source| WatcherError::HeadState {
            source: anyhow::Error::new(source),
        })?;

    let Some(head_id) = head_id else {
        return Ok(None);
    };
    let head_commit = head_id.to_string();

    let commit = repo
        .find_commit(head_id)
        .map_err(|source| WatcherError::HeadState {
            source: anyhow::Error::new(source),
        })?;
    let message = commit.message().map_err(|source| WatcherError::HeadState {
        source: anyhow::Error::new(source),
    })?;
    let author = commit.author().map_err(|source| WatcherError::HeadState {
        source: anyhow::Error::new(source),
    })?;
    let commit_time = commit.time().map_err(|source| WatcherError::HeadState {
        source: anyhow::Error::new(source),
    })?;

    let parent_id = commit.parent_ids().next().map(ObjectId::from);
    let parent_commit = parent_id.as_ref().map(ToString::to_string);
    let new_tree = commit.tree().map_err(|source| WatcherError::HeadState {
        source: anyhow::Error::new(source),
    })?;
    let old_tree = match parent_id {
        Some(parent_id) => {
            let parent = repo
                .find_commit(parent_id)
                .map_err(|source| WatcherError::HeadState {
                    source: anyhow!(source.to_string()),
                })?;
            parent.tree().map_err(|source| WatcherError::HeadState {
                source: anyhow!(source.to_string()),
            })?
        }
        None => repo.empty_tree(),
    };

    let (files, hunks) = collect_commit_changes(repo, &old_tree, &new_tree, &head_commit)?;

    Ok(Some(CommitObservation {
        branch,
        head_commit,
        parent_commit,
        occurred_at: Utc::now(),
        commit_author_name: Some(author.name.to_string()),
        commit_author_email: Some(author.email.to_string()),
        commit_timestamp: DateTime::from_timestamp(commit_time.seconds, 0),
        commit_title: Some(message.summary().to_string()),
        commit_body: message.body.map(|body| body.to_string()),
        message: Some(commit.message_raw_sloppy().to_string()),
        files,
        hunks,
    }))
}

fn collect_commit_changes(
    repo: &Repository,
    old_tree: &gix::Tree<'_>,
    new_tree: &gix::Tree<'_>,
    _commit_oid: &str,
) -> Result<(Vec<GitCommitFilePayload>, Vec<GitCommitHunkPayload>), WatcherError> {
    let mut options = DiffOptions::default();
    options.track_path();
    options.track_rewrites(None);

    let changes = repo
        .diff_tree_to_tree(Some(old_tree), Some(new_tree), Some(options))
        .map_err(|source| WatcherError::HeadState {
            source: anyhow::Error::new(source),
        })?;

    let mut resource_cache =
        repo.diff_resource_cache_for_tree_diff()
            .map_err(|source| WatcherError::HeadState {
                source: anyhow::Error::new(source),
            })?;
    let mut file_hunk_counts: BTreeMap<String, u32> = BTreeMap::new();
    let mut files = Vec::new();
    let mut all_hunks = Vec::new();

    for detached in changes {
        let change = detached.attach(repo, repo);
        if change.entry_mode().is_tree() {
            continue;
        }

        let file_path = change.location().to_string();
        let old_path = match detached {
            gix::object::tree::diff::ChangeDetached::Rewrite {
                ref source_location,
                ..
            } => Some(source_location.to_string()),
            _ => None,
        };
        let change_kind = match detached {
            gix::object::tree::diff::ChangeDetached::Addition { .. } => "added",
            gix::object::tree::diff::ChangeDetached::Deletion { .. } => "deleted",
            gix::object::tree::diff::ChangeDetached::Modification { .. } => "modified",
            gix::object::tree::diff::ChangeDetached::Rewrite { copy: true, .. } => "copied",
            gix::object::tree::diff::ChangeDetached::Rewrite { copy: false, .. } => "renamed",
        }
        .to_string();

        let hunks = collect_hunks_for_change(&change, &mut resource_cache, &file_path)
            .map_err(|source| WatcherError::HeadState { source })?;
        file_hunk_counts.insert(file_path.clone(), hunks.len() as u32);
        all_hunks.extend(hunks);

        files.push(GitCommitFilePayload {
            path: file_path.clone(),
            old_path,
            change_kind,
            hunk_count: *file_hunk_counts.get(&file_path).unwrap_or(&0),
        });
    }

    Ok((files, all_hunks))
}

fn collect_hunks_for_change(
    change: &gix::object::tree::diff::Change<'_, '_, '_>,
    resource_cache: &mut gix::diff::blob::Platform,
    file_path: &str,
) -> Result<Vec<GitCommitHunkPayload>, anyhow::Error> {
    let platform = change.diff(resource_cache)?;
    let prep = platform.resource_cache.prepare_diff()?;
    let gix::diff::blob::platform::prepare_diff::Operation::InternalDiff { algorithm } =
        prep.operation
    else {
        return Ok(Vec::new());
    };

    let input = prep.interned_input();
    let diff = gix::diff::blob::diff_with_slider_heuristics(algorithm, &input);
    let collector = HunkCollector::default();
    let collector = gix::diff::blob::UnifiedDiff::new(
        &diff,
        &input,
        collector,
        gix::diff::blob::unified_diff::ContextSize::symmetrical(0),
    )
    .consume()?;

    Ok(collector
        .hunks
        .into_iter()
        .map(|mut hunk| {
            hunk.file_path = file_path.to_string();
            hunk
        })
        .collect())
}

impl gix::diff::blob::unified_diff::ConsumeHunk for HunkCollector {
    type Out = Self;

    fn consume_hunk(
        &mut self,
        header: gix::diff::blob::unified_diff::HunkHeader,
        lines: &[(gix::diff::blob::unified_diff::DiffLineKind, &[u8])],
    ) -> std::io::Result<()> {
        let mut added = Vec::new();
        let mut removed = Vec::new();
        let mut context = Vec::new();

        for (kind, line) in lines {
            let text = String::from_utf8_lossy(line)
                .trim_end_matches('\n')
                .to_string();
            match kind {
                gix::diff::blob::unified_diff::DiffLineKind::Add => added.push(text),
                gix::diff::blob::unified_diff::DiffLineKind::Remove => removed.push(text),
                gix::diff::blob::unified_diff::DiffLineKind::Context => context.push(text),
            }
        }

        let context_before_hash = context
            .first()
            .map(|line| stable_fingerprint([line.as_str()]));
        let context_after_hash = context
            .last()
            .map(|line| stable_fingerprint([line.as_str()]));

        self.hunks.push(GitCommitHunkPayload {
            id: Uuid::new_v4(),
            file_path: String::new(),
            old_start: header.before_hunk_start,
            old_lines: header.before_hunk_len,
            new_start: header.after_hunk_start,
            new_lines: header.after_hunk_len,
            hunk_header: context.first().cloned(),
            added_line_count: added.len() as u32,
            removed_line_count: removed.len() as u32,
            context_before_hash,
            context_after_hash,
            added_lines_fingerprint: (!added.is_empty())
                .then(|| stable_fingerprint(added.iter().map(String::as_str))),
            removed_lines_fingerprint: (!removed.is_empty())
                .then(|| stable_fingerprint(removed.iter().map(String::as_str))),
        });
        Ok(())
    }

    fn finish(self) -> Self::Out {
        self
    }
}

fn stable_fingerprint<'a>(lines: impl IntoIterator<Item = &'a str>) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for line in lines {
        normalize_line(line).hash(&mut hasher);
    }
    format!("{:016x}", hasher.finish())
}

fn normalize_line(line: &str) -> String {
    line.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn read_checkpoint(path: &Path) -> Result<Option<String>, WatcherError> {
    let data = match fs::read_to_string(path) {
        Ok(data) => data,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(WatcherError::HeadState {
                source: anyhow::Error::new(error),
            });
        }
    };

    let checkpoint: WatcherCheckpoint =
        serde_json::from_str(&data).map_err(|source| WatcherError::HeadState {
            source: anyhow::Error::new(source),
        })?;
    Ok(Some(checkpoint.head_commit))
}

fn write_checkpoint(path: &Path, head_commit: &str) -> Result<(), WatcherError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| WatcherError::HeadState {
            source: anyhow::Error::new(source),
        })?;
    }

    let payload = serde_json::to_vec_pretty(&WatcherCheckpoint {
        head_commit: head_commit.to_string(),
    })
    .map_err(|source| WatcherError::HeadState {
        source: anyhow::Error::new(source),
    })?;

    fs::write(path, payload).map_err(|source| WatcherError::HeadState {
        source: anyhow::Error::new(source),
    })
}

fn default_project_id(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| path.display().to_string())
}

fn repository_fingerprint(repo: &Repository, work_dir: &Path) -> Option<String> {
    repository_identity_source(repo, work_dir).map(|identity| sha256_fingerprint(&identity))
}

fn repository_identity_source(repo: &Repository, work_dir: &Path) -> Option<String> {
    let config = repo.config_snapshot();
    if let Some(remote) = config.string("remote.origin.url") {
        let remote = remote.to_string();
        if !remote.is_empty() {
            return Some(format!("remote:{}", normalize_remote_url(&remote)));
        }
    }

    repository_local_id(work_dir).map(|repository_id| format!("local:{repository_id}"))
}

fn repository_local_id(work_dir: &Path) -> Option<String> {
    let id_path = work_dir.join(".promptwho/repository-id");
    match fs::read_to_string(&id_path) {
        Ok(existing) => {
            let existing = existing.trim();
            if existing.is_empty() {
                None
            } else {
                Some(existing.to_string())
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let repository_id = Uuid::new_v4().to_string();
            let parent = id_path.parent()?;
            if fs::create_dir_all(parent).is_err() {
                return None;
            }

            match fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&id_path)
            {
                Ok(mut file) => {
                    if file.write_all(repository_id.as_bytes()).is_err() {
                        return None;
                    }
                    if file.write_all(b"\n").is_err() {
                        return None;
                    }
                    Some(repository_id)
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    fs::read_to_string(&id_path)
                        .ok()
                        .map(|existing| existing.trim().to_string())
                        .filter(|existing| !existing.is_empty())
                }
                Err(_) => None,
            }
        }
        Err(_) => None,
    }
}

fn sha256_fingerprint(value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        encoded.push_str(&format!("{byte:02x}"));
    }
    format!("git:{encoded}")
}

fn normalize_remote_url(remote: &str) -> String {
    let trimmed = remote.trim().trim_end_matches(".git").to_ascii_lowercase();
    if let Some(rest) = trimmed.strip_prefix("git@")
        && let Some((host, path)) = rest.split_once(':')
    {
        return format!("ssh://git@{host}/{path}");
    }

    if let Some(rest) = trimmed.strip_prefix("http://") {
        return format!("https://{rest}");
    }

    trimmed
}

#[cfg(test)]
mod tests {
    use super::{
        default_project_id, normalize_line, normalize_remote_url, read_checkpoint,
        repository_local_id, sha256_fingerprint, stable_fingerprint, write_checkpoint,
    };
    use std::path::Path;
    use tempfile::tempdir;

    #[test]
    fn default_project_id_prefers_leaf_directory_name() {
        assert_eq!(default_project_id(Path::new("/tmp/promptwho")), "promptwho");
    }

    #[test]
    fn normalize_remote_url_collapses_common_git_forms() {
        assert_eq!(
            normalize_remote_url("git@github.com:PromptWho/Repo.git"),
            "ssh://git@github.com/promptwho/repo"
        );
        assert_eq!(
            normalize_remote_url("http://github.com/PromptWho/Repo.git"),
            "https://github.com/promptwho/repo"
        );
    }

    #[test]
    fn repository_local_id_persists_for_reuse() {
        let dir = tempdir().expect("tempdir should exist");

        let first = repository_local_id(dir.path()).expect("repository id should be created");
        let second = repository_local_id(dir.path()).expect("repository id should be reused");

        assert_eq!(first, second);
    }

    #[test]
    fn sha256_fingerprint_is_stable() {
        assert_eq!(
            sha256_fingerprint("remote:https://github.com/promptwho/repo"),
            "git:52faa9537af34d9b54f5510cfbd0fdbc2f5d9e84226b1a6e3c3d7f8f0f9939d4"
        );
    }

    #[test]
    fn checkpoint_round_trips_head_commit() {
        let dir = tempdir().expect("tempdir should exist");
        let path = dir.path().join(".promptwho/git-watcher.json");

        write_checkpoint(&path, "abc123").expect("checkpoint write should succeed");

        assert_eq!(
            read_checkpoint(&path).expect("checkpoint read should succeed"),
            Some("abc123".to_string())
        );
    }

    #[test]
    fn fingerprints_ignore_whitespace_shape() {
        assert_eq!(normalize_line("  let   x = 1;  "), "let x = 1;");
        assert_eq!(
            stable_fingerprint(["let x = 1;", "return x;"]),
            stable_fingerprint([" let   x = 1; ", "return   x;"])
        );
    }
}
