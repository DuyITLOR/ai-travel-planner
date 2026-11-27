# Review passes

Checks specific to this project. Generic bugs (null paths, off-by-one, dead code) are expected to be caught without a list. The conventions themselves live in `rust-api` and `next-web`; this file is what to look for in a diff.

## Pass A: Security and access

- **Every api endpoint except `/health` takes `AuthUser`**, and it is extracted before the body is read, so an unauthenticated caller learns nothing from a 400 or 415.
- **Membership is the first data check.** A caller who is not a member of the trip gets `404`, the same answer as for an id that does not exist. `403` is only for a member who lacks a right (not the creator). A `403` to a non-member is a leak.
- **Membership before the idempotency key.** A stored response is never replayed before the caller is confirmed to be a member now (incident: a removed member could replay a plan for 24 h).
- **The user id comes from the token**, never from the path, query or body.
- **`detail` and logs**: no SQL text, table name, stack trace, token or file path in a problem `detail`. Storage errors are logged with `tracing::error!` and answered with `Problem::internal()`. Web logs carry an error code, never an email, password or token.
- **Secrets**: nothing but the public `NEXT_PUBLIC_*` values (api URL, Supabase URL, anon key, site URL) reaches `apps/web`. A model key, `DATABASE_URL` or `service_role` in web code or a `NEXT_PUBLIC_*` name is CRITICAL. No key in a test, a fixture or a committed file.
- **Web session**: the server decides who the user is with `currentUser()` / `auth.getUser()`, never from `getSession()` alone or from anything the browser sent. A new signed-in page has its prefix in `PROTECTED_PREFIXES`. Every redirect target taken from input goes through `safeNextPath`.
- **Removed checks**: an `AuthUser` parameter, a membership check or a body rule deleted without a replacement.

## Pass B: Contract

- **Contract first**: if endpoint behaviour changed, `docs/contract/<endpoint>.md` changed too, in a commit of its own that comes before the code. Behaviour changed with no contract change is HIGH.
- **Problem table**: every `ProblemType` variant has a row; slug, status and title match the row exactly; the variant is in `ProblemType::ALL`. Every problem the handler can return is one the contract names for that endpoint.
- **No 500 for client input**: any input a client can send that reaches `Problem::internal()`, an `unwrap`, or an axum default rejection.
- **Order of checks** matches the contract's numbered list. Swapping two checks changes which error a client sees.
- **Both sides of the JSON**: the Rust view struct and the Zod schema in `packages/shared` changed together and agree on names (camelCase), nullability (`Option` ↔ `.nullable()`, absent ↔ `.optional()`) and integer-ness.
- **Compatibility**: a response field removed or renamed, a request field made required, a status changed. Additive is fine; anything else needs the contract's version note.
- **Headers**: a new request header the browser sends, or response header the web reads, is in `allow_headers` / `expose_headers` in `apps/api/src/lib.rs`.

## Pass C: Logic and data

- **Money**: `i64` in Rust, `z.number().int()` in Zod, field names end in `Vnd`/`_vnd`. No `f32`/`f64`, no `as` cast from a float, no division that drops dong silently. Sums and products use `saturating_*` or `checked_*` and are clamped to 0..=2^53−1 on the way out. The client never sends an amount the server should compute.
- **Atomic section**: lookup, checks, validator and write happen under one lock or one transaction; nothing is written before everything that can fail has run.
- **Validator**: a plan the validator rejected is never stored as saved or shown to the group. A change to `validator.rs` was planned and approved, not a side effect.
- **SQL**: every value through `.bind()`. A `format!` or string concatenation that builds SQL from a value is CRITICAL. Reads have a `LIMIT` or are by key; writes are scoped by id and by the caller's trip.
- **Ids and time**: path ids are parsed as the hyphenated 36-character UUID only; timestamps are RFC 3339 UTC with milliseconds; `day` is `YYYY-MM-DD`, times are `HH:MM` in Asia/Ho_Chi_Minh. Opening hours and "open now" compare in that zone, not UTC.
- **Write without read**: if what gets stored changed, who reads it (the `GET`, the view struct, the web) still works.
- **Web state**: an `Idempotency-Key` is created once per user intent and reused on retry, not regenerated per request. A `409 version-conflict` refetches instead of retrying blindly.

## Pass D: Conventions and design

- **Rust**: run the diff against `rust-api`: handler shape, no `Json<T>`/`Path<Uuid>` extractors where the contract names the error, store behind a trait and injected through `AppState`, domain failure enum mapped to `Problem` in the route module, no `unwrap`/`expect` on a request path.
- **Web**: run the diff against `next-web`: responses parsed with the shared schema, errors branched on problem `type`, `'use client'` only where needed, the right Supabase client for where the code runs, accessible controls, UI text in Vietnamese.
- **Proportionality**: an abstraction with one caller, a new dependency where the platform does it, a new endpoint where an existing one fits.
- **Scope**: every change traces to a requirement in the plan. Unrelated edits are flagged once, as scope.
- **Dependencies and migrations**: any in the diff were in the plan's "Needs a yes first".
- **Comments**: group comments that only restate the code into one note.

Design findings are MEDIUM or LOW unless they break compatibility or hide a failure.

## Pass E: Check integrity

- **A check that cannot fail**: `--passWithNoTests`, `|| true`, `continue-on-error`, a skipped or ignored test, an assertion that is always true.
- **Tests bent to the code**: a test, the eval data or a validator rule edited in the same change as the code it checks, with no contract change that explains it. HIGH: say which assertion changed and what it asserted before.
- **Contract tests read the contract**: expected status and title come from the contract file's table, never from `ProblemType::status()` or any value produced by the code under test.
- **No network**: a test that opens a socket, reaches Supabase, or calls a real LLM. Stores and the token check are injected (`with_plans`, `with_auth`); web tests stub `fetch`.
- **Coverage of the change**: new behaviour has a test, and each new problem type has a case. Mention once in the summary if missing.
- **Silenced tools**: a new `#[allow(...)]`, `eslint-disable`, `@ts-ignore` / `@ts-expect-error` or `any` without a stated reason.
