//! Contract tests for `POST /plans/{planId}/save` and `GET /plans/{planId}`, following
//! docs/contract/save-plan.md.
//! Section numbers in the test names point at that file. These tests never call a model.

use std::sync::{
    Arc, OnceLock,
    atomic::{AtomicI64, Ordering},
};

use atp_api::{
    AppState,
    auth::{AUDIENCE, JwtVerifier},
    build_app,
    config::Config,
    db::Db,
    plans::{
        model::{Item, Member, Plan, PlanStatus, Stop, Trip},
        store::{InMemoryPlanStore, PlanStore, SaveCommand, SaveFailure, SaveOutcome},
    },
    problem::{ProblemType, TYPE_BASE},
};
use axum::{
    Router,
    body::Body,
    http::{HeaderMap, HeaderValue, Request, StatusCode},
};
use chrono::{DateTime, Duration, Utc};
use http_body_util::BodyExt;
use jsonwebtoken::{EncodingKey, Header, encode};
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;

const SECRET: &str = "contract-test-secret-not-a-real-one";

// ---------- fixture ----------

const CREATOR: Uuid = Uuid::from_u128(0xc0);
const MEMBER: Uuid = Uuid::from_u128(0x4e); // in the trips, cap 400 000
const STRANGER: Uuid = Uuid::from_u128(0x57);

const TRIP_A: Uuid = Uuid::from_u128(0xa); // budget 4 000 000
const TRIP_B: Uuid = Uuid::from_u128(0xb); // budget 1 000 000, same members
const TRIP_S: Uuid = Uuid::from_u128(0x5); // the stranger's own trip

const OK_PLAN: Uuid = Uuid::from_u128(0x100); // TRIP_A, v3, 300 000 per person
const OVER_CAP: Uuid = Uuid::from_u128(0x200); // TRIP_A, 450 000 per person
const OVER_BOTH: Uuid = Uuid::from_u128(0x300); // TRIP_B, 450 000 per person
const OTHERS_PLAN: Uuid = Uuid::from_u128(0x400); // TRIP_S, the stranger's
const AT_CAP: Uuid = Uuid::from_u128(0x500); // TRIP_A, exactly 400 000 per person
const AT_BUDGET: Uuid = Uuid::from_u128(0x600); // TRIP_B, exactly 1 000 000 in total
const TRIP_TIE: Uuid = Uuid::from_u128(0x7); // three members share the lowest cap
const TIE_PLAN: Uuid = Uuid::from_u128(0x700); // TRIP_TIE, 150 000 per person
const HUGE_PLAN: Uuid = Uuid::from_u128(0x800); // TRIP_A, a cost past 2^53 - 1

fn members() -> Vec<Member> {
    vec![
        Member {
            user_id: CREATOR,
            cap_vnd: None,
        },
        Member {
            user_id: Uuid::from_u128(0x3),
            cap_vnd: Some(900_000),
        },
        Member {
            user_id: MEMBER,
            cap_vnd: Some(400_000),
        },
        Member {
            user_id: Uuid::from_u128(0x4),
            cap_vnd: None,
        },
    ]
}

fn item(id: &str, price: i64, qty: i64) -> Item {
    Item {
        item_id: id.into(),
        name: id.into(),
        unit_price_vnd: price,
        quantity: qty,
    }
}

fn plan(id: Uuid, trip: Uuid, by: Uuid, version: i64, items: Vec<Item>) -> Plan {
    Plan {
        id,
        trip_id: trip,
        status: PlanStatus::Draft,
        version,
        created_by: by,
        saved_at: None,
        saved_by: None,
        stops: vec![Stop {
            place_id: "hcm-ben-thanh-market".into(),
            day: "2026-12-06".into(),
            start: "09:00".into(),
            end: "10:30".into(),
            items,
        }],
    }
}

fn store() -> Arc<InMemoryPlanStore> {
    seed(InMemoryPlanStore::default())
}

fn seed(s: InMemoryPlanStore) -> Arc<InMemoryPlanStore> {
    s.insert_trip(Trip {
        id: TRIP_A,
        budget_vnd: 4_000_000,
        members: members(),
    });
    s.insert_trip(Trip {
        id: TRIP_B,
        budget_vnd: 1_000_000,
        members: members(),
    });
    s.insert_trip(Trip {
        id: TRIP_S,
        budget_vnd: 1_000_000,
        members: vec![Member {
            user_id: STRANGER,
            cap_vnd: None,
        }],
    });
    // 60 000 + 2 × 120 000 = 300 000 per person
    let ok = vec![item("banh-xeo", 60_000, 1), item("ticket", 120_000, 2)];
    // 450 000 per person
    let over = vec![item("set-menu", 150_000, 3)];
    s.insert_plan(plan(OK_PLAN, TRIP_A, CREATOR, 3, ok.clone()));
    s.insert_plan(plan(OVER_CAP, TRIP_A, CREATOR, 1, over.clone()));
    s.insert_plan(plan(OVER_BOTH, TRIP_B, CREATOR, 1, over));
    s.insert_plan(plan(OTHERS_PLAN, TRIP_S, STRANGER, 1, ok));
    s.insert_plan(plan(
        AT_CAP,
        TRIP_A,
        CREATOR,
        1,
        vec![item("ticket", 200_000, 2)],
    ));
    s.insert_plan(plan(
        AT_BUDGET,
        TRIP_B,
        CREATOR,
        1,
        vec![item("ticket", 250_000, 1)],
    ));
    s.insert_plan(plan(
        HUGE_PLAN,
        TRIP_A,
        CREATOR,
        1,
        vec![item("bad-price", i64::MAX / 2, 3)],
    ));
    let capped = |id: u128| Member {
        user_id: Uuid::from_u128(id),
        cap_vnd: Some(100_000),
    };
    s.insert_trip(Trip {
        id: TRIP_TIE,
        budget_vnd: 10_000_000,
        members: vec![
            Member {
                user_id: CREATOR,
                cap_vnd: Some(100_000),
            },
            capped(0x9),
            capped(0x8),
        ],
    });
    s.insert_plan(plan(
        TIE_PLAN,
        TRIP_TIE,
        CREATOR,
        1,
        vec![item("ticket", 150_000, 1)],
    ));
    Arc::new(s)
}

fn app_with(plans: Arc<dyn PlanStore>) -> Router {
    let config = Config {
        port: 0,
        web_origins: vec!["http://localhost:3000".into()],
        database_url: None,
    };
    let state = AppState::new(Db::disabled())
        .with_plans(plans)
        .with_auth(JwtVerifier::new(Some(SECRET)));
    build_app(&config, state)
}

