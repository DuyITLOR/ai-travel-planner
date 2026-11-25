# `POST /plans/{planId}/save` — contract, version 1

With its companion `GET /plans/{planId}` (section 4.8), which a client needs to get the
`version` it saves.

AI Travel Planner api (`apps/api`) · frozen for PA#2, 26 Nov 2026

Saving is the step that shares a schedule with the group (brief, section 3, step 6).
Before it, a plan is a **draft** only its creator sees; after it, every member of the
trip sees it. This is the endpoint whose mistakes cost the most: a plan over a
member's cap that slips through reaches all four people, and a retried request
that saves twice sends the group two copies.

This file is the specification. `apps/api/tests/save_plan_contract.rs` checks the
implementation against it, and fails if the problem table in section 2 and the
code disagree. If the code and this file disagree, the code is wrong. If this file
is wrong, change it in its own commit that says why, before changing the code.

## 1. Conventions

- Base URL: the api origin: `http://localhost:4000` locally; the deployed api on Render
  has its own origin, which the web app reads from `NEXT_PUBLIC_API_URL`.
- Requests and responses are JSON, UTF-8. Field names are `camelCase`.
- **Money is an integer number of dong (VND)**, in fields ending in `Vnd`. Never a
  float, never thousands of dong. The server computes every amount; the client
  never sends one.
- Ids (`planId`, `tripId`, user ids) are UUIDs, lowercase, e.g.
  `0b6f8a4e-1c2d-4e5f-8a9b-0c1d2e3f4a5b`. User ids are Supabase Auth user ids.
- Timestamps are RFC 3339 strings in UTC with milliseconds, e.g.
  `2026-12-05T02:15:00.000Z`. `day` is `YYYY-MM-DD`; `start`/`end` are `HH:MM`,
  24-hour, Asia/Ho_Chi_Minh time.
- Clients ignore response fields they do not know. The server ignores request
  fields it does not know.
- Browsers: for the origins in the api's `WEB_ORIGIN`, CORS allows the request
  headers `Authorization`, `Content-Type` and `Idempotency-Key`, and exposes the
  response headers `Idempotent-Replayed` and `WWW-Authenticate` to scripts.

## 2. Errors

Every 4xx and 5xx response to a request for the path `/plans/{planId}/save` or
`/plans/{planId}` (with a non-empty `planId` segment), whatever its method, is a **problem details**
object (RFC 9457) with `Content-Type: application/problem+json`:

```http
HTTP/1.1 422 Unprocessable Content
Content-Type: application/problem+json

{
  "type": "https://atp.example/problems/plan-rejected",
  "title": "Plan rejected by the validator",
  "status": 422,
  "detail": "The plan breaks 1 rule; it was not saved.",
  "violations": [
    {
      "rule": "over-member-cap",
      "message": "Each person would spend 450000 VND, over the lowest member cap of 400000 VND.",
      "userId": "9c4e2b1a-7d3f-4a6b-8e5c-1f2a3b4c5d6e",
      "amountVnd": 450000,
      "limitVnd": 400000
    }
  ]
}
```

| Member       | Rule                                                                                                                                                                |
| ------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `type`       | `https://atp.example/problems/` followed by a slug from the table below; in version 1 there are no others yet. A later version may **add** a type (section 5): a client that meets an unknown `type` treats it by its `status` (4xx: show `detail`, do not retry; 5xx: as `internal`). (`.example` is reserved: the URI names the problem, it is not a web page) |
| `title`      | the title from the table below, exactly                                                                                                                             |
| `status`     | a number, equal to the HTTP status of the response                                                                                                                  |
| `detail`     | one sentence about **this** occurrence. Never a stack trace, an SQL message, a table name, a token or a file path                                                   |
| extra member | only those named in the table: `errors`, `currentVersion`, `violations`                                                                                             |

