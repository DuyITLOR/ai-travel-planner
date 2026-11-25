//! RFC 9457 problem details. Every slug here is listed in a table in
//! `docs/contract/save-plan.md`; `tests/save_plan_contract.rs` fails if the two drift.

use axum::{
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde_json::{Map, Value, json};

/// Prefix of every problem `type`. `.example` is reserved (RFC 2606): the URI names
/// the problem, it is not meant to be fetched.
pub const TYPE_BASE: &str = "https://atp.example/problems/";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProblemType {
    InvalidJson,
    Validation,
    IdempotencyKeyMissing,
    Unauthenticated,
    NotPlanOwner,
    PlanNotFound,
    VersionConflict,
    UnsupportedMediaType,
    MethodNotAllowed,
    BodyTooLarge,
    PlanRejected,
    IdempotencyKeyReused,
    Internal,
}

impl ProblemType {
    pub const ALL: [ProblemType; 13] = [
        Self::InvalidJson,
        Self::Validation,
        Self::IdempotencyKeyMissing,
        Self::Unauthenticated,
        Self::NotPlanOwner,
        Self::PlanNotFound,
        Self::VersionConflict,
        Self::UnsupportedMediaType,
        Self::MethodNotAllowed,
        Self::BodyTooLarge,
        Self::PlanRejected,
        Self::IdempotencyKeyReused,
        Self::Internal,
    ];

    pub fn slug(self) -> &'static str {
        match self {
            Self::InvalidJson => "invalid-json",
            Self::Validation => "validation",
            Self::IdempotencyKeyMissing => "idempotency-key-missing",
            Self::Unauthenticated => "unauthenticated",
            Self::NotPlanOwner => "not-plan-owner",
            Self::PlanNotFound => "plan-not-found",
            Self::VersionConflict => "version-conflict",
            Self::UnsupportedMediaType => "unsupported-media-type",
            Self::MethodNotAllowed => "method-not-allowed",
            Self::BodyTooLarge => "body-too-large",
            Self::PlanRejected => "plan-rejected",
            Self::IdempotencyKeyReused => "idempotency-key-reused",
            Self::Internal => "internal",
        }
    }

    pub fn status(self) -> StatusCode {
        match self {
            Self::InvalidJson | Self::Validation | Self::IdempotencyKeyMissing => {
                StatusCode::BAD_REQUEST
            }
            Self::Unauthenticated => StatusCode::UNAUTHORIZED,
            Self::NotPlanOwner => StatusCode::FORBIDDEN,
            Self::PlanNotFound => StatusCode::NOT_FOUND,
            Self::VersionConflict => StatusCode::CONFLICT,
            Self::UnsupportedMediaType => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            Self::MethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED,
            Self::BodyTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            Self::PlanRejected | Self::IdempotencyKeyReused => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::InvalidJson => "Malformed JSON",
            Self::Validation => "Invalid request",
            Self::IdempotencyKeyMissing => "Idempotency-Key required",
            Self::Unauthenticated => "Authentication required",
            Self::NotPlanOwner => "Not the plan's creator",
            Self::PlanNotFound => "Plan not found",
            Self::VersionConflict => "Plan changed since review",
            Self::UnsupportedMediaType => "Unsupported media type",
            Self::MethodNotAllowed => "Method not allowed",
            Self::BodyTooLarge => "Request body too large",
            Self::PlanRejected => "Plan rejected by the validator",
            Self::IdempotencyKeyReused => "Idempotency-Key reused",
            Self::Internal => "Internal error",
        }
    }

    pub fn uri(self) -> String {
        format!("{TYPE_BASE}{}", self.slug())
    }
}

/// One problem occurrence. Extra members (`errors`, `violations`, ...) go in `extensions`.
#[derive(Debug, Clone)]
pub struct Problem {
    pub kind: ProblemType,
    pub detail: String,
    pub extensions: Map<String, Value>,
}

impl Problem {
    pub fn new(kind: ProblemType, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
            extensions: Map::new(),
        }
    }

    pub fn with(mut self, key: &str, value: Value) -> Self {
        self.extensions.insert(key.to_string(), value);
        self
    }

    /// The only 500 body clients ever see: no internal error text leaks out.
    pub fn internal() -> Self {
        Self::new(
            ProblemType::Internal,
            "The server could not complete the request. Try again later.",
        )
    }

    pub fn body(&self) -> Value {
        let mut body = json!({
            "type": self.kind.uri(),
            "title": self.kind.title(),
            "status": self.kind.status().as_u16(),
            "detail": self.detail,
        });
        let obj = body.as_object_mut().expect("problem body is an object");
        for (k, v) in &self.extensions {
            obj.insert(k.clone(), v.clone());
        }
        body
    }
}

impl IntoResponse for Problem {
    fn into_response(self) -> Response {
        let status = self.kind.status();
        let mut res = (status, self.body().to_string()).into_response();
        let headers = res.headers_mut();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/problem+json"),
        );
        if self.kind == ProblemType::Unauthenticated {
            headers.insert(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
        }
        res
    }
}
