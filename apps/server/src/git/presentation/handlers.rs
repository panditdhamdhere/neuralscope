use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;
use uuid::Uuid;

use crate::api::state::AppState;
use crate::auth::application::{ensure_project_member, ensure_project_writer};
use crate::auth::presentation::AuthUser;
use crate::git::application::{
    CommitListQuery, CreateCommitRequest, CreateDeploymentRequest, DeploymentListQuery, GitService,
};
use crate::git::domain::{Commit, Deployment};
use crate::AppError;

#[derive(Serialize)]
pub struct ListResponse<T> {
    pub data: Vec<T>,
    pub meta: ListMeta,
}

#[derive(Serialize)]
pub struct ListMeta {
    pub total: usize,
}

/// `GET /api/v1/projects/:project_id/git/commits`
pub async fn list_commits(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Query(query): Query<CommitListQuery>,
) -> Result<Json<ListResponse<CommitResponse>>, AppError> {
    ensure_project_member(&state.db, auth.user.id, project_id).await?;

    let commits = GitService::new(&state.db)
        .list_commits(project_id, query)
        .await?;
    let total = commits.len();

    Ok(Json(ListResponse {
        data: commits.into_iter().map(CommitResponse::from).collect(),
        meta: ListMeta { total },
    }))
}

/// `POST /api/v1/projects/:project_id/git/commits`
pub async fn ingest_commit(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Json(body): Json<CreateCommitRequest>,
) -> Result<(StatusCode, Json<CommitResponse>), AppError> {
    ensure_project_writer(&state.db, auth.user.id, project_id).await?;

    let commit = GitService::new(&state.db)
        .ingest_commit(project_id, body)
        .await?;

    Ok((StatusCode::CREATED, Json(CommitResponse::from(commit))))
}

/// `GET /api/v1/projects/:project_id/git/commits/:sha`
pub async fn get_commit(
    auth: AuthUser,
    State(state): State<AppState>,
    Path((project_id, sha)): Path<(Uuid, String)>,
) -> Result<Json<CommitResponse>, AppError> {
    ensure_project_member(&state.db, auth.user.id, project_id).await?;

    let commit = GitService::new(&state.db)
        .get_commit(project_id, &sha)
        .await?;
    Ok(Json(CommitResponse::from(commit)))
}

/// `GET /api/v1/projects/:project_id/git/deployments`
pub async fn list_deployments(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Query(query): Query<DeploymentListQuery>,
) -> Result<Json<ListResponse<crate::git::application::DeploymentWithCommit>>, AppError> {
    ensure_project_member(&state.db, auth.user.id, project_id).await?;

    let deployments = GitService::new(&state.db)
        .list_deployments(project_id, query)
        .await?;
    let total = deployments.len();

    Ok(Json(ListResponse {
        data: deployments,
        meta: ListMeta { total },
    }))
}

/// `POST /api/v1/projects/:project_id/git/deployments`
pub async fn ingest_deployment(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Json(body): Json<CreateDeploymentRequest>,
) -> Result<(StatusCode, Json<DeploymentResponse>), AppError> {
    ensure_project_writer(&state.db, auth.user.id, project_id).await?;

    let deployment = GitService::new(&state.db)
        .ingest_deployment(project_id, body)
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(DeploymentResponse::from(deployment)),
    ))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitResponse {
    id: Uuid,
    project_id: Uuid,
    sha: String,
    author: String,
    message: String,
    branch: String,
    committed_at: chrono::DateTime<chrono::Utc>,
}

impl From<Commit> for CommitResponse {
    fn from(commit: Commit) -> Self {
        Self {
            id: commit.id,
            project_id: commit.project_id,
            sha: commit.sha,
            author: commit.author,
            message: commit.message,
            branch: commit.branch,
            committed_at: commit.committed_at,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentResponse {
    id: Uuid,
    project_id: Uuid,
    commit_sha: String,
    environment: String,
    deployed_by: Option<String>,
    deployed_at: chrono::DateTime<chrono::Utc>,
}

impl From<Deployment> for DeploymentResponse {
    fn from(deployment: Deployment) -> Self {
        Self {
            id: deployment.id,
            project_id: deployment.project_id,
            commit_sha: deployment.commit_sha,
            environment: deployment.environment,
            deployed_by: deployment.deployed_by,
            deployed_at: deployment.deployed_at,
        }
    }
}
