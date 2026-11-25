//! `POST /plans/{planId}/save` — docs/contract/save-plan.md. The checks below run in
//! the order section 4 of the contract lists them.

use axum::{
    body::{Body, to_bytes},
    extract::{Path, State, rejection::PathRejection},
    http::{HeaderMap, HeaderValue, header},
    response::{IntoResponse, Response},
};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    AppState,
    auth::AuthUser,
    plans::store::{SaveCommand, SaveFailure, SaveOutcome},
    problem::{Problem, ProblemType},
};

pub const IDEMPOTENCY_KEY: &str = "idempotency-key";
pub const REPLAYED: &str = "idempotent-replayed";
const MAX_KEY_LEN: usize = 255;
/// The body is one small object; anything larger is refused before it is parsed.
pub const MAX_BODY_BYTES: usize = 16 * 1024;
/// 2^53 - 1: the largest integer a JavaScript client holds exactly.
pub const MAX_VERSION: i64 = 9_007_199_254_740_991;

pub async fn save_plan(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    path: Result<Path<String>, PathRejection>,
    headers: HeaderMap,
    body: Body,
) -> Response {
    // A segment axum cannot decode (e.g. `%FF`) is a plan that does not exist (step 7);
    // the empty string fails the same length check below.
    let raw_id = path.map(|Path(id)| id).unwrap_or_default();
    match handle(&state, user, &raw_id, &headers, body).await {
        Ok(res) => res,
        Err(problem) => problem.into_response(),
    }
}

/// Any method other than POST on this path (contract section 4.3, step 0).
pub async fn method_not_allowed() -> Response {
    let mut res = Problem::new(
        ProblemType::MethodNotAllowed,
        "This path only accepts POST.",
    )
    .into_response();
    res.headers_mut()
        .insert(header::ALLOW, HeaderValue::from_static("POST"));
    res
}

async fn handle(
    state: &AppState,
    user: Uuid,
    raw_id: &str,
    headers: &HeaderMap,
    body: Body,
) -> Result<Response, Problem> {
    // 2. Idempotency-Key: 1 to 255 printable ASCII characters after trimming spaces.
    // Exactly one header: with two, which one the client meant is a guess.
    let mut keys = headers.get_all(IDEMPOTENCY_KEY).iter();
    let key = keys
        .next()
        .filter(|_| keys.next().is_none())
        .and_then(|v| std::str::from_utf8(v.as_bytes()).ok())
        .map(|k| k.trim_matches(' '))
        .filter(|k| {
            !k.is_empty()
                && k.len() <= MAX_KEY_LEN
                && k.bytes().all(|b| (0x20..=0x7e).contains(&b))
        })
        .ok_or_else(|| {
            Problem::new(
                ProblemType::IdempotencyKeyMissing,
                "Send an Idempotency-Key header of 1 to 255 printable ASCII characters, e.g. a UUID.",
            )
        })?
        .to_string();

    // 3. Content-Type
    if !is_json(headers) {
        return Err(Problem::new(
            ProblemType::UnsupportedMediaType,
            "Send the body as Content-Type: application/json.",
        ));
    }

    // 4. At most 16 KiB
    let body = to_bytes(body, MAX_BODY_BYTES).await.map_err(|_| {
        Problem::new(
            ProblemType::BodyTooLarge,
            "The request body is larger than 16 KiB.",
        )
    })?;

    // 5. Valid JSON
    let value: Value = serde_json::from_slice(&body).map_err(|_| {
        Problem::new(
            ProblemType::InvalidJson,
            "The request body is not valid JSON.",
        )
    })?;

    // 6. Body rules
    let expected_version = read_body(&value)?;

    // 7. Path id
    // Only the hyphenated 36-character form; `Uuid::parse_str` also takes braces, `urn:uuid:`
    // and the simple form, which the contract does not allow.
    let plan_id = Some(raw_id)
        .filter(|id| id.len() == 36)
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(not_found)?;

    // 8-14. The atomic part. Two requests are "the same" when they name the same plan and
    // the same expectedVersion; fields the server ignores do not count.
    let cmd = SaveCommand {
        user,
        plan_id,
        key,
        fingerprint: json!({ "planId": plan_id, "expectedVersion": expected_version }),
        expected_version,
    };
    match state.plans.save(cmd) {
        Ok(SaveOutcome::Saved(body)) => Ok(json_response(body, false)),
        Ok(SaveOutcome::Replayed(body)) => Ok(json_response(body, true)),
        Err(failure) => Err(to_problem(failure)),
    }
}

fn is_json(headers: &HeaderMap) -> bool {
    headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(';').next())
        .is_some_and(|essence| essence.trim().eq_ignore_ascii_case("application/json"))
}

fn read_body(value: &Value) -> Result<i64, Problem> {
    let Some(obj) = value.as_object() else {
        return Err(validation(vec![error("", "must be a JSON object")]));
    };
    match obj.get("expectedVersion") {
        None => Err(validation(vec![error("/expectedVersion", "is required")])),
        Some(v) => match v.as_i64() {
            Some(n) if (1..=MAX_VERSION).contains(&n) && !v.is_f64() => Ok(n),
            _ => Err(validation(vec![error(
                "/expectedVersion",
                "must be an integer from 1 to 9007199254740991",
            )])),
        },
    }
}

fn error(pointer: &str, detail: &str) -> Value {
    json!({ "pointer": pointer, "detail": detail })
}

fn validation(errors: Vec<Value>) -> Problem {
    let detail = if errors.len() == 1 {
        "1 field is invalid.".to_string()
    } else {
        format!("{} fields are invalid.", errors.len())
    };
    Problem::new(ProblemType::Validation, detail).with("errors", Value::Array(errors))
}

fn not_found() -> Problem {
    Problem::new(
        ProblemType::PlanNotFound,
        "No plan with this id in a trip you belong to.",
    )
}

fn to_problem(failure: SaveFailure) -> Problem {
    match failure {
        SaveFailure::NotFound => not_found(),
        SaveFailure::NotOwner => Problem::new(
            ProblemType::NotPlanOwner,
            "Only the member who created this plan can save it.",
        ),
        SaveFailure::VersionConflict { current } => Problem::new(
            ProblemType::VersionConflict,
            "The plan was edited after you reviewed it. Review the current version and save again.",
        )
        .with("currentVersion", json!(current)),
        SaveFailure::Rejected(violations) => {
            let detail = if violations.len() == 1 {
                "The plan breaks 1 rule; it was not saved.".to_string()
            } else {
                format!(
                    "The plan breaks {} rules; it was not saved.",
                    violations.len()
                )
            };
            Problem::new(ProblemType::PlanRejected, detail).with("violations", json!(violations))
        }
        SaveFailure::KeyReused => Problem::new(
            ProblemType::IdempotencyKeyReused,
            "This Idempotency-Key was already used for a different request.",
        ),
        SaveFailure::Internal(msg) => {
            tracing::error!(error = %msg, "saving a plan failed");
            Problem::internal()
        }
    }
}

fn json_response(body: Value, replayed: bool) -> Response {
    let mut res = (
        [(header::CONTENT_TYPE, "application/json")],
        body.to_string(),
    )
        .into_response();
    if replayed {
        res.headers_mut()
            .insert(REPLAYED, HeaderValue::from_static("true"));
    }
    res
}