| Slug                      | Status | Title                          | When                                                                                                                                   | Extra member                                                                                           |
| ------------------------- | ------ | ------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------ |
| `invalid-json`            | 400    | Malformed JSON                 | the body is not valid JSON (an empty body included)                                                                                    |                                                                                                        |
| `validation`              | 400    | Invalid request                | the body breaks a rule of section 4.2                                                                                                  | `errors`: one `{ "pointer", "detail" }` per broken field; `pointer` is a JSON Pointer, `""` = the body |
| `idempotency-key-missing` | 400    | Idempotency-Key required       | no `Idempotency-Key` header, or one that breaks the rule of section 4.2 (empty, longer than 255, or a character outside printable ASCII) |                                                                                                        |
| `unauthenticated`         | 401    | Authentication required        | no `Authorization: Bearer` token, or one that is invalid, expired or not for this project. The response also has `WWW-Authenticate: Bearer` |                                                                                                        |
| `not-plan-owner`          | 403    | Not the plan's creator         | the caller is a member of the plan's trip but did not create the plan: on a save (4.1), or on reading a plan that is still a draft (4.8) |                                                                                                        |
| `plan-not-found`          | 404    | Plan not found                 | no plan with that id, `planId` is not a UUID, **or the plan belongs to a trip the caller is not a member of**                          |                                                                                                        |
| `method-not-allowed`      | 405    | Method not allowed             | a method the path does not accept: anything but `POST` on `/save` (`Allow: POST`), anything but `GET`/`HEAD` on `/plans/{planId}` (`Allow: GET, HEAD`) |                                                                                                        |
| `version-conflict`        | 409    | Plan changed since review      | `expectedVersion` is not the plan's current `version`: someone edited it after the caller reviewed it                                  | `currentVersion`: integer                                                                              |
| `body-too-large`          | 413    | Request body too large         | the body is larger than 16 KiB (16384 bytes)                                                                                           |                                                                                                        |
| `unsupported-media-type`  | 415    | Unsupported media type         | `Content-Type` is missing or is not `application/json` (parameters such as `; charset=utf-8` are allowed)                              |                                                                                                        |
| `plan-rejected`           | 422    | Plan rejected by the validator | the backend validator found at least one violation (section 4.4). Nothing was saved or shared                                         | `violations`: array, at least one entry                                                                |
| `idempotency-key-reused`  | 422    | Idempotency-Key reused         | the caller already used this key, within 24 hours, for a different request (section 4.5)                                               |                                                                                                        |
| `internal`                | 500    | Internal error                 | anything else. `detail` is a fixed, generic sentence. Nothing was saved                                                                |                                                                                                        |

Clients decide what to do from `type`, never from `title` or `detail`.

## 3. Resources

**Plan**

```json
{
  "id": "0b6f8a4e-1c2d-4e5f-8a9b-0c1d2e3f4a5b",
  "tripId": "7a3e9c10-55b2-4d7e-9f00-3c2b1a0d9e8f",
  "status": "saved",
  "version": 3,
  "createdBy": "5d1c7f0a-2b3c-4d5e-8f9a-0b1c2d3e4fe2",
  "savedAt": "2026-12-05T02:15:00.000Z",
  "savedBy": "5d1c7f0a-2b3c-4d5e-8f9a-0b1c2d3e4fe2",
  "stops": [
    {
      "placeId": "hcm-ben-thanh-market",
      "day": "2026-12-06",
      "start": "09:00",
      "end": "10:30",
      "items": [
        { "itemId": "bt-banh-xeo", "name": "Bánh xèo", "unitPriceVnd": 60000, "quantity": 1 }
      ],
      "costPerPersonVnd": 60000
    }
  ],
  "cost": { "perPersonVnd": 60000, "memberCount": 4, "totalVnd": 240000, "budgetVnd": 4000000 }
}
```

