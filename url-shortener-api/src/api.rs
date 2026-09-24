use axum::{
    Json, Router,
    body::Body,
    extract::{Path, State, rejection::JsonRejection},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tower_http::{limit::RequestBodyLimitLayer, trace::TraceLayer};
use tracing::warn;

use crate::short_link::{
    domain::{DestinationUrl, ShortCode},
    service::{ShortLinkRepository, ShortLinkService},
};

#[derive(Clone)]
struct ApiState<R> {
    links: ShortLinkService<R>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateLinkRequest {
    destination_url: String,
}

#[derive(Serialize)]
struct CreateLinkResponse {
    code: String,
    short_path: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: ErrorDetails,
}

#[derive(Serialize)]
struct ErrorDetails {
    code: &'static str,
    message: &'static str,
}

pub fn router<R>(links: ShortLinkService<R>) -> Router
where
    R: ShortLinkRepository + Clone + 'static,
{
    Router::new()
        .route("/api/v1/links", post(create_link::<R>))
        .route("/health/live", get(liveness))
        .route("/health/ready", get(readiness::<R>))
        .route("/{code}", get(resolve_link::<R>))
        .layer(RequestBodyLimitLayer::new(4096))
        .layer(TraceLayer::new_for_http())
        .with_state(ApiState { links })
}

async fn create_link<R>(
    State(state): State<ApiState<R>>,
    body: Result<Json<CreateLinkRequest>, JsonRejection>,
) -> Response<Body>
where
    R: ShortLinkRepository + Clone + 'static,
{
    let Json(request) = match body {
        Ok(request) => request,
        Err(_) => {
            return api_error(
                StatusCode::BAD_REQUEST,
                "invalid_json",
                "Request body must be a JSON object with destination_url.",
            );
        }
    };

    let destination = match DestinationUrl::parse(&request.destination_url) {
        Ok(destination) => destination,
        Err(_) => {
            return api_error(
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid_destination_url",
                "Destination URL must be HTTP(S), contain no credentials, and be at most 2048 bytes.",
            );
        }
    };

    let result = match state.links.create(&destination).await {
        Ok(result) => result,
        Err(_) => {
            warn!(operation = "create", "short-link storage operation failed");
            return api_error(
                StatusCode::SERVICE_UNAVAILABLE,
                "storage_unavailable",
                "Short links are temporarily unavailable. Try again later.",
            );
        }
    };

    let status = if result.was_created {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };
    let response = CreateLinkResponse {
        short_path: format!("/{}", result.code),
        code: result.code.to_string(),
    };
    (status, Json(response)).into_response()
}

async fn resolve_link<R>(
    State(state): State<ApiState<R>>,
    Path(value): Path<String>,
) -> Response<Body>
where
    R: ShortLinkRepository + Clone + 'static,
{
    let code = match ShortCode::parse(&value) {
        Ok(code) => code,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };

    let destination = match state.links.resolve(&code).await {
        Ok(Some(destination)) => destination,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(_) => {
            warn!(operation = "resolve", "short-link storage operation failed");
            return api_error(
                StatusCode::SERVICE_UNAVAILABLE,
                "storage_unavailable",
                "Short links are temporarily unavailable. Try again later.",
            );
        }
    };

    let location = match HeaderValue::try_from(destination.as_str()) {
        Ok(location) => location,
        Err(_) => {
            warn!(
                operation = "resolve",
                "stored destination could not be represented as a Location header"
            );
            return api_error(
                StatusCode::SERVICE_UNAVAILABLE,
                "storage_unavailable",
                "Short links are temporarily unavailable. Try again later.",
            );
        }
    };

    let mut response = StatusCode::MOVED_PERMANENTLY.into_response();
    response.headers_mut().insert(header::LOCATION, location);
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=3600"),
    );
    response
}

async fn liveness() -> StatusCode {
    StatusCode::NO_CONTENT
}

async fn readiness<R>(State(state): State<ApiState<R>>) -> StatusCode
where
    R: ShortLinkRepository + Clone + 'static,
{
    match state.links.is_ready().await {
        Ok(()) => StatusCode::NO_CONTENT,
        Err(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}

fn api_error(status: StatusCode, code: &'static str, message: &'static str) -> Response<Body> {
    let body = ErrorResponse {
        error: ErrorDetails { code, message },
    };
    (status, Json(body)).into_response()
}
