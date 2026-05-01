use axum::{
    body::{Body, Bytes},
    extract::{FromRequest, Request},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use rmp_serde::decode::Error as MsgPackDecodeError;
use serde::{Serialize, de::DeserializeOwned};
use std::ops::{Deref, DerefMut};
use tracing::Level;

use crate::errors::ServerError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseFormat {
    MsgPack,
    Json,
}

#[derive(Debug, Clone)]
pub enum MsgPackOrJson<T> {
    MsgPack(T),
    Json(T),
}

impl<T> MsgPackOrJson<T> {
    pub fn response_format(&self) -> ResponseFormat {
        match self {
            MsgPackOrJson::MsgPack(_) => ResponseFormat::MsgPack,
            MsgPackOrJson::Json(_) => ResponseFormat::Json,
        }
    }
}
impl<T> AsRef<T> for MsgPackOrJson<T> {
    fn as_ref(&self) -> &T {
        match self {
            MsgPackOrJson::MsgPack(value) | MsgPackOrJson::Json(value) => value,
        }
    }
}

impl<T> Deref for MsgPackOrJson<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            MsgPackOrJson::MsgPack(value) | MsgPackOrJson::Json(value) => value,
        }
    }
}

impl<T> DerefMut for MsgPackOrJson<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            MsgPackOrJson::MsgPack(value) | MsgPackOrJson::Json(value) => value,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ServerMsg<T>(pub MsgPackOrJson<T>);

impl<T> Deref for ServerMsg<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<T> DerefMut for ServerMsg<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.deref_mut()
    }
}

impl<S, T> FromRequest<S> for ServerMsg<T>
where
    S: Send + Sync,
    T: DeserializeOwned,
{
    type Rejection = ServerError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let content_type = header_value(req.headers(), header::CONTENT_TYPE);
        let accept = header_value(req.headers(), header::ACCEPT);
        let response_format = preferred_response_format(&req);

        if !has_msgpack_content_type(&req) {
            tracing::warn!(
                content_type = content_type.as_deref().unwrap_or("<missing>"),
                accept = accept.as_deref().unwrap_or("<missing>"),
                "Rejected ingest request with unsupported content type"
            );
            return Err(ServerError::invalid_msgpack_content_type(response_format));
        }

        let bytes = Bytes::from_request(req, state).await.map_err(|err| {
            tracing::warn!(
                content_type = content_type.as_deref().unwrap_or("<missing>"),
                accept = accept.as_deref().unwrap_or("<missing>"),
                error = %err,
                "Failed to read MsgPack request body"
            );
            ServerError::invalid_msgpack_payload(err.to_string(), response_format)
        })?;

        let value = rmp_serde::from_slice(&bytes).map_err(|err| {
            log_decode_error(&err, &bytes, content_type.as_deref(), accept.as_deref());
            ServerError::invalid_msgpack_payload(err.to_string(), response_format)
        })?;

        match response_format {
            ResponseFormat::Json => Ok(ServerMsg(MsgPackOrJson::Json(value))),
            ResponseFormat::MsgPack => Ok(ServerMsg(MsgPackOrJson::MsgPack(value))),
        }
    }
}

fn log_decode_error(
    error: &MsgPackDecodeError,
    bytes: &[u8],
    content_type: Option<&str>,
    accept: Option<&str>,
) {
    tracing::warn!(
        content_type = content_type.unwrap_or("<missing>"),
        accept = accept.unwrap_or("<missing>"),
        body_len = bytes.len(),
        error = %error,
        error_kind = classify_decode_error(error),
        "Failed to decode MsgPack ingest request"
    );

    if tracing::enabled!(Level::TRACE) {
        tracing::trace!(
            body_preview_hex = %preview_hex(bytes, 96),
            body_preview_utf8 = preview_utf8(bytes, 96).as_deref().unwrap_or("<non-utf8>"),
            "MsgPack ingest payload preview"
        );
    }
}

fn classify_decode_error(error: &MsgPackDecodeError) -> &'static str {
    match error {
        MsgPackDecodeError::InvalidMarkerRead(_) | MsgPackDecodeError::InvalidDataRead(_) => {
            "malformed_msgpack"
        }
        MsgPackDecodeError::TypeMismatch(_)
        | MsgPackDecodeError::OutOfRange
        | MsgPackDecodeError::LengthMismatch(_)
        | MsgPackDecodeError::Utf8Error(_)
        | MsgPackDecodeError::Uncategorized(_) => "schema_mismatch",
        MsgPackDecodeError::Syntax(_) => "invalid_value",
        MsgPackDecodeError::DepthLimitExceeded => "depth_limit_exceeded",
    }
}

fn header_value(headers: &axum::http::HeaderMap, name: header::HeaderName) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned)
}

fn preview_hex(bytes: &[u8], limit: usize) -> String {
    let preview = &bytes[..bytes.len().min(limit)];
    let mut rendered = preview
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join(" ");

    if bytes.len() > limit {
        rendered.push_str(" ...");
    }

    rendered
}

fn preview_utf8(bytes: &[u8], limit: usize) -> Option<String> {
    let preview = &bytes[..bytes.len().min(limit)];
    let rendered = String::from_utf8(preview.to_vec()).ok()?;

    if bytes.len() > limit {
        Some(format!("{rendered}..."))
    } else {
        Some(rendered)
    }
}

pub fn preferred_response_format<B>(req: &Request<B>) -> ResponseFormat {
    let Some(accept) = req.headers().get(header::ACCEPT) else {
        return ResponseFormat::MsgPack;
    };

    let Ok(accept) = accept.to_str() else {
        return ResponseFormat::MsgPack;
    };

    if accept
        .split(',')
        .map(str::trim)
        .any(|value| value == "application/json")
    {
        ResponseFormat::Json
    } else {
        ResponseFormat::MsgPack
    }
}

fn has_msgpack_content_type<B>(req: &Request<B>) -> bool {
    let Some(content_type) = req.headers().get(header::CONTENT_TYPE) else {
        return false;
    };

    let Ok(content_type) = content_type.to_str() else {
        return false;
    };

    matches!(
        content_type,
        "application/msgpack" | "application/x-msgpack"
    ) || content_type.ends_with("+msgpack")
}

impl<T> IntoResponse for ServerMsg<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        match self.0 {
            MsgPackOrJson::Json(value) => {
                let string = match serde_json::to_string(&value) {
                    Ok(res) => res,
                    Err(err) => {
                        return Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .header(header::CONTENT_TYPE, "text/plain")
                            .body(Body::new(err.to_string()))
                            .unwrap();
                    }
                };

                let mut res = string.into_response();
                res.headers_mut().insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("application/json"),
                );
                res
            }
            MsgPackOrJson::MsgPack(value) => {
                let bytes = match rmp_serde::encode::to_vec_named(&value) {
                    Ok(res) => res,
                    Err(err) => {
                        return Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .header(header::CONTENT_TYPE, "text/plain")
                            .body(Body::new(err.to_string()))
                            .unwrap();
                    }
                };
                let mut res = bytes.into_response();
                res.headers_mut().insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("application/msgpack"),
                );
                res
            }
        }
    }
}
