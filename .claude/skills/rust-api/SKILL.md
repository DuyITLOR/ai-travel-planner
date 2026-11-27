---
name: rust-api
description: "Rules for Rust code in apps/api (axum 0.8, sqlx 0.9, Rust 2024): handler shape, problem-details errors, stores behind traits, auth and membership order, integer VND, contract tests. Load BEFORE writing, planning or reviewing anything under apps/api."
user-invocable: false
---

# Rust api conventions

The reference implementation is the save-plan endpoint. When unsure, copy its shape:
`src/routes/plans.rs` (handler), `src/plans/{model,store,validator}.rs` (domain), `src/problem.rs` (errors), `src/auth.rs` (caller), `tests/save_plan_contract.rs` (tests), `docs/contract/save-plan.md` (spec).

## Layout

| What                                  | Where                                                           |
| ------------------------------------- | --------------------------------------------------------------- |
| HTTP: extract, check, map to response | `src/routes/<feature>.rs`, registered in `src/routes/mod.rs`    |
| Domain types and response views       | `src/<feature>/model.rs`                                        |
| Storage trait and its implementations | `src/<feature>/store.rs`                                        |
| Pure rules                            | `src/<feature>/validator.rs` (ask before editing the plans one) |
| Shared state                          | `AppState` in `src/lib.rs`                                      |

A feature module never imports from `routes`; `routes` imports from features. No HTTP type (`StatusCode`, `Response`, `Problem`) below the route module.

## Handlers

1. **Thin outer function, fallible inner one.** The `pub async fn` handler extracts and returns `Response`; the work is in a private `async fn handle(...) -> Result<Response, Problem>` so `?` works and every error is a `Problem`.
2. **Extractor order**: `State(state)`, then `AuthUser(user)`, then path, headers, body. `AuthUser` before the body means an unauthenticated caller always gets 401, never a 400 about a body.
3. **Do not use extractors whose rejection the contract does not describe.** `Json<T>`, `Path<Uuid>` and `Query<T>` answer axum's plain-text 400/415/422 on bad input, which is not a problem document. Take `Result<Path<String>, PathRejection>` and parse the id yourself; take `Body`, bound it with `to_bytes(body, MAX_BODY_BYTES)`, then `serde_json::from_slice`. Each failure maps to the problem type the contract names.
4. **Checks run in the contract's order**, each with a comment carrying its step number (`// 3. Content-Type`). The order decides which error a client sees; changing it is a contract change.
5. **Every route has a method fallback**: `.route(path, get(h).fallback(not_allowed))` answering `405 method-not-allowed` with an `Allow` header.
6. **Path ids** are the hyphenated 36-character UUID form only (`parse_plan_id`): `Uuid::parse_str` alone also accepts braces, `urn:uuid:` and the simple form.
7. **Limits are named constants** next to the handler (`MAX_BODY_BYTES`, `MAX_KEY_LEN`), with the reason in a doc comment.
8. A browser-sent request header or a response header the web reads must be added to `allow_headers` / `expose_headers` in `build_app`.

## Errors

- Every error response is `Problem::new(ProblemType::X, "one sentence about this occurrence")`, plus `.with("member", json!(..))` for the extra members the contract names. Nothing else produces an error body.
- **New problem type**: contract table row first (own commit), then the variant, then `ALL` (and its length), `slug`, `status`, `title`. `s2_problem_table_matches_the_code` fails if they drift.
- **500 is for server faults only.** Bad client input never reaches `Problem::internal()`. A storage or serialisation failure is logged with `tracing::error!(error = %e, "...")` and answered with `Problem::internal()`; its text never goes into `detail`.
- **No `unwrap()`, `expect()`, `panic!`, indexing `[i]` or `as` narrowing on a request path.** `expect` is acceptable only for something that cannot fail by construction, with a message that says why. A poisoned mutex is recovered (`unwrap_or_else(|e| e.into_inner())`), not unwrapped.
- `anyhow` is for startup (`main.rs`, `config.rs`, `db.rs`). Request code returns typed failures.

## Stores