fn token_with(user: Uuid, secret: &str, exp_offset: i64, aud: &str) -> String {
    let exp = chrono::Utc::now().timestamp() + exp_offset;
    let claims = json!({ "sub": user, "aud": aud, "exp": exp, "role": "authenticated" });
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()).unwrap(),
    )
    .unwrap()
}

fn token(user: Uuid) -> String {
    token_with(user, SECRET, 3600, AUDIENCE)
}

// ---------- request helper ----------

struct Req {
    plan: String,
    auth: Option<String>,
    key: Option<String>,
    content_type: Option<&'static str>,
    body: String,
}

impl Req {
    /// A valid request from CREATOR for OK_PLAN at version 3.
    fn new(plan: Uuid) -> Self {
        Self {
            plan: plan.to_string(),
            auth: Some(token(CREATOR)),
            key: Some(Uuid::new_v4().to_string()),
            content_type: Some("application/json"),
            body: r#"{"expectedVersion":3}"#.into(),
        }
    }
    fn by(mut self, user: Uuid) -> Self {
        self.auth = Some(token(user));
        self
    }
    fn version(mut self, v: i64) -> Self {
        self.body = json!({ "expectedVersion": v }).to_string();
        self
    }
    fn key(mut self, k: &str) -> Self {
        self.key = Some(k.into());
        self
    }
    fn body(mut self, b: &str) -> Self {
        self.body = b.into();
        self
    }
}

struct Res {
    status: StatusCode,
    headers: HeaderMap,
    json: Value,
}

async fn send(app: &Router, r: Req) -> Res {
    let mut b = Request::post(format!("/plans/{}/save", r.plan));
    if let Some(a) = r.auth {
        b = b.header("authorization", format!("Bearer {a}"));
    }
    if let Some(k) = r.key {
        b = b.header("idempotency-key", k);
    }
    if let Some(ct) = r.content_type {
        b = b.header("content-type", ct);
    }
    send_raw(app, b.body(Body::from(r.body)).unwrap()).await
}

async fn send_raw(app: &Router, req: Request<Body>) -> Res {
    let res = app.clone().oneshot(req).await.unwrap();
    let (parts, body) = res.into_parts();
    let bytes = body.collect().await.unwrap().to_bytes();
    Res {
        status: parts.status,
        headers: parts.headers,
        json: serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    }
}

/// One row of the problem table in section 2 of the contract.
#[derive(Debug, Clone)]
struct Row {
    slug: String,
    status: u16,
    title: String,
    /// The extra member this type carries (`errors`, `currentVersion`, `violations`), if any.
    extra: Option<String>,
}

/// The problem table of section 2, read from the contract file itself.
fn contract_table() -> &'static Vec<Row> {
    static TABLE: OnceLock<Vec<Row>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../docs/contract/save-plan.md"
        );
        let doc = std::fs::read_to_string(path).expect("docs/contract/save-plan.md exists");
        doc.lines()
            .filter_map(|line| {
                let cells: Vec<&str> = line.split('|').map(str::trim).collect();
                // | `slug` | 400 | Title | When | Extra |
                let slug = cells.get(1)?.strip_prefix('`')?.strip_suffix('`')?;
                let status: u16 = cells.get(2)?.parse().ok()?;
                let extra = cells
                    .get(5)
                    .and_then(|c| c.strip_prefix('`'))
                    .and_then(|c| c.split('`').next())
                    .map(str::to_string);
                Some(Row {
                    slug: slug.to_string(),
                    status,
                    title: cells.get(3)?.to_string(),
                    extra,
                })
            })
            .collect()
    })
}

/// Section 2: every problem has the same shape, and `status`/`title` are the ones the
/// **contract** names for that slug (not whatever the code says).
fn assert_problem(res: &Res, kind: ProblemType) {
    let slug = kind.slug();
    let row = contract_table()
        .iter()
        .find(|r| r.slug == slug)
        .unwrap_or_else(|| panic!("`{slug}` is not in the contract's problem table"));
    let (status, title) = (&row.status, &row.title);
    assert_eq!(
        res.status.as_u16(),
        *status,
        "status for {slug}: {}",
        res.json
    );
    assert_eq!(
        res.headers["content-type"], "application/problem+json",
        "content-type for {slug}"
    );
    assert_eq!(res.json["type"], format!("{TYPE_BASE}{slug}"));
    assert_eq!(res.json["title"], title.as_str());
    assert_eq!(res.json["status"], *status);
    let detail = res.json["detail"].as_str().expect("detail is a string");
    assert!(!detail.is_empty());
    // Only the extra member the table names for this type, and that one always.
    let mut allowed = vec!["type", "title", "status", "detail"];
    if let Some(extra) = &row.extra {
        allowed.push(extra.as_str());
        assert!(
            res.json.get(extra).is_some(),
            "{slug} must carry `{extra}`: {}",
            res.json
        );
    }
    for k in res.json.as_object().unwrap().keys() {
        assert!(allowed.contains(&k.as_str()), "{slug} must not carry `{k}`");
    }
}

// ---------- section 2: the problem table is the code ----------

#[test]
fn s2_problem_table_matches_the_code() {
    let mut in_doc: Vec<(String, u16, String)> = contract_table()
        .iter()
        .map(|r| (r.slug.clone(), r.status, r.title.clone()))
        .collect();
    let mut in_code: Vec<(String, u16, String)> = ProblemType::ALL
        .iter()
        .map(|k| (k.slug().into(), k.status().as_u16(), k.title().into()))
        .collect();
    in_doc.sort();
    in_code.sort();
    assert_eq!(
        in_doc, in_code,
        "section 2 of the contract and problem.rs disagree"
    );
}

// ---------- 4.3 step 13: success ----------

#[tokio::test]
async fn s4_saves_a_draft_and_returns_the_plan() {
    let store = store();
    let app = app_with(store.clone());
    let res = send(&app, Req::new(OK_PLAN)).await;

    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    assert_eq!(res.headers["content-type"], "application/json");
    assert!(res.headers.get("idempotent-replayed").is_none());
    let p = &res.json;
    assert_eq!(p["id"], OK_PLAN.to_string());
    assert_eq!(p["tripId"], TRIP_A.to_string());
    assert_eq!(p["status"], "saved");
    assert_eq!(p["version"], 3, "saving does not change the version");
    assert_eq!(p["createdBy"], CREATOR.to_string());
    assert_eq!(p["savedBy"], CREATOR.to_string());
    let saved_at = p["savedAt"].as_str().unwrap();
    assert!(DateTime::parse_from_rfc3339(saved_at).is_ok());
    assert!(saved_at.ends_with('Z'));

    let stop = &p["stops"][0];
    assert_eq!(stop["placeId"], "hcm-ben-thanh-market");
    assert_eq!(stop["day"], "2026-12-06");
    assert_eq!(stop["start"], "09:00");
    assert_eq!(stop["items"][1]["unitPriceVnd"], 120_000);
    assert_eq!(stop["items"][1]["quantity"], 2);
    assert_eq!(stop["costPerPersonVnd"], 300_000);
    assert_eq!(
        p["cost"],
        json!({ "perPersonVnd": 300_000, "memberCount": 4, "totalVnd": 1_200_000, "budgetVnd": 4_000_000 })
    );
    // money fields are integers, never floats
    assert!(p["cost"]["totalVnd"].is_i64());

    let stored = store.plan(OK_PLAN).unwrap();
    assert_eq!(stored.status, PlanStatus::Saved);
}

