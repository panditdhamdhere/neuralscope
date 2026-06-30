use axum::{
    routing::{get, post},
    Router,
};

use crate::api::state::AppState;

use super::handlers;

/// Git routes under `/api/v1/projects/:project_id/git`.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/projects/{project_id}/git/commits",
            get(handlers::list_commits).post(handlers::ingest_commit),
        )
        .route(
            "/projects/{project_id}/git/commits/{sha}",
            get(handlers::get_commit),
        )
        .route(
            "/projects/{project_id}/git/deployments",
            get(handlers::list_deployments).post(handlers::ingest_deployment),
        )
}
