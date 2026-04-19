pub mod errors;
pub mod extractors;
pub mod router;
pub mod routes;
pub mod state;

use anyhow::Result;
use promptwho_core::{PromptwhoConfig, StorageConfig as RuntimeStorageConfig};
use promptwho_storage_surreal::{SurrealConfig, SurrealStore};
use std::sync::Arc;

pub use self::router::build_router;
pub use self::state::AppState;

pub async fn run(config: &PromptwhoConfig) -> Result<()> {
    let listen_addr = config.server.listen_addr();
    let (store, endpoint) = match &config.storage {
        RuntimeStorageConfig::Surreal { config: surreal } => (
            Arc::new(
                SurrealStore::connect(SurrealConfig {
                    endpoint: surreal.endpoint.clone(),
                    namespace: surreal.namespace.clone(),
                    database: surreal.database.clone(),
                    username: surreal.username.clone(),
                    password: surreal.password.clone(),
                    vector_enabled: surreal.vector_enabled,
                    sync_enabled: surreal.sync_enabled,
                })
                .await?,
            ),
            surreal.endpoint.clone(),
        ),
    };
    let listener = tokio::net::TcpListener::bind(listen_addr).await?;
    tracing::info!(addr = %listen_addr, endpoint = %endpoint, "promptwho-server listening");
    let state = AppState { store };
    axum::serve(listener, build_router(state)).await?;
    Ok(())
}
