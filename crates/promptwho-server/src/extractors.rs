use axum::{
    body::{Body, Bytes},
    extract::{FromRequest, Request},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::{Serialize, de::DeserializeOwned};
use std::ops::{Deref, DerefMut};

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
        let response_format = preferred_response_format(&req);

        if !has_msgpack_content_type(&req) {
            return Err(ServerError::invalid_msgpack_content_type(response_format));
        }

        let bytes = Bytes::from_request(req, state).await.map_err(|err| {
            ServerError::invalid_msgpack_payload(err.to_string(), response_format)
        })?;

        let value = rmp_serde::from_slice(&bytes).map_err(|err| {
            ServerError::invalid_msgpack_payload(err.to_string(), response_format)
        })?;

        match response_format {
            ResponseFormat::Json => Ok(ServerMsg(MsgPackOrJson::Json(value))),
            ResponseFormat::MsgPack => Ok(ServerMsg(MsgPackOrJson::MsgPack(value))),
        }
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
