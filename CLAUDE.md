# AI Travel Planner

## Stack

- pnpm 10 workspace, Node 20 (`.nvmrc`), Rust 1.100.0 (`apps/api/rust-toolchain.toml`, same in CI).
- `apps/web`: Next.js 15, React 19, TypeScript 5.9 (Vercel). `packages/shared`: Zod 3 schemas of api responses.
- `apps/api`: Rust 2024, axum 0.8, sqlx 0.9 on Supabase Postgres (Render). Auth: Supabase HS256 tokens via `AuthUser`.
- Tests: Vitest; Rust tests in `apps/api/tests/` drive the router with `oneshot`. Contracts: `docs/contract/<endpoint>.md`.

## Commands

From the repo root after `pnpm install`; each covers TS and Rust, and CI runs exactly these.

- `pnpm lint`: ESLint + `cargo clippy --all-targets --locked -- -D warnings`
- `pnpm format:check`: Prettier + `cargo fmt --check` (fix with `pnpm format`)
- `pnpm typecheck`: `tsc --noEmit` per package + `cargo check --all-targets --locked`
- `pnpm test`: Vitest + `cargo test --locked`, contract tests included
- `pnpm build`: `next build` + `cargo build --release --locked`

## Conventions

- API errors are RFC 9457 problem details; every `type` is in the endpoint's `docs/contract/*.md` table.
- Contract first: change `docs/contract/` in its own commit, then the handler, then the tests.
- Contract tests read expected status and title from the contract file, never from the code under test.
- A changed api JSON shape updates the Rust struct and the Zod schema in `packages/shared` in the same PR.
- Money is integer VND (`i64`, Zod `int`), never a float. SQL only through sqlx `.bind()`, never concatenated.
- Tests reach no network: stores and the token check are injected through `AppState` (`with_plans`, `with_auth`).
- Branches `feat/<issue>-...`, `fix/...`, `chore/...`; everything reaches `main` through a PR with green CI.

## Never

- Put a model key, `DATABASE_URL` or the Supabase `service_role` key in `apps/web` or any `NEXT_PUBLIC_*` variable: Next.js inlines those into the browser bundle.
- Call a real LLM in unit or contract tests.
- Edit a test, the eval data or a validator rule just to make a check pass. If a test is wrong, fix the contract first.
- Save or share a plan the validator rejected, or replay a stored idempotent response before checking membership.
- Return 500 for bad client input; return the problem `type` the contract names.
- Make a check unable to fail: no `--passWithNoTests`, `|| true` or `continue-on-error` in a check.

## Permissions (enforced in `.claude/settings.json`)

- **Allowed:** read the code; run the `pnpm` commands and `pnpm dev`; edit source in `apps/` and `packages/`.
- **Ask first:** edit tests, `validator.rs` or `docs/contract/`; add a dependency or a migration; `supabase db push`; edit `.github/`, `render.yaml`, this file; `git push`.
- **Never:** read or print any `.env` file except `.env.example`; commit a secret; push to `main` (protection refuses it too); force-push; reset the cloud database.

## Incidents

<!-- Only real incidents, each traced to an AI-LOG entry or a commit. Never invent one. -->

- `--passWithNoTests` (Never): `apps/web` had no test file, so CI's test step was green while testing nothing. AI-LOG 2026-11-24 "the web test step could never fail".
- Contract tests read the contract (Conventions): our `assert_problem` compared with `ProblemType::status()`, so a 422 → 400 break failed 1 test, not 3. AI-LOG 2026-11-24 "contract and implementation".
- Replay before membership (Never): the key was looked up first, so a member removed from the trip could replay the plan for 24 h. AI-LOG 2026-11-24 "review round 1".
