use axum::{Router, response::Html, routing::get};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_scalar::{Scalar, Servable};

use crate::state::AppState;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "PromptWho API",
        // read from Cargo.toml .version
        version = env!("CARGO_PKG_VERSION"),
        description = "Promptwho is an LLM observability and attribution platform.",
        contact(
            name = "Daniel Capecci",
            url = "https://promptwho.io",
        ),
        license(
            name = "Proprietary"
        )
    ),
    tags((name = "promptwho-server", description = "Local promptwho ingest and query server"))
)]
struct ApiDoc;

pub fn build_app_router(state: AppState) -> OpenApiRouter {
    OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(
            crate::routes::health::readyz,
            crate::routes::events::ingest_events
        ))
        .with_state(state)
}

pub fn build_router(state: AppState) -> Router {
    let (router, api) = build_app_router(state).split_for_parts();
    router
        .route(
            "/docs",
            get({
                let scalar = Scalar::with_url("/docs", api.clone());
                move || async move { Html(scalar.to_html()) }
            }),
        )
        .layer(TraceLayer::new_for_http())
}