- Storage sits behind a trait in `store.rs` (`pub trait PlanStore: Send + Sync`), held in `AppState` as `Arc<dyn Trait>` and replaced through a `with_<name>` builder. Handlers never touch `PgPool` directly.
- Each store ships an in-memory implementation used by tests and until its Postgres one lands. Time comes from an injected `Clock`, never `Utc::now()` inside logic that tests need to move.
- A store returns a **domain failure enum** (`SaveFailure`: `NotFound`, `NotOwner`, `Internal(String)`, ...). The route module owns the single `to_problem` mapping. `Internal(String)` is logged, never sent.
- **One atomic section per operation**: one lock in memory, one transaction in Postgres. Inside it, in this order: existence and membership → ownership → stored idempotency key → version → validator → write. Compute everything that can fail before the first write.
- **Membership before the idempotency key**, always: a member removed from the trip must not get a stored response replayed.
- Only a success is stored under an idempotency key, in the same atomic step as the write.

## Access

- `AuthUser(Uuid)` is the only source of the caller's id. Never read a user id from the path, query or body to decide access.
- Not a member of the trip, or the thing does not exist → `404`, same body for both. A member without the right → `403`.
- Every query for a user-facing read or write is scoped by the caller's trip membership, in the store, not in the handler after the fact.

## Models and JSON

- Domain structs (`Plan`, `Trip`) and response structs (`PlanView`) are separate. Only views derive `Serialize`, with `#[serde(rename_all = "camelCase")]`; enums that appear in JSON use `rename_all = "snake_case"`.
- A view is built by `View::new(&domain, ...)`; fields the server computes (costs, counts) are computed there, never accepted from the client.
- `Option<T>` in a view is `null` in JSON. A member that must be absent needs `#[serde(skip_serializing_if = "Option::is_none")]`; the contract says which.
- A changed view means the Zod schema in `packages/shared` changes in the same PR.
- The server ignores request fields it does not know: read the body as `serde_json::Value` or a struct without `deny_unknown_fields`.

## Money, ids, time

- Money is `i64` dong, field names end in `_vnd`. No `f32`/`f64` anywhere near an amount.
- Arithmetic on amounts uses `saturating_add` / `saturating_mul` (or `checked_*`), and amounts are passed through `clamp` (0..=`MAX_VND`, 2^53−1) before they leave the server.
- Timestamps: `DateTime<Utc>`, written as `to_rfc3339_opts(SecondsFormat::Millis, true)`. `day` is `YYYY-MM-DD`, `start`/`end` are `HH:MM` in Asia/Ho_Chi_Minh; validate the exact shape, not just that chrono can parse it.
- Integers a JavaScript client reads stay within 1..=2^53−1.

## SQL

- sqlx with `.bind()` for every value. Never `format!` or concatenate a value into SQL. A dynamic identifier (sort column) comes from a fixed `match`, never from input.
- `Cargo.toml` turns sqlx's default features off, so do not assume the `query!` or `migrate!` macros are there: use `sqlx::query(..)` / `sqlx::query_as(..)`. Turning a feature on is a dependency change: ask first.
- Get the pool from `state.db.pool()`; `None` means not configured and is a `500 internal`, logged.
- Lists have a `LIMIT`. Schema changes are a new file from `supabase migration new`, never an edit of an applied migration; ask first.

## Tests

- One file per endpoint: `tests/<endpoint>_contract.rs`, driving `build_app` with `tower::ServiceExt::oneshot`. No network, no database, no real LLM: stores via `with_<name>`, tokens signed with a test secret via `with_auth`.
- Test names start with the contract section they check: `s2_409_when_the_plan_changed_since_review`, `s4_5_keys_are_per_caller`.
- **Expected status and title are read from the contract file's table** (`contract_table()`), never from `ProblemType::status()` or `title()`.
- Each problem type the endpoint can return has a case; each numbered check has a case showing it runs before the next one.
- Editing an existing test is "ask first". A failing test means the code is wrong, or the contract is and gets fixed first.

## Style

- Edition 2024: `let ... else`, `is_some_and`, `is_none_or`, let-chains instead of nested `if let`.
- `cargo fmt` and `cargo clippy --all-targets -- -D warnings` clean. No `#[allow(...)]` to quiet clippy without a comment giving the reason.
- `//!` at the top of a module says what it is and which contract section it implements. Comments explain why, and cite the contract (`contract section 4.3, step 7`).
- No new crate without asking; say the name, the version and what it replaces.
