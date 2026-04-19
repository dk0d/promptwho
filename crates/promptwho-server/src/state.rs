use std::sync::Arc;

use promptwho_storage_surreal::SurrealStore;

#[derive(Clone, Debug)]
pub struct AppState {
    pub store: Arc<SurrealStore>,
}
