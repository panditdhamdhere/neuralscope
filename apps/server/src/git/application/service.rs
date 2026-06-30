//! Git commit and deployment use cases.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::git::domain::{Commit, Deployment};
use crate::AppError;

/// Manages Git commit history and deployment correlation.
pub struct GitService<'a> {
    pool: &'a PgPool,
}

impl<'a> GitService<'a> {
    #[must_use]
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Records a Git commit for a project.
    pub async fn ingest_commit(
        &self,
        project_id: Uuid,
        body: CreateCommitRequest,
    ) -> Result<Commit, AppError> {
        if body.sha.len() < 7 {
            return Err(AppError::Validation(
                "commit sha must be at least 7 characters".into(),
            ));
        }

        let committed_at = body.committed_at.unwrap_or_else(Utc::now);
        let branch = body.branch.unwrap_or_else(|| "main".into());

        let row = sqlx::query_as::<_, CommitRow>(
            r#"
            INSERT INTO git_commits (project_id, sha, author, message, branch, committed_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (project_id, sha) DO UPDATE
                SET author = EXCLUDED.author,
                    message = EXCLUDED.message,
                    branch = EXCLUDED.branch,
                    committed_at = EXCLUDED.committed_at
            RETURNING id, project_id, sha, author, message, branch, committed_at
            "#,
        )
        .bind(project_id)
        .bind(&body.sha)
        .bind(&body.author)
        .bind(&body.message)
        .bind(&branch)
        .bind(committed_at)
        .fetch_one(self.pool)
        .await?;

        Ok(row.into())
    }

    /// Records a deployment linked to a commit SHA.
    pub async fn ingest_deployment(
        &self,
        project_id: Uuid,
        body: CreateDeploymentRequest,
    ) -> Result<Deployment, AppError> {
        if body.commit_sha.len() < 7 {
            return Err(AppError::Validation(
                "commit_sha must be at least 7 characters".into(),
            ));
        }

        let row = sqlx::query_as::<_, DeploymentRow>(
            r#"
            INSERT INTO deployments (project_id, commit_sha, environment, deployed_by, deployed_at)
            VALUES ($1, $2, $3, $4, COALESCE($5, NOW()))
            RETURNING id, project_id, commit_sha, environment, deployed_by, deployed_at
            "#,
        )
        .bind(project_id)
        .bind(&body.commit_sha)
        .bind(&body.environment)
        .bind(&body.deployed_by)
        .bind(body.deployed_at)
        .fetch_one(self.pool)
        .await?;

        Ok(row.into())
    }

    /// Lists commits for a project, newest first.
    pub async fn list_commits(
        &self,
        project_id: Uuid,
        query: CommitListQuery,
    ) -> Result<Vec<Commit>, AppError> {
        let limit = query.limit.unwrap_or(50).clamp(1, 100);

        let rows = sqlx::query_as::<_, CommitRow>(
            r#"
            SELECT id, project_id, sha, author, message, branch, committed_at
            FROM git_commits
            WHERE project_id = $1
              AND ($2::text IS NULL OR branch = $2)
              AND ($3::text IS NULL OR message ILIKE '%' || $3 || '%' OR author ILIKE '%' || $3 || '%')
            ORDER BY committed_at DESC
            LIMIT $4
            "#,
        )
        .bind(project_id)
        .bind(query.branch)
        .bind(query.search)
        .bind(limit)
        .fetch_all(self.pool)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// Fetches a single commit by SHA prefix or full SHA.
    pub async fn get_commit(&self, project_id: Uuid, sha: &str) -> Result<Commit, AppError> {
        let row = sqlx::query_as::<_, CommitRow>(
            r#"
            SELECT id, project_id, sha, author, message, branch, committed_at
            FROM git_commits
            WHERE project_id = $1 AND sha LIKE $2 || '%'
            ORDER BY length(sha) ASC
            LIMIT 1
            "#,
        )
        .bind(project_id)
        .bind(sha)
        .fetch_optional(self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Commit '{sha}' not found")))?;

        Ok(row.into())
    }

    /// Lists deployments with optional commit metadata.
    pub async fn list_deployments(
        &self,
        project_id: Uuid,
        query: DeploymentListQuery,
    ) -> Result<Vec<DeploymentWithCommit>, AppError> {
        let limit = query.limit.unwrap_or(50).clamp(1, 100);

        let rows = sqlx::query_as::<_, DeploymentWithCommitRow>(
            r#"
            SELECT
                d.id,
                d.project_id,
                d.commit_sha,
                d.environment,
                d.deployed_by,
                d.deployed_at,
                c.message AS commit_message,
                c.author AS commit_author,
                c.branch AS commit_branch
            FROM deployments d
            LEFT JOIN git_commits c
                ON c.project_id = d.project_id AND c.sha = d.commit_sha
            WHERE d.project_id = $1
              AND ($2::text IS NULL OR d.environment = $2)
            ORDER BY d.deployed_at DESC
            LIMIT $3
            "#,
        )
        .bind(project_id)
        .bind(query.environment)
        .bind(limit)
        .fetch_all(self.pool)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateCommitRequest {
    pub sha: String,
    pub author: String,
    pub message: String,
    pub branch: Option<String>,
    pub committed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDeploymentRequest {
    pub commit_sha: String,
    pub environment: String,
    pub deployed_by: Option<String>,
    pub deployed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CommitListQuery {
    pub branch: Option<String>,
    pub search: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct DeploymentListQuery {
    pub environment: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentWithCommit {
    pub id: Uuid,
    pub project_id: Uuid,
    pub commit_sha: String,
    pub environment: String,
    pub deployed_by: Option<String>,
    pub deployed_at: DateTime<Utc>,
    pub commit_message: Option<String>,
    pub commit_author: Option<String>,
    pub commit_branch: Option<String>,
}

#[derive(sqlx::FromRow)]
struct CommitRow {
    id: Uuid,
    project_id: Uuid,
    sha: String,
    author: String,
    message: String,
    branch: String,
    committed_at: DateTime<Utc>,
}

impl From<CommitRow> for Commit {
    fn from(row: CommitRow) -> Self {
        Self {
            id: row.id,
            project_id: row.project_id,
            sha: row.sha,
            author: row.author,
            message: row.message,
            branch: row.branch,
            committed_at: row.committed_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct DeploymentRow {
    id: Uuid,
    project_id: Uuid,
    commit_sha: String,
    environment: String,
    deployed_by: Option<String>,
    deployed_at: DateTime<Utc>,
}

impl From<DeploymentRow> for Deployment {
    fn from(row: DeploymentRow) -> Self {
        Self {
            id: row.id,
            project_id: row.project_id,
            commit_sha: row.commit_sha,
            environment: row.environment,
            deployed_at: row.deployed_at,
            deployed_by: row.deployed_by,
        }
    }
}

#[derive(sqlx::FromRow)]
struct DeploymentWithCommitRow {
    id: Uuid,
    project_id: Uuid,
    commit_sha: String,
    environment: String,
    deployed_by: Option<String>,
    deployed_at: DateTime<Utc>,
    commit_message: Option<String>,
    commit_author: Option<String>,
    commit_branch: Option<String>,
}

impl From<DeploymentWithCommitRow> for DeploymentWithCommit {
    fn from(row: DeploymentWithCommitRow) -> Self {
        Self {
            id: row.id,
            project_id: row.project_id,
            commit_sha: row.commit_sha,
            environment: row.environment,
            deployed_by: row.deployed_by,
            deployed_at: row.deployed_at,
            commit_message: row.commit_message,
            commit_author: row.commit_author,
            commit_branch: row.commit_branch,
        }
    }
}
