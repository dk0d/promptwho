use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_msgpack::MsgPack;
use promptwho_protocol::ErrorResponse;

use crate::extractors::ResponseFormat;

#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("invalid msgpack payload: {message}")]
    Decode {
        message: String,
        response_format: ResponseFormat,
    },
    #[error("{error}")]
    Store {
        error: promptwho_core::ingest::IngestError,
        response_format: ResponseFormat,
    },
    #[error("{0}")]
    Query(String),
}

impl ServerError {
    pub fn invalid_msgpack_payload(
        message: impl Into<String>,
        response_format: ResponseFormat,
    ) -> Self {
        Self::Decode {
            message: message.into(),
            response_format,
        }
    }

    pub fn invalid_msgpack_content_type(response_format: ResponseFormat) -> Self {
        Self::Decode {
            message: "Expected request with `Content-Type: application/msgpack`".to_string(),
            response_format,
        }
    }

    pub fn store(
        error: promptwho_core::ingest::IngestError,
        response_format: ResponseFormat,
    ) -> Self {
        Self::Store {
            error,
            response_format,
        }
    }

    pub fn query(error: promptwho_storage::StoreError) -> Self {
        Self::Query(error.to_string())
    }
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let (status, code, message, response_format) = match self {
            ServerError::Decode {
                message,
                response_format,
            } => (
                StatusCode::BAD_REQUEST,
                "invalid_msgpack",
                message,
                response_format,
            ),
            ServerError::Store {
                error,
                response_format,
            } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "store_error",
                error.to_string(),
                response_format,
            ),
            ServerError::Query(message) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "query_error",
                message,
                ResponseFormat::Json,
            ),
        };

        let body = ErrorResponse {
            code: code.to_string(),
            message,
        };

        match response_format {
            ResponseFormat::Json => (status, Json(body)).into_response(),
            ResponseFormat::MsgPack => (status, MsgPack(body)).into_response(),
        }
    }
}