#[tokio::test]
async fn s4_content_type_may_carry_a_charset() {
    let app = app_with(store());
    let mut r = Req::new(OK_PLAN);
    r.content_type = Some("application/json; charset=utf-8");
    assert_eq!(send(&app, r).await.status, StatusCode::OK);
}

#[tokio::test]
async fn s1_unknown_request_fields_are_ignored() {
    let app = app_with(store());
    let r = Req::new(OK_PLAN).body(r#"{"expectedVersion":3,"note":"hi"}"#);
    assert_eq!(send(&app, r).await.status, StatusCode::OK);
}

// ---------- 401 unauthenticated ----------

#[tokio::test]
async fn s2_401_without_a_token() {
    let app = app_with(store());
    let mut r = Req::new(OK_PLAN);
    r.auth = None;
    let res = send(&app, r).await;
    assert_problem(&res, ProblemType::Unauthenticated);
    assert_eq!(res.headers["www-authenticate"], "Bearer");
}

#[tokio::test]
async fn s2_401_for_bad_signature_expired_or_wrong_audience() {
    let app = app_with(store());
    for bad in [
        token_with(CREATOR, "another-project-secret", 3600, AUDIENCE),
        token_with(CREATOR, SECRET, -60, AUDIENCE),
        token_with(CREATOR, SECRET, 3600, "anon"),
        "not-a-jwt".to_string(),
    ] {
        let mut r = Req::new(OK_PLAN);
        r.auth = Some(bad);
        assert_problem(&send(&app, r).await, ProblemType::Unauthenticated);
    }
}

#[tokio::test]
async fn s4_3_auth_is_checked_before_anything_else() {
    let app = app_with(store());
    let r = Req {
        plan: "not-a-uuid".into(),
        auth: None,
        key: None,
        content_type: None,
        body: "{".into(),
    };
    assert_problem(&send(&app, r).await, ProblemType::Unauthenticated);
}

// ---------- 400 idempotency-key-missing ----------

#[tokio::test]
async fn s2_400_idempotency_key_missing_empty_or_too_long() {
    let app = app_with(store());
    for key in [None, Some("   ".to_string()), Some("k".repeat(256))] {
        let mut r = Req::new(OK_PLAN);
        r.key = key;
        assert_problem(&send(&app, r).await, ProblemType::IdempotencyKeyMissing);
    }
    // 255 characters is fine
    let r = Req::new(OK_PLAN).key(&"k".repeat(255));
    assert_eq!(send(&app, r).await.status, StatusCode::OK);
}

// ---------- 415 unsupported-media-type ----------

#[tokio::test]
async fn s2_415_when_the_body_is_not_declared_json() {
    let app = app_with(store());
    for ct in [
        None,
        Some("text/plain"),
        Some("application/json-patch+json"),
        Some("application/jsonx"),
    ] {
        let mut r = Req::new(OK_PLAN);
        r.content_type = ct;
        assert_problem(&send(&app, r).await, ProblemType::UnsupportedMediaType);
    }
}

// ---------- 400 invalid-json ----------

#[tokio::test]
async fn s2_400_invalid_json() {
    let app = app_with(store());
    for body in ["{", "", "{\"expectedVersion\": }"] {
        let r = Req::new(OK_PLAN).body(body);
        assert_problem(&send(&app, r).await, ProblemType::InvalidJson);
    }
}

// ---------- 400 validation ----------

#[tokio::test]
async fn s2_400_validation_points_at_the_field() {
    let app = app_with(store());
    let cases = [
        ("{}", "/expectedVersion"),
        (r#"{"expectedVersion":0}"#, "/expectedVersion"),
        (r#"{"expectedVersion":3.0}"#, "/expectedVersion"),
        (r#"{"expectedVersion":"3"}"#, "/expectedVersion"),
        (r#"{"expectedVersion":null}"#, "/expectedVersion"),
        ("[3]", ""),
    ];
    for (body, pointer) in cases {
        let res = send(&app, Req::new(OK_PLAN).body(body)).await;
        assert_problem(&res, ProblemType::Validation);
        let errors = res.json["errors"].as_array().expect("errors array");
        assert_eq!(errors.len(), 1, "{body}");
        assert_eq!(errors[0]["pointer"], pointer, "{body}");
        assert!(errors[0]["detail"].is_string());
    }
}

// ---------- 404 plan-not-found and the ownership rule ----------

#[tokio::test]
async fn s2_404_for_an_unknown_or_malformed_plan_id() {
    let app = app_with(store());
    let res = send(&app, Req::new(Uuid::new_v4())).await;
    assert_problem(&res, ProblemType::PlanNotFound);

    let mut r = Req::new(OK_PLAN);
    r.plan = "123".into();
    assert_problem(&send(&app, r).await, ProblemType::PlanNotFound);
}

#[tokio::test]
async fn s4_1_another_users_plan_is_404_not_403() {
    let store = store();
    let app = app_with(store.clone());
    // CREATOR is not in TRIP_S: the stranger's plan must look like it does not exist.
    let res = send(&app, Req::new(OTHERS_PLAN).version(1)).await;
    assert_problem(&res, ProblemType::PlanNotFound);
    assert_eq!(
        res.json["detail"],
        send(&app, Req::new(Uuid::new_v4()).version(1)).await.json["detail"],
        "same answer as for a plan that does not exist"
    );
    // ...and the stranger cannot save ours.
    let res = send(&app, Req::new(OK_PLAN).by(STRANGER)).await;
    assert_problem(&res, ProblemType::PlanNotFound);
    assert_eq!(store.plan(OK_PLAN).unwrap().status, PlanStatus::Draft);
}

// ---------- 403 not-plan-owner ----------

#[tokio::test]
async fn s4_1_a_member_who_did_not_create_the_plan_gets_403() {
    let store = store();
    let app = app_with(store.clone());
    let res = send(&app, Req::new(OK_PLAN).by(MEMBER)).await;
    assert_problem(&res, ProblemType::NotPlanOwner);
    assert_eq!(store.plan(OK_PLAN).unwrap().status, PlanStatus::Draft);
}

// ---------- 409 version-conflict ----------

#[tokio::test]
async fn s2_409_when_the_plan_changed_since_review() {
    let store = store();
    let app = app_with(store.clone());
    let res = send(&app, Req::new(OK_PLAN).version(2)).await;
    assert_problem(&res, ProblemType::VersionConflict);
    assert_eq!(res.json["currentVersion"], 3);
    assert_eq!(store.plan(OK_PLAN).unwrap().status, PlanStatus::Draft);
}

// ---------- 422 plan-rejected (4.4) ----------

#[tokio::test]
async fn s4_4_over_the_lowest_member_cap_is_rejected_and_not_shared() {
    let store = store();
    let app = app_with(store.clone());
    let res = send(&app, Req::new(OVER_CAP).version(1)).await;
    assert_problem(&res, ProblemType::PlanRejected);
    assert_eq!(
        res.json["violations"],
        json!([{
            "rule": "over-member-cap",
            "message": res.json["violations"][0]["message"],
            "userId": MEMBER.to_string(),
            "amountVnd": 450_000,
            "limitVnd": 400_000
        }])
    );
    let stored = store.plan(OVER_CAP).unwrap();
    assert_eq!(stored.status, PlanStatus::Draft);
    assert!(stored.saved_at.is_none());
}

#[tokio::test]
async fn s4_4_both_rules_are_reported_in_order() {
    let app = app_with(store());
    let res = send(&app, Req::new(OVER_BOTH).version(1)).await;
    assert_problem(&res, ProblemType::PlanRejected);
    let v = res.json["violations"].as_array().unwrap();
    assert_eq!(v.len(), 2);
    assert_eq!(v[0]["rule"], "over-member-cap");
    assert_eq!(v[1]["rule"], "over-trip-budget");
    assert_eq!(v[1]["amountVnd"], 1_800_000);
    assert_eq!(v[1]["limitVnd"], 1_000_000);
    assert!(v[1].get("userId").is_none());
}

// ---------- 4.5 idempotency ----------

#[tokio::test]
async fn s4_5_a_retry_with_the_same_key_replays_and_saves_once() {
    let store = store();
    let app = app_with(store.clone());
    let first = send(&app, Req::new(OK_PLAN).key("retry-1")).await;
    assert_eq!(first.status, StatusCode::OK);
    assert!(first.headers.get("idempotent-replayed").is_none());

    // key order and whitespace do not make a request different
    let again = send(
        &app,
        Req::new(OK_PLAN)
            .key("retry-1")
            .body("{ \"expectedVersion\" : 3 }"),
    )
    .await;
    assert_eq!(again.status, StatusCode::OK);
    assert_eq!(again.headers["idempotent-replayed"], "true");
    assert_eq!(again.json, first.json, "identical body, same savedAt");
}

#[tokio::test]
async fn s2_422_same_key_for_a_different_request() {
    let store = store();
    let app = app_with(store.clone());
    assert_eq!(
        send(&app, Req::new(OK_PLAN).key("k-1")).await.status,
        StatusCode::OK
    );
    // different plan, same key
    let res = send(&app, Req::new(OVER_CAP).version(1).key("k-1")).await;
    assert_problem(&res, ProblemType::IdempotencyKeyReused);
    assert_eq!(store.plan(OVER_CAP).unwrap().status, PlanStatus::Draft);
    // same plan, different body, same key
    let res = send(&app, Req::new(OK_PLAN).version(4).key("k-1")).await;
    assert_problem(&res, ProblemType::IdempotencyKeyReused);
}

#[tokio::test]
async fn s4_5_keys_are_per_caller() {
    let app = app_with(store());
    // the stranger used "shared-key" first, on their own plan
    let res = send(
        &app,
        Req::new(OTHERS_PLAN)
            .by(STRANGER)
            .version(1)
            .key("shared-key"),
    )
    .await;
    assert_eq!(res.status, StatusCode::OK);
    // the same key from CREATOR is a fresh key, not a reuse
    let res = send(&app, Req::new(OK_PLAN).key("shared-key")).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    assert!(res.headers.get("idempotent-replayed").is_none());
}

#[tokio::test]
async fn s4_5_an_error_is_not_stored_under_the_key() {
    let app = app_with(store());
    let res = send(&app, Req::new(OK_PLAN).version(2).key("fix-me")).await;
    assert_problem(&res, ProblemType::VersionConflict);
    // the client fixes the version and sends the same key again
    let res = send(&app, Req::new(OK_PLAN).key("fix-me")).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    assert!(res.headers.get("idempotent-replayed").is_none());
}

#[tokio::test]
async fn s4_3_saving_again_with_a_new_key_changes_nothing() {
    let store = store();
    let app = app_with(store.clone());
    let first = send(&app, Req::new(OK_PLAN)).await;
    let second = send(&app, Req::new(OK_PLAN)).await; // new random key
    assert_eq!(second.status, StatusCode::OK);
    assert!(second.headers.get("idempotent-replayed").is_none());
    assert_eq!(second.json["savedAt"], first.json["savedAt"]);
    assert_eq!(second.json, first.json);
}

// ---------- 500 internal ----------

struct BrokenStore;

impl PlanStore for BrokenStore {
    fn save(&self, _: SaveCommand) -> Result<SaveOutcome, SaveFailure> {
        Err(SaveFailure::Internal(
            "error returned from database: relation \"plans\" does not exist".into(),
        ))
    }
    fn read(&self, _: Uuid, _: Uuid) -> Result<Value, SaveFailure> {
        Err(SaveFailure::Internal(
            "error returned from database: relation \"plans\" does not exist".into(),
        ))
    }
}

#[tokio::test]
async fn s2_500_internal_never_leaks_the_cause() {
    let app = app_with(Arc::new(BrokenStore));
    let res = send(&app, Req::new(OK_PLAN)).await;
    assert_problem(&res, ProblemType::Internal);
    let text = res.json.to_string();
    assert!(!text.contains("relation"), "{text}");
    assert!(!text.contains("plans\\\""), "{text}");
}

// ---------- 405 method-not-allowed, 413 body-too-large ----------

#[tokio::test]
async fn s2_405_for_any_other_method_without_a_token() {
    let app = app_with(store());
    for method in ["GET", "PUT", "DELETE"] {
        let req = Request::builder()
            .method(method)
            .uri(format!("/plans/{OK_PLAN}/save"))
            .body(Body::empty())
            .unwrap();
        let res = send_raw(&app, req).await;
        assert_problem(&res, ProblemType::MethodNotAllowed);
        assert_eq!(res.headers["allow"], "POST");
    }
}

#[tokio::test]
async fn s2_413_for_a_body_over_16_kib() {
    let app = app_with(store());
    // 16 384 bytes exactly is accepted (the extra field is ignored)
    let pad = 16_384 - r#"{"expectedVersion":3,"pad":""}"#.len();
    let body = format!(r#"{{"expectedVersion":3,"pad":"{}"}}"#, "x".repeat(pad));
    assert_eq!(body.len(), 16_384);
    assert_eq!(
        send(&app, Req::new(OK_PLAN).body(&body)).await.status,
        StatusCode::OK
    );
    // one byte more is refused before it is parsed, even when it is not JSON
    let res = send(&app, Req::new(OVER_CAP).body(&"x".repeat(16_385))).await;
    assert_problem(&res, ProblemType::BodyTooLarge);
}

// ---------- 4.2 / 4.3 header details ----------

#[tokio::test]
async fn s4_2_a_non_ascii_idempotency_key_is_refused() {
    let app = app_with(store());
    let req = Request::post(format!("/plans/{OK_PLAN}/save"))
        .header("authorization", format!("Bearer {}", token(CREATOR)))
        .header("content-type", "application/json")
        .header(
            "idempotency-key",
            HeaderValue::from_bytes("khóa-1".as_bytes()).unwrap(),
        )
        .body(Body::from(r#"{"expectedVersion":3}"#))
        .unwrap();
    assert_problem(
        &send_raw(&app, req).await,
        ProblemType::IdempotencyKeyMissing,
    );
}

#[tokio::test]
async fn s4_3_the_bearer_scheme_is_case_insensitive() {
    let app = app_with(store());
    let req = Request::post(format!("/plans/{OK_PLAN}/save"))
        .header("authorization", format!("bearer {}", token(CREATOR)))
        .header("content-type", "application/json")
        .header("idempotency-key", "lower-case-scheme")
        .body(Body::from(r#"{"expectedVersion":3}"#))
        .unwrap();
    assert_eq!(send_raw(&app, req).await.status, StatusCode::OK);
}

// ---------- 4.3 order of checks: two faults at once, the earlier step wins ----------

#[tokio::test]
async fn s4_3_order_key_before_media_type_before_size_before_json() {
    let app = app_with(store());
    // step 2 before 3
    let mut r = Req::new(OK_PLAN);
    r.key = None;
    r.content_type = Some("text/plain");
    assert_problem(&send(&app, r).await, ProblemType::IdempotencyKeyMissing);
    // step 3 before 4 and 5
    let mut r = Req::new(OK_PLAN).body(&"{".repeat(20_000));
    r.content_type = Some("text/plain");
    assert_problem(&send(&app, r).await, ProblemType::UnsupportedMediaType);
    // step 4 before 5
    let r = Req::new(OK_PLAN).body(&"{".repeat(20_000));
    assert_problem(&send(&app, r).await, ProblemType::BodyTooLarge);
}

#[tokio::test]
async fn s4_3_order_body_rules_before_plan_id() {
    let app = app_with(store());
    let mut r = Req::new(OK_PLAN).body("{}");
    r.plan = "123".into();
    assert_problem(&send(&app, r).await, ProblemType::Validation);
}

#[tokio::test]
async fn s4_3_order_ownership_before_key_before_version_before_validator() {
    let app = app_with(store());
    // a key used on our plan, then sent for the stranger's plan: 404 (step 8), not 422 (step 10)
    assert_eq!(
        send(&app, Req::new(OK_PLAN).key("k-order")).await.status,
        StatusCode::OK
    );
    let res = send(&app, Req::new(OTHERS_PLAN).version(1).key("k-order")).await;
    assert_problem(&res, ProblemType::PlanNotFound);
    // a member who is not the creator, with a stale version: 403 (step 9), not 409 (step 11)
    let res = send(&app, Req::new(OVER_CAP).by(MEMBER).version(7)).await;
    assert_problem(&res, ProblemType::NotPlanOwner);
    // the key reused with a stale version: 422 reused (step 10), not 409 (step 11)
    let res = send(&app, Req::new(OK_PLAN).version(2).key("k-order")).await;
    assert_problem(&res, ProblemType::IdempotencyKeyReused);
    // an over-cap plan with a stale version: 409 (step 11), not 422 plan-rejected (step 13)
    let res = send(&app, Req::new(OVER_CAP).version(2)).await;
    assert_problem(&res, ProblemType::VersionConflict);
}

// ---------- 4.1 + 4.5: a member who left cannot replay ----------

#[tokio::test]
async fn s4_1_a_member_who_left_the_trip_cannot_replay_a_stored_save() {
    let store = store();
    let app = app_with(store.clone());
    assert_eq!(
        send(&app, Req::new(OK_PLAN).key("before-leaving"))
            .await
            .status,
        StatusCode::OK
    );
    store.remove_member(TRIP_A, CREATOR);
    let res = send(&app, Req::new(OK_PLAN).key("before-leaving")).await;
    assert_problem(&res, ProblemType::PlanNotFound);
    assert!(res.headers.get("idempotent-replayed").is_none());
}

// ---------- 4.5: what makes two requests the same, and the 24 hours ----------

#[tokio::test]
async fn s4_5_ignored_fields_do_not_make_a_retry_different() {
    let app = app_with(store());
    let first = send(&app, Req::new(OK_PLAN).key("k-note")).await;
    assert_eq!(first.status, StatusCode::OK);
    let again = send(
        &app,
        Req::new(OK_PLAN)
            .key("k-note")
            .body(r#"{"note":"retry from a flaky network","expectedVersion":3}"#),
    )
    .await;
    assert_eq!(again.status, StatusCode::OK, "{}", again.json);
    assert_eq!(again.headers["idempotent-replayed"], "true");
}

#[tokio::test]
async fn s4_5_a_key_is_kept_for_24_hours_then_forgotten() {
    let start = Utc::now();
    let offset = Arc::new(AtomicI64::new(0));
    let clock = offset.clone();
    let store = seed(InMemoryPlanStore::with_clock(Box::new(move || {
        start + Duration::seconds(clock.load(Ordering::SeqCst))
    })));
    let app = app_with(store);
    let first = send(&app, Req::new(OK_PLAN).key("k-ttl")).await;
    assert_eq!(first.status, StatusCode::OK);

    // 23:59:59 later: the same request replays, a different one is refused
    offset.store(24 * 3600 - 1, Ordering::SeqCst);
    let res = send(&app, Req::new(OK_PLAN).key("k-ttl")).await;
    assert_eq!(res.headers["idempotent-replayed"], "true");
    let res = send(&app, Req::new(OK_PLAN).version(2).key("k-ttl")).await;
    assert_problem(&res, ProblemType::IdempotencyKeyReused);

    // exactly 24:00:00 later: the key is forgotten, the request is handled as new
    offset.store(24 * 3600, Ordering::SeqCst);
    let res = send(&app, Req::new(OK_PLAN).version(2).key("k-ttl")).await;
    assert_problem(&res, ProblemType::VersionConflict);
    let res = send(&app, Req::new(OK_PLAN).key("k-ttl")).await;
    assert_eq!(res.status, StatusCode::OK);
    assert!(res.headers.get("idempotent-replayed").is_none());
    assert_eq!(res.json["savedAt"], first.json["savedAt"], "saved once");
}

// ---------- section 1: browsers ----------

#[tokio::test]
async fn s1_cors_lets_a_browser_send_the_token_and_read_the_replay_header() {
    let app = app_with(store());
    let preflight = Request::builder()
        .method("OPTIONS")
        .uri(format!("/plans/{OK_PLAN}/save"))
        .header("origin", "http://localhost:3000")
        .header("access-control-request-method", "POST")
        .header(
            "access-control-request-headers",
            "authorization,content-type,idempotency-key",
        )
        .body(Body::empty())
        .unwrap();
    let res = send_raw(&app, preflight).await;
    assert_eq!(res.status, StatusCode::OK, "a preflight is never a 405");
    assert_eq!(res.json, Value::Null, "empty body");
    let allowed = res.headers["access-control-allow-headers"]
        .to_str()
        .unwrap()
        .to_ascii_lowercase();
    for h in ["authorization", "content-type", "idempotency-key"] {
        assert!(allowed.contains(h), "{h} not allowed: {allowed}");
    }

    let req = Request::post(format!("/plans/{OK_PLAN}/save"))
        .header("origin", "http://localhost:3000")
        .header("authorization", format!("Bearer {}", token(CREATOR)))
        .header("content-type", "application/json")
        .header("idempotency-key", "cors-1")
        .body(Body::from(r#"{"expectedVersion":3}"#))
        .unwrap();
    let res = send_raw(&app, req).await;
    assert_eq!(res.status, StatusCode::OK);
    let exposed = res.headers["access-control-expose-headers"]
        .to_str()
        .unwrap()
        .to_ascii_lowercase();
    assert!(exposed.contains("idempotent-replayed"), "{exposed}");
    assert!(exposed.contains("www-authenticate"), "{exposed}");
}

// ---------- 4.4: limits are inclusive, and ties ----------

#[tokio::test]
async fn s4_4_a_cost_equal_to_the_lowest_cap_is_allowed() {
    let app = app_with(store());
    let res = send(&app, Req::new(AT_CAP).version(1)).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    assert_eq!(res.json["cost"]["perPersonVnd"], 400_000);
}

#[tokio::test]
async fn s4_4_a_total_equal_to_the_budget_is_allowed() {
    let app = app_with(store());
    let res = send(&app, Req::new(AT_BUDGET).version(1)).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    assert_eq!(res.json["cost"]["totalVnd"], 1_000_000);
    assert_eq!(res.json["cost"]["budgetVnd"], 1_000_000);
}

#[tokio::test]
async fn s4_4_a_tie_on_the_lowest_cap_names_the_smallest_user_id() {
    let app = app_with(store());
    let res = send(&app, Req::new(TIE_PLAN).version(1)).await;
    assert_problem(&res, ProblemType::PlanRejected);
    assert_eq!(
        res.json["violations"][0]["userId"],
        Uuid::from_u128(0x8).to_string()
    );
    assert_eq!(res.json["violations"][0]["limitVnd"], 100_000);
}

// ---------- 4.2: bounds and forms ----------

#[tokio::test]
async fn s4_2_expected_version_is_at_most_2_pow_53_minus_1() {
    let app = app_with(store());
    let res = send(
        &app,
        Req::new(OK_PLAN).body(r#"{"expectedVersion":9007199254740992}"#),
    )
    .await;
    assert_problem(&res, ProblemType::Validation);
    assert_eq!(res.json["errors"][0]["pointer"], "/expectedVersion");
    // the largest allowed value is valid; it just is not this plan's version
    let res = send(&app, Req::new(OK_PLAN).version(9_007_199_254_740_991)).await;
    assert_problem(&res, ProblemType::VersionConflict);
}

#[tokio::test]
async fn s4_2_plan_id_only_in_the_hyphenated_form() {
    let app = app_with(store());
    let upper = OK_PLAN.to_string().to_uppercase();
    let mut r = Req::new(OK_PLAN);
    r.plan = upper;
    assert_eq!(send(&app, r).await.status, StatusCode::OK);

    let simple = OK_PLAN.simple().to_string();
    for id in [
        format!("{{{OK_PLAN}}}"),
        format!("urn:uuid:{OK_PLAN}"),
        simple,
    ] {
        let mut r = Req::new(OK_PLAN);
        r.plan = id.clone();
        assert_problem(&send(&app, r).await, ProblemType::PlanNotFound);
    }
}

#[tokio::test]
async fn s4_2_two_idempotency_key_headers_are_refused() {
    let app = app_with(store());
    let req = Request::post(format!("/plans/{OK_PLAN}/save"))
        .header("authorization", format!("Bearer {}", token(CREATOR)))
        .header("content-type", "application/json")
        .header("idempotency-key", "first")
        .header("idempotency-key", "second")
        .body(Body::from(r#"{"expectedVersion":3}"#))
        .unwrap();
    assert_problem(
        &send_raw(&app, req).await,
        ProblemType::IdempotencyKeyMissing,
    );
}

// ---------- round 3: what a stranger's client relies on ----------

#[tokio::test]
async fn s4_3_any_options_is_answered_by_cors_not_405() {
    let app = app_with(store());
    let req = Request::builder()
        .method("OPTIONS")
        .uri(format!("/plans/{OK_PLAN}/save"))
        .body(Body::empty())
        .unwrap();
    let res = send_raw(&app, req).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.json, Value::Null, "empty body");
}

#[tokio::test]
async fn s4_3_a_path_segment_that_is_not_utf8_is_404_problem() {
    let app = app_with(store());
    let mut r = Req::new(OK_PLAN);
    r.plan = "%FF".into();
    assert_problem(&send(&app, r).await, ProblemType::PlanNotFound);
}

#[tokio::test]
async fn s4_5_same_key_same_version_other_plan_is_a_different_request() {
    let app = app_with(store());
    let first = send(&app, Req::new(AT_CAP).version(1).key("k-plan")).await;
    assert_eq!(first.status, StatusCode::OK, "{}", first.json);
    let res = send(&app, Req::new(AT_BUDGET).version(1).key("k-plan")).await;
    assert_problem(&res, ProblemType::IdempotencyKeyReused);
}

#[tokio::test]
async fn s4_2_content_type_is_case_insensitive_and_last_duplicate_member_wins() {
    let app = app_with(store());
    let mut r = Req::new(OK_PLAN).body(r#"{"expectedVersion":1,"expectedVersion":3}"#);
    r.content_type = Some("Application/JSON; charset=UTF-8");
    let res = send(&app, r).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
}

#[tokio::test]
async fn s3_amounts_never_exceed_2_pow_53_minus_1() {
    let store = store();
    let app = app_with(store.clone());
    let res = send(&app, Req::new(HUGE_PLAN).version(1)).await;
    assert_problem(&res, ProblemType::PlanRejected);
    for v in res.json["violations"].as_array().unwrap() {
        assert_eq!(v["amountVnd"], 9_007_199_254_740_991_i64, "{v}");
    }
    assert_eq!(store.plan(HUGE_PLAN).unwrap().status, PlanStatus::Draft);
}

#[tokio::test]
async fn s3_saved_at_and_saved_by_are_always_present() {
    let app = app_with(store());
    let res = send(&app, Req::new(OK_PLAN)).await;
    let obj = res.json.as_object().unwrap();
    for k in [
        "id",
        "tripId",
        "status",
        "version",
        "createdBy",
        "savedAt",
        "savedBy",
        "stops",
        "cost",
    ] {
        assert!(obj.contains_key(k), "missing {k}");
    }
    let saved_at = res.json["savedAt"].as_str().unwrap();
    // milliseconds and Z: 2026-12-05T02:15:00.000Z
    assert_eq!(saved_at.len(), 24, "{saved_at}");
    assert_eq!(&saved_at[19..20], ".");
}

// ---------- 4.8 GET /plans/{planId} ----------

async fn get(app: &Router, plan: &str, user: Option<Uuid>) -> Res {
    let mut b = Request::get(format!("/plans/{plan}"));
    if let Some(u) = user {
        b = b.header("authorization", format!("Bearer {}", token(u)));
    }
    send_raw(app, b.body(Body::empty()).unwrap()).await
}

#[tokio::test]
async fn s4_8_a_saved_plan_is_read_by_every_member_as_saved() {
    let app = app_with(store());
    let saved = send(&app, Req::new(OK_PLAN)).await;
    assert_eq!(saved.status, StatusCode::OK, "{}", saved.json);
    for user in [CREATOR, MEMBER] {
        let res = get(&app, &OK_PLAN.to_string(), Some(user)).await;
        assert_eq!(res.status, StatusCode::OK, "{}", res.json);
        assert_eq!(res.headers["content-type"], "application/json");
        assert_eq!(res.json, saved.json, "GET returns what the save returned");
    }
}

#[tokio::test]
async fn s4_8_a_draft_is_seen_only_by_its_creator() {
    let app = app_with(store());
    let res = get(&app, &OK_PLAN.to_string(), Some(CREATOR)).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    assert_eq!(res.json["status"], "draft");
    assert_eq!(res.json["version"], 3);
    assert!(res.json["savedAt"].is_null() && res.json["savedBy"].is_null());
    let res = get(&app, &OK_PLAN.to_string(), Some(MEMBER)).await;
    assert_problem(&res, ProblemType::NotPlanOwner);
}

#[tokio::test]
async fn s4_8_404_for_a_stranger_an_unknown_or_a_malformed_id() {
    let app = app_with(store());
    for (plan, user) in [
        (OK_PLAN.to_string(), STRANGER),
        (Uuid::from_u128(0xdead).to_string(), CREATOR),
        ("not-a-uuid".to_string(), CREATOR),
        (OK_PLAN.simple().to_string(), CREATOR),
        ("%FF".to_string(), CREATOR),
    ] {
        assert_problem(
            &get(&app, &plan, Some(user)).await,
            ProblemType::PlanNotFound,
        );
    }
}

#[tokio::test]
async fn s4_8_401_before_the_id_is_looked_at() {
    let app = app_with(store());
    for plan in [OK_PLAN.to_string(), "not-a-uuid".to_string()] {
        let res = get(&app, &plan, None).await;
        assert_problem(&res, ProblemType::Unauthenticated);
        assert_eq!(res.headers["www-authenticate"], "Bearer");
    }
}

#[tokio::test]
async fn s4_8_405_for_any_other_method_without_a_token() {
    let app = app_with(store());
    for method in ["POST", "PUT", "PATCH", "DELETE"] {
        let req = Request::builder()
            .method(method)
            .uri(format!("/plans/{OK_PLAN}"))
            .body(Body::empty())
            .unwrap();
        let res = send_raw(&app, req).await;
        assert_problem(&res, ProblemType::MethodNotAllowed);
        assert_eq!(res.headers["allow"], "GET, HEAD");
    }
}

#[tokio::test]
async fn s4_8_500_internal_never_leaks_the_cause() {
    let app = app_with(Arc::new(BrokenStore));
    let res = get(&app, &OK_PLAN.to_string(), Some(CREATOR)).await;
    assert_problem(&res, ProblemType::Internal);
    assert!(!res.json.to_string().contains("relation"));
}

#[tokio::test]
async fn s3_stored_amounts_are_reported_within_0_and_2_pow_53_minus_1() {
    let app = app_with(store());
    let res = get(&app, &HUGE_PLAN.to_string(), Some(CREATOR)).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    let stop = &res.json["stops"][0];
    assert_eq!(stop["items"][0]["unitPriceVnd"], 9_007_199_254_740_991_i64);
    assert_eq!(stop["costPerPersonVnd"], 9_007_199_254_740_991_i64);
    assert_eq!(res.json["cost"]["totalVnd"], 9_007_199_254_740_991_i64);
}

#[tokio::test]
async fn s3_a_budget_past_2_pow_53_minus_1_is_reported_as_2_pow_53_minus_1() {
    let s = InMemoryPlanStore::default();
    s.insert_trip(Trip {
        id: TRIP_A,
        budget_vnd: i64::MAX,
        members: vec![Member {
            user_id: CREATOR,
            cap_vnd: Some(-5),
        }],
    });
    s.insert_plan(plan(OK_PLAN, TRIP_A, CREATOR, 1, vec![item("t", 1, 1)]));
    let app = app_with(Arc::new(s));
    let res = get(&app, &OK_PLAN.to_string(), Some(CREATOR)).await;
    assert_eq!(res.json["cost"]["budgetVnd"], 9_007_199_254_740_991_i64);
    // a cap below 0 counts as 0: any cost over it breaks over-member-cap with limitVnd 0
    let res = send(&app, Req::new(OK_PLAN).version(1)).await;
    assert_problem(&res, ProblemType::PlanRejected);
    assert_eq!(res.json["violations"][0]["limitVnd"], 0);
}

#[tokio::test]
async fn s4_5_a_replay_after_an_edit_returns_the_stored_response() {
    let store = store();
    let app = app_with(store.clone());
    let first = send(&app, Req::new(OK_PLAN).key("k-edit")).await;
    assert_eq!(first.status, StatusCode::OK, "{}", first.json);
    // someone edits the plan: version 4, a draft again
    store.insert_plan(plan(OK_PLAN, TRIP_A, CREATOR, 4, vec![item("t", 1, 1)]));
    let replay = send(&app, Req::new(OK_PLAN).key("k-edit")).await;
    assert_eq!(replay.status, StatusCode::OK);
    assert_eq!(replay.headers["idempotent-replayed"], "true");
    assert_eq!(replay.json, first.json, "the stored body, version 3");
    let now = get(&app, &OK_PLAN.to_string(), Some(CREATOR)).await;
    assert_eq!(now.json["version"], 4);
    assert_eq!(now.json["status"], "draft");
}

#[tokio::test]
async fn s4_8_head_is_answered_like_get_without_a_body() {
    let app = app_with(store());
    let req = Request::head(format!("/plans/{OK_PLAN}"))
        .header("authorization", format!("Bearer {}", token(CREATOR)))
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.headers()["content-type"], "application/json");
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    assert!(bytes.is_empty());
}

#[tokio::test]
async fn s3_a_stored_plan_that_breaks_section_3_is_never_shown_or_saved() {
    type Break = fn(&mut Plan);
    fn second_stop(p: &mut Plan, day: &str, start: &str, end: &str) {
        let mut stop = p.stops[0].clone();
        stop.day = day.into();
        stop.start = start.into();
        stop.end = end.into();
        p.stops.push(stop);
    }
    let broken: [(&str, Break); 8] = [
        ("quantity 0", |p| p.stops[0].items[0].quantity = 0),
        ("start 25:00", |p| p.stops[0].start = "25:00".into()),
        ("end 9:00", |p| p.stops[0].end = "9:00".into()),
        ("day 2026-13-01", |p| p.stops[0].day = "2026-13-01".into()),
        ("version 0", |p| p.version = 0),
        ("end before start", |p| p.stops[0].end = "08:00".into()),
        ("overlap", |p| {
            second_stop(p, "2026-12-06", "10:00", "11:00")
        }),
        ("out of order", |p| {
            second_stop(p, "2026-12-05", "09:00", "10:00")
        }),
    ];
    for (what, breaks) in broken {
        let store = store();
        let mut p = plan(OK_PLAN, TRIP_A, CREATOR, 3, vec![item("t", 1_000, 1)]);
        breaks(&mut p);
        store.insert_plan(p);
        let app = app_with(store.clone());
        let res = get(&app, &OK_PLAN.to_string(), Some(CREATOR)).await;
        assert_problem(&res, ProblemType::Internal);
        let res = send(&app, Req::new(OK_PLAN)).await;
        assert_problem(&res, ProblemType::Internal);
        assert_eq!(
            store.plan(OK_PLAN).unwrap().status,
            PlanStatus::Draft,
            "{what}"
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn s4_3_two_saves_at_the_same_time_share_the_plan_once() {
    let store = store();
    let app = app_with(store.clone());
    let tasks: Vec<_> = (0..8)
        .map(|_| {
            let app = app.clone();
            tokio::spawn(async move { send(&app, Req::new(OK_PLAN)).await })
        })
        .collect();
    let mut saved_at = Vec::new();
    for t in tasks {
        let res = t.await.unwrap();
        assert_eq!(res.status, StatusCode::OK, "{}", res.json);
        saved_at.push(res.json["savedAt"].clone());
    }
    // one save happened; every other request saw it already saved (step 12)
    assert!(saved_at.windows(2).all(|w| w[0] == w[1]), "{saved_at:?}");
    let stored = store.plan(OK_PLAN).unwrap();
    assert_eq!(stored.status, PlanStatus::Saved);
    assert_eq!(
        stored
            .saved_at
            .unwrap()
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        saved_at[0].as_str().unwrap()
    );
}

#[tokio::test]
async fn s3_stops_may_touch_and_span_days() {
    let store = store();
    let mut p = plan(OK_PLAN, TRIP_A, CREATOR, 3, vec![item("t", 1_000, 1)]);
    let mut next = p.stops[0].clone();
    next.start = "10:30".into(); // the first one ends at 10:30
    next.end = "11:00".into();
    let mut tomorrow = p.stops[0].clone();
    tomorrow.day = "2026-12-07".into();
    tomorrow.start = "08:00".into();
    tomorrow.end = "08:30".into();
    p.stops.extend([next, tomorrow]);
    store.insert_plan(p);
    let app = app_with(store);
    let res = get(&app, &OK_PLAN.to_string(), Some(CREATOR)).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    assert_eq!(send(&app, Req::new(OK_PLAN)).await.status, StatusCode::OK);
}

#[tokio::test]
async fn s4_1_a_server_without_its_jwt_secret_answers_401_to_every_token() {
    let config = Config {
        port: 0,
        web_origins: vec!["http://localhost:3000".into()],
        database_url: None,
    };
    let app = build_app(&config, AppState::new(Db::disabled()).with_plans(store()));
    let res = send(&app, Req::new(OK_PLAN)).await;
    assert_problem(&res, ProblemType::Unauthenticated);
    let res = get(&app, &OK_PLAN.to_string(), Some(CREATOR)).await;
    assert_problem(&res, ProblemType::Unauthenticated);
}

#[tokio::test]
async fn s4_2_the_key_is_trimmed_of_spaces_and_compared_case_sensitively() {
    let app = app_with(store());
    let first = send(&app, Req::new(OK_PLAN).key("k-Case")).await;
    assert_eq!(first.status, StatusCode::OK, "{}", first.json);
    let trimmed = send(&app, Req::new(OK_PLAN).key("  k-Case  ")).await;
    assert_eq!(trimmed.headers["idempotent-replayed"], "true");
    // another case is another key: saved already at this version (step 12), not a replay
    let other = send(&app, Req::new(OK_PLAN).key("K-CASE")).await;
    assert_eq!(other.status, StatusCode::OK);
    assert!(other.headers.get("idempotent-replayed").is_none());
}
