use async_trait::async_trait;
use promptwho_protocol::{EventEnvelope, IngestEventsRequest};
use reqwest::{Client, StatusCode};
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum PublishError {
    #[error("failed to encode msgpack request")]
    Encode(#[source] rmp_serde::encode::Error),
    #[error("failed to publish event request")]
    Http(#[from] reqwest::Error),
    #[error("server rejected ingest request with status {status}")]
    UnexpectedStatus { status: StatusCode },
}

#[async_trait]
pub trait EventEmitter: Send + Sync {
    async fn publish(&self, event: EventEnvelope) -> Result<(), PublishError>;
}

#[derive(Debug, Clone)]
pub struct HttpEventEmitter {
    client: Client,
    events_url: String,
}

impl HttpEventEmitter {
    pub fn new(server_base_url: impl AsRef<str>) -> Self {
        let base = server_base_url.as_ref().trim_end_matches('/');
        Self {
            client: Client::new(),
            events_url: format!("{base}/v1/events"),
        }
    }

    pub fn with_client(server_base_url: impl AsRef<str>, client: Client) -> Self {
        let base = server_base_url.as_ref().trim_end_matches('/');
        Self {
            client,
            events_url: format!("{base}/v1/events"),
        }
    }
}

#[async_trait]
impl EventEmitter for HttpEventEmitter {
    async fn publish(&self, event: EventEnvelope) -> Result<(), PublishError> {
        let body = rmp_serde::to_vec_named(&IngestEventsRequest {
            request_id: Uuid::new_v4(),
            events: vec![event],
        })
        .map_err(PublishError::Encode)?;

        let response = self
            .client
            .post(&self.events_url)
            .header(reqwest::header::CONTENT_TYPE, "application/msgpack")
            .header(reqwest::header::ACCEPT, "application/msgpack")
            .body(body)
            .send()
            .await?;

        if response.status() != StatusCode::ACCEPTED {
            return Err(PublishError::UnexpectedStatus {
                status: response.status(),
            });
        }

        Ok(())
    }
}
