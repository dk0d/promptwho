use axum::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use promptwho_protocol::HealthResponse;
use tracing::instrument;

#[utoipa::path(
    get,
    path = "/readyz",
    tag="Health",
    // responses((status = 200, description = "Server health", body = HealthResponse))
)]
#[instrument]
pub async fn readyz() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(HealthResponse {
            status: "ok".to_string(),
        }),
    )
        .into_response()
}