| Field                      | Rule                                                                                                        |
| -------------------------- | ----------------------------------------------------------------------------------------------------------- |
| `status`                   | `"draft"` or `"saved"`. Every `200` of the save has `"saved"`; `GET` (4.8) shows a draft to its creator     |
| `version`                  | integer from 1 to 2⁵³−1. Goes up by one on every edit of the plan. **Saving does not change it**            |
| `createdBy`                | the member who created the plan, the only one who may save it                                               |
| `savedAt`, `savedBy`       | always present. `null` while `draft`; set by the save that shares this `version`, never changed by a repeat. `savedAt` has milliseconds and ends in `Z` |
| `stops`                    | in schedule order. May be empty                                                                             |
| `items[].quantity`         | integer ≥ 1, **per person**: every member follows the same schedule                                         |
| `items[].unitPriceVnd`     | the price in the shared place data when the item was put in the plan                                        |
| `costPerPersonVnd`         | sum of `unitPriceVnd × quantity` over the stop's items                                                      |
| `cost.perPersonVnd`        | sum of `costPerPersonVnd` over all stops                                                                    |
| `cost.memberCount`         | number of members of the trip (1 for an individual trip)                                                    |
| `cost.totalVnd`            | `perPersonVnd × memberCount`                                                                                |
| `cost.budgetVnd`           | the trip budget: the group total, or the one person's amount for an individual trip                         |
| every `…Vnd` field         | integer from 0 to 2⁵³−1, whether stored (`unitPriceVnd`, `budgetVnd`, a member's cap) or computed. A stored amount outside that range is reported, and validated, as the nearest end (below 0 as 0); a computed amount that would be larger is reported as 9007199254740991, which is over any budget, so such a plan is always rejected |
| `start`, `end`             | `00:00` to `23:59`                                                                                          |

**Editing after a save.** Plans are edited through endpoints outside this contract
(they come with the agent in PA#3). Every edit, of a draft or of a saved plan, adds
one to `version` and sets `status` back to `"draft"`, `savedAt` and `savedBy` back
to `null`: the group keeps seeing nothing new until the creator saves the new
version with this endpoint. The current content and `version` are read with
`GET /plans/{planId}` (section 4.8); after a `409` the client reads the plan again
and shows it to the user before saving.

**Violation** (inside `plan-rejected`)

| Field       | Rule                                                                                          |
| ----------- | --------------------------------------------------------------------------------------------- |
| `rule`      | a rule code from section 4.4. **Clients must accept codes they do not know** (section 5)      |
| `message`   | one sentence for the user, in English                                                         |
| `userId`    | present only on `over-member-cap` (absent, not `null`, on every other rule): the member whose cap is the lowest |
| `amountVnd` | `over-member-cap`: `cost.perPersonVnd`. `over-trip-budget`: `cost.totalVnd`                   |
| `limitVnd`  | `over-member-cap`: that member's cap. `over-trip-budget`: `cost.budgetVnd`                    |

## 4. The endpoint

### 4.1 Who may call it

- **Role:** a signed-in user. Send the Supabase Auth access token of the session
  (HS256, audience `authenticated`, not expired; a server started without its JWT
  secret answers `401` to every token):
  `Authorization: Bearer <access token>`. There is no anonymous access and no
  admin override.
- **Ownership:** the caller must be a **member of the plan's trip** _and_ the
  **member who created the plan** (`createdBy`).
  - Not a member of the trip → `404 plan-not-found`, exactly as if the plan did
    not exist. A stranger cannot learn that a plan id exists.
  - A member, but not the creator → `403 not-plan-owner`. Members already see that
    the trip has a draft, so nothing leaks by saying who may save it.

### 4.2 Request

```http
POST /plans/0b6f8a4e-1c2d-4e5f-8a9b-0c1d2e3f4a5b/save HTTP/1.1
Authorization: Bearer eyJhbGciOiJIUzI1NiIs...
Content-Type: application/json
Idempotency-Key: 9f1c2e7a-3b4d-4c5e-9f6a-7b8c9d0e1f2a

{ "expectedVersion": 3 }
```

| Part                     | Rule                                                                                                                       |
| ------------------------ | -------------------------------------------------------------------------------------------------------------------------- |
| `planId` (path)          | the plan's UUID in the hyphenated 36-character form (`8-4-4-4-12` hex digits; upper or lower case). Any other form, including braces, `urn:uuid:` or no hyphens, is `404 plan-not-found`. A trailing slash is a different path, outside this contract |
| `Idempotency-Key` header | exactly one such header, 1 to 255 printable ASCII characters (bytes `0x20`–`0x7E`) after trimming leading and trailing spaces, chosen by the client, **one per save the user intends**. A UUID v4 is a good choice. Compared exactly (case-sensitive). Two or more `Idempotency-Key` headers are `400 idempotency-key-missing` |
| `Content-Type` header    | `application/json`, compared without regard to case, optionally followed by parameters (`; charset=utf-8`). Other `+json` types such as `application/json-patch+json` are `415` |
| body                     | a JSON object. If a member appears twice, the last one counts                                                               |
| `expectedVersion`        | required, integer from 1 to 9007199254740991 (2⁵³−1, the largest integer a JavaScript client holds exactly): the `version` of the draft the user reviewed and confirmed. `3.0`, `"3"`, `0` and anything larger are invalid |

The body carries no money and no stops: the server saves the plan exactly as it
is stored, after checking it.

### 4.3 Order of checks

The server checks in this order and stops at the first step that fails:

0. The method is `POST`, else `405 method-not-allowed` with `Allow: POST` (no token
   needed). The one exception is `OPTIONS`, with or without `Origin`: it is answered by
   the CORS layer with `200`, an empty body and, for an allowed origin, the CORS
   headers of section 1. It is never a problem and never needs a token.
1. Bearer token valid, else `401 unauthenticated`. The scheme name `Bearer` is
   case-insensitive.
2. `Idempotency-Key` follows section 4.2, else `400 idempotency-key-missing`.
3. `Content-Type` is `application/json`, else `415 unsupported-media-type`.
4. The body is at most 16 KiB, else `413 body-too-large`.
5. The body is valid JSON, else `400 invalid-json`.
6. The body follows section 4.2, else `400 validation` listing **every** broken field.
7. `planId` is a UUID in the form of section 4.2, else `404 plan-not-found`. A path
   segment that is not valid UTF-8 once percent-decoded (e.g. `%FF`) is also
   `404 plan-not-found`.
8. The plan exists and the caller is a member of its trip, else `404 plan-not-found`.
9. The caller created the plan, else `403 not-plan-owner`.
10. **The key was used before** by this caller within 24 hours (section 4.5): replay
    the stored response, or `422 idempotency-key-reused`. This comes after steps 8
    and 9 on purpose: a member who has left the trip cannot replay a stored plan.
11. `expectedVersion` equals the plan's `version`, else `409 version-conflict`
    with `currentVersion`.
12. The plan is already `saved` at this version: `200` with the Plan, unchanged.
    Nothing is shared a second time.
13. The validator runs (section 4.4). Any violation: `422 plan-rejected`; the plan
    stays `draft`, nothing is shared.
14. The plan becomes `saved`, `savedAt` and `savedBy` are set, the group can see it:
    **`200`** with the Plan.

Steps 8 to 14 are **atomic**: two requests at the same time for the same plan
cannot both save it, and the key is stored together with the save.

### 4.4 The validator

The same validator the agent calls as `validate_plan` runs again here, in the
backend, on the stored plan. A plan that breaks any rule is never saved.

| Rule code          | Breaks when                                                                                        |
| ------------------ | -------------------------------------------------------------------------------------------------- |
| `over-member-cap`  | `cost.perPersonVnd` is greater than the **lowest** personal cap among the trip's members           |
| `over-trip-budget` | `cost.totalVnd` is greater than `cost.budgetVnd`                                                   |

A plan can break both; then `violations` has one entry for each, in this order.
Members without a personal cap are bound only by the trip budget. A cost **equal**
to a limit is allowed. When several members share the lowest cap, `userId` is the
one with the smallest id. These two rules
are enforced in version 1. The place rules of the brief (a place or item that
does not exist, outside opening hours, travel time longer than the gap between
stops, an outdoor stop when rain is above 60%) are added as new rule codes when
the shared place data lands; that is an additive change (section 5).

### 4.5 Idempotency

A client that gets no answer (a timeout, a dropped mobile connection) sends the
same request again with the same key. That retry must not save twice or share twice.

- Keys are **per caller**: two users using the same key do not affect each other.
- Two requests are _the same_ when they have the same `planId` and the same
  `expectedVersion`. Fields the server ignores (section 1), key order and whitespace
  do not make requests different.
- Same key, same request, within 24 hours → the **stored** response: `200`, the
  identical body (same `savedAt`), plus the header **`Idempotent-Replayed: true`**.
  The first, original response does not carry that header.
- Same key, different request, within 24 hours → `422 idempotency-key-reused`.
  Nothing changes.
- **Only a `200` is stored.** A request that ends in any error stores nothing, so the
  client may fix the problem and send it again with the same key.
- A key is kept for exactly 24 hours from the save that stored it: a request
  23:59:59 later still replays, one 24:00:00 later or more is handled as new.
- A replay is the response **as stored at the save**, even if the plan was edited
  since: it says the save of that version happened, not what the plan is now. The
  current plan is read with `GET /plans/{planId}` (section 4.8).
- Independently of keys, saving is safe to repeat: a new key on a plan already
  saved at that version returns `200` with the plan unchanged (step 12).

### 4.6 Responses

| Status | Body       | When                                                                                       |
| ------ | ---------- | ------------------------------------------------------------------------------------------ |
| `200`  | a Plan     | saved now, saved already at this version, or a replay (`Idempotent-Replayed: true`)       |
| 4xx/5xx | a problem | section 2                                                                                  |

A `200` has `Content-Type: application/json`. A client does not need to tell the three
kinds of `200` apart: in every case the plan is saved and was shared with the group
exactly once, at `savedAt`. Show "Saved" either way.

### 4.7 What a client does with each answer

| Answer                                   | Retry?                                   | What to do                                                                                     |
| ---------------------------------------- | ---------------------------------------- | ---------------------------------------------------------------------------------------------- |
| `200`                                    | no                                       | show the saved plan; the group sees it now                                                     |
| no answer (timeout, connection lost)     | **yes, same key, same body**             | the server either replays the stored `200` or saves now; it never saves twice                  |
| `401 unauthenticated`                    | yes, after refreshing the Supabase session; same key | if refreshing fails, send the user to sign in                                    |
| `403 not-plan-owner`                     | no                                       | tell the user only the plan's creator can save it                                              |
| `404 plan-not-found`                     | no                                       | the plan is gone or the user left the trip; go back to the trip                                |
| `409 version-conflict`                   | after the user reviews again, **new key**, `expectedVersion` = `currentVersion` | read the plan (`GET /plans/{planId}`), show what changed, ask the user to confirm again |
| `422 plan-rejected`                      | no                                       | show each violation's `message`; the user edits the plan (which is a new version)              |
| `422 idempotency-key-reused`             | with a **new key**                       | a client bug: one key was used for two different saves                                        |
| `400` (any type), `405`, `413`, `415`    | no                                       | a client bug; fix the request                                                                  |
| `500 internal`                           | yes, same key, with backoff (1 s, 2 s, 4 s…) | nothing was saved; after 3 failures tell the user to try later                             |

### 4.8 Reading a plan: `GET /plans/{planId}`

What a client calls to show a plan and to learn the `version` it will send as
`expectedVersion`.

```http
GET /plans/0b6f8a4e-1c2d-4e5f-8a9b-0c1d2e3f4a5b HTTP/1.1
Authorization: Bearer eyJhbGciOiJIUzI1NiIs...
```

No body, no `Idempotency-Key`: reading changes nothing and is always safe to retry.
`HEAD` is answered like `GET`, without a body. Checks, in this order:

0. The method is `GET` or `HEAD`, else `405 method-not-allowed` with
   `Allow: GET, HEAD` (no token needed). `OPTIONS` is the CORS layer's, as in 4.3.
1. Bearer token valid (as in 4.1), else `401 unauthenticated`.
2. `planId` is a UUID in the form of section 4.2, else `404 plan-not-found`.
3. The plan exists and the caller is a member of its trip, else `404 plan-not-found`.
4. The plan is `saved`, or the caller created it, else `403 not-plan-owner`: a draft
   is its creator's own until it is saved.
5. **`200`** with the Plan (section 3) as it is now, `Content-Type: application/json`.
   After a save and before any edit, it equals the save's `200` body.

| Answer                 | What a client does                                                       |
| ---------------------- | ------------------------------------------------------------------------ |
| `200`                  | show the plan; keep its `version` for the save                           |
| `401 unauthenticated`  | refresh the Supabase session and read again                              |
| `403 not-plan-owner`   | the creator has not saved it yet; tell the user the plan is not shared yet |
| `404 plan-not-found`   | the plan is gone or the user left the trip; go back to the trip          |
| `500 internal`         | retry with backoff (1 s, 2 s, 4 s…)                                      |

## 5. When this contract changes

- **Within version 1, only additive changes**: a new optional request field, a new
  response field, a new problem type, **a new validator rule code**. Clients must
  ignore unknown fields and treat an unknown `rule` like any other violation
  (show its `message`).
- Anything else is **breaking**: renaming or removing a field, changing a type or
  a unit, making a rule stricter for the request, changing the status of a
  problem type, or changing who may call the endpoint. A breaking change is
  served at a new path, `/v2/plans/{planId}/save`. This path keeps working for at
  least two checkpoints (about two weeks) after `/v2` ships, and its responses then
  carry `Deprecation` (RFC 9745) and `Sunset` (RFC 8594) headers with the date.
- Every change, additive or breaking, is made here first, in its own commit, and
  the contract tests are updated in the same pull request as the code, never in a
  commit that only makes a failing test pass.
- Changes are listed below.

| Date        | Change                       |
| ----------- | ---------------------------- |
| 26 Nov 2026 | Version 1: the save (4.1–4.7) and the read it depends on (4.8) |
