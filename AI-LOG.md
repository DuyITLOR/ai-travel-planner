# AI-LOG — AI Travel Planner

Team: Lê Nhựt Duy (23127178) · Nguyễn Thành Dâng (23127334) · Trương Thành Đạt (23127344) · CSC13114 · 23KTPM1

## Tools

| Tool                   | Used for                                                                                        |
| ---------------------- | ----------------------------------------------------------------------------------------------- |
| Claude Code (Opus 5.5) | PA#2: reading the brief and rubric, `CLAUDE.md`, CI, the save-plan contract, its code and tests |

## Data that leaves our system

- PA#2 sends no product data to any model: the agent is not built yet, and no test calls a model.
- Prompts to Claude Code contained our own repository and the course documents only. No keys, no
  `.env` files (denied in `.claude/settings.json`), no user data, no other team's work.
- From PA#3 (the agent): only place ids, place data and interest tags go to the model (Qwen on our
  own machine first, Gemini as fallback). Never names, emails, or check-in times and coordinates.

## Entries

## 2026-11-23 — understanding PA#2

Tool: Claude Code.
Asked for: a summary of what the PA#2 description, the rubric and the class brief require.
Kept: the seven criteria and the order of work (gate first, then every change through a PR).
Changed: we read "harness" as the agent loop around the LLM (tools, repair rounds). The rubric
says "the rules your assistant works under": the harness here is around our coding assistant.
The agent harness is PA#3.
Rejected: nothing.
By hand: the choice to keep the team's existing repo and stack (Next.js + Rust/axum + Supabase).

## 2026-11-23 — CLAUDE.md

Tool: Claude Code.
Asked for: a `CLAUDE.md` under 40 lines with stack / commands / conventions / never, three-tier
permissions, and an empty Incidents section, following a spec we wrote.
Kept: the four sections, the product-specific Never lines, the three permission tiers.
Changed: the first draft was written before the repo existed locally and guessed React + Vite, a
Makefile and `VITE_*` variables. The real repo is Next.js with pnpm scripts, so the most important
Never line ("no model key in the browser bundle") named the wrong prefix. Rewritten from the repo:
`NEXT_PUBLIC_*`, the four `pnpm` commands, sqlx with bound parameters.
Changed: the draft failed `prettier --check` (no blank line after headings). CI runs Prettier on
the whole repo, so committing it would have turned CI red. Formatted before commit.
Rejected: the Makefile. Windows machines in the team have no `make`, and the root `package.json`
already had one script per check.
By hand: the spec in the prompt: which sections, the three mandatory Never lines, the rule that
Incidents may only hold real incidents with a link.

## 2026-11-23 — README and CLAUDE.md commands

Tool: Claude Code.
Asked for: check that the commands in `CLAUDE.md` and `README.md` agree.
Kept: the command names, which already matched.
Changed: README described `pnpm lint` as "ESLint" and `pnpm test` as "Vitest", but the scripts also
run `cargo clippy` and `cargo test`. We fixed README rather than copying its wording into
`CLAUDE.md`, which would have told the assistant that Rust is not checked.
Rejected: nothing.
By hand: the decision that README follows `package.json`, not the other way round.

## 2026-11-24 — the web test step could never fail

Tool: Claude Code.
Asked for: audit CI against rubric criterion 3 ("the same commands, locally and in CI").
Found: `apps/web` ran `vitest run --passWithNoTests` and had no test file. Running it printed
"No test files found, exiting with code 0": CI showed a green test step that tested nothing.
Found: CI ran its own variants (`pnpm exec prettier`, `cargo clippy --locked`, filtered `pnpm`
calls) instead of the README commands, and never ran `cargo check`.
Changed: `ci.yml` now has one job per README command (`pnpm lint`, `pnpm format:check`,
`pnpm typecheck`, `pnpm test`); the flag is gone and `apps/web/src/lib/api.test.ts` adds two real
tests. Rule added to `CLAUDE.md` Incidents.
Rejected: keeping the two-job CI (TS / Rust). Required checks are chosen per job, and a job that
mixes four checks cannot say which one failed.
By hand: Duy (repo owner) made `lint`, `format`, `typecheck` and `test` required checks on `main`; we
merged PR #3 only after its first commit was blocked red and the second turned it green.

## 2026-11-24 — contract and implementation of `POST /plans/{planId}/save`

Tool: Claude Code.
Asked for: the contract of the save endpoint in the shape of our IA#3 `CONTRACT.md`, an
implementation behind it and contract tests.
Kept: the 11 problem types (13 after review round 1), the order of checks, idempotency per caller with a 24-hour key,
404 for a trip you are not in vs 403 for a member who is not the creator.
Changed: we broke the code on purpose three times to see whether the tests bite. Making the
rejected plan answer 400 instead of 422 failed only **one** test: `assert_problem` compared the
response with `ProblemType::status()`, i.e. the code checked itself. The helper now reads status
and title from the table in `docs/contract/save-plan.md`; the same break fails three tests (five once the
tests of review rounds 2 and 3 were added).
Rejected: the `idempotency-key-in-flight` (409) type from IA#3. The save runs under one lock, so a
second request with the same key can never see the first one half done; listing a type no test can
produce would be a promise we cannot check.
Rejected: a Postgres store in PA#2. The store is a trait with an in-memory implementation; the
Postgres one, in one transaction, comes with the shared place data in PA#3.
By hand: Nothing in the code. We did not re-read the handler line by line; the check was mechanical: each
rule came with a test that failed first, and CI ran all 46 on PR #4 before we merged it.

## 2026-11-24 — secrets in the history

Tool: Claude Code (ran gitleaks in Docker).
Asked for: check the whole git history for secrets before we make the repo public.
Kept: the result of `gitleaks git` over all branches: "16 commits scanned … no leaks found"
(`git rev-list --all --count` says 18; we report the tool's own line, and a grep for key patterns
over `git log --all -p` found nothing either).
Changed: nothing in history. In the working tree gitleaks flags the example `Idempotency-Key` UUID
in the contract as a "generic API key": a false positive, left as is.
Re-run on 2026-11-25, before the tag: "23 commits scanned", one finding, that same example UUID
(`docs/contract/save-plan.md` line 173, now in history through commit `6fe6faf`). Nothing else.
Rejected: nothing.
By hand: Duy made the repository public only after this scan came back clean.

## 2026-11-24 — review round 1

Tool: Claude Code, a second agent asked to grade the repo against the rubric as the lecturer would.
Asked for: an adversarial review: marks per criterion, and every place the contract and the code
disagree.
Kept: most of its findings, each fixed with a test that failed first:

- The `Idempotency-Key` lookup ran **before** the membership check. A member removed from a trip
  could resend a key within 24 hours and get the full plan back. The lookup now comes after
  membership and ownership (contract §4.3 steps 8–10); test
  `s4_1_a_member_who_left_the_trip_cannot_replay_a_stored_save`. Added to `CLAUDE.md` Never.
- The contract promised problem details for every error, but `GET` gave axum's empty 405 and a
  2 MB body gave a plain-text 413. Added `method-not-allowed` and `body-too-large` (16 KiB).
- "1 to 255 characters" for the key, while the code counted bytes: the contract now says printable
  ASCII.
- An ignored request field changed the idempotency fingerprint, so a retry that added a field got
  `422`. The fingerprint is now `planId` + `expectedVersion`.
- No test noticed if two checks swapped places, or if the 24-hour key window changed. Added
  two-faults-at-once tests and an injectable clock. We broke the code twice on purpose (TTL 48 h;
  media type checked before the key): each time one test went red.
- CORS answered `Access-Control-Allow-Headers: *`, which browsers never apply to `Authorization`.
  Now named explicitly, and `Idempotent-Replayed` is exposed.
- `CLAUDE.md` described a model trait that does not exist yet; Rust was unpinned.

Changed: its suggestion to also turn the global 404 fallback into problem details. That fallback
answers every unknown path, and `tests/health.rs` asserts its current body. Rather than edit that
test, the contract now scopes problem details to the save path.
Rejected: nothing else.
By hand: The review setup: we asked for a separate agent playing the lecturer, with the rubric, at least
five rounds, a fix after each, and every step needing our hands written to a TODO file.

## 2026-11-24 — review round 2

Tool: Claude Code, a fresh grading agent that did not see round 1.
Asked for: the same adversarial review, plus its own mutations of the code.
Kept, each with a test that fails first:

- Three mutations survived our tests: `>` changed to `>=` in both validator rules (no fixture cost
  exactly the cap or the budget), "first member with a cap" instead of the lowest (the 400k member
  happened to be listed before the 900k one), and dropping `WWW-Authenticate` from the CORS
  exposed headers. Added plans at exactly the cap and exactly the budget, put the 900k member
  first, and assert the exposed header. We re-ran the three mutations: 2, 3 and 1 tests go red.
- The contract promised `405` for every method other than POST, but a CORS preflight `OPTIONS`
  is answered `200` by the CORS layer. The contract now says so, and a test checks it.
- `expectedVersion` had no upper bound in the contract while the code rejected values above
  2⁶³−1. Bound set to 2⁵³−1 (what a JavaScript client holds exactly), in both.
- Behaviour the code had and the contract did not state: braced, `urn:uuid:` and unhyphenated
  plan ids were accepted; two `Idempotency-Key` headers silently used the first; ties on the
  lowest cap. The code now accepts only the 36-character form and refuses two keys; ties name the
  smallest user id. Each is a contract line and a test.
- `Edit(apps/**)` in `.claude/settings.json` let the assistant edit tests and the validator without
  asking, which is the one thing our Never list forbids. Tests, the validator and the contract are
  now "ask"; more `.env` read paths are denied.
- Cargo scripts lost `--locked` when CI moved to the `pnpm` commands; restored in
  `apps/api/package.json`, so a changed lockfile fails instead of being rewritten.

Changed: its "budget cap" wording. A Google Cloud budget only alerts; the real cap is that the
project has no billing account. README now says exactly that.
Rejected: PowerShell deny rules. Branch protection, not a local rule, is what stops a push to
`main`; we say so instead of listing every shell's spelling of `cat`.
By hand: Duy checked the Google Cloud project of `GEMINI_API_KEY` directly: it has no billing account, so
the README sentence is true.

## 2026-11-24 — review round 3

Tool: Claude Code, a third grading agent asked to read the contract as a stranger writing a client,
and to run its own mutations.
Kept, each with a test that fails first:

- Two contract sentences the code did not keep: `OPTIONS` without `Origin` also got `200` from
  the CORS layer (the contract only exempted a real preflight), and `POST /plans/%FF/save`
  returned axum's plain-text `400`, not a problem. The contract now says any `OPTIONS` is the
  CORS layer's; the handler maps axum's path rejection to `404 plan-not-found`.
- Three of its eleven mutations survived and now die: the fingerprint without `planId` (no test
  reused a key on another plan at the same version), a prefix match on `application/json`
  (`application/jsonx` passed), and `<` → `<=` on the 24-hour window (tests stepped ±60 s; they
  now hit 23:59:59 and exactly 24:00:00).
- Questions a client author could not answer: added §4.7 (what to do with each answer, which
  retries reuse the key), what an edit after a save does, that `userId` is absent rather than
  `null`, which amount each rule reports, and an upper bound for every amount (a cost past 2⁵³−1
  is reported as 2⁵³−1 and rejected).
- `500` promised "nothing was saved", but the store set `saved` before the last fallible step. The
  response is now built first and the plan changed last.
- The Zod schema let any problem carry any extra member and did not tie a slug to its status and
  title. It now does, and its test reads the contract's problem table (slug, status, title, extra
  member) and parses the `422` example.
- `.env.example` carried values (`PORT=4000`); the rubric asks for names only. Values are now
  comments, and the code treats an empty variable as unset; otherwise `PORT=` copied to `.env`
  would stop the api from starting.
- `CLAUDE.md` had grown to 56 lines with two Never lines that repeated Incidents, and its
  "Allowed" tier said tests could be edited freely while `settings.json` asks first. Merged to
  51 lines; the tiers now match the file that enforces them.

Rejected: its mutation of the 404 `detail` text. The contract fixes `type`, `status` and `title`,
not the wording of `detail`, so a test pinning that text would make wording changes breaking.
By hand: Nothing. The fixes were checked by tests that failed first, not by us re-reading them.

## 2026-11-24 — review round 4

Tool: Claude Code, a fourth grading agent that copied the tree to a fresh directory and followed
README literally (`pnpm install --frozen-lockfile`, the four checks, `pnpm build`), with the Rust
half run in Docker on the pinned 1.100.0.
Kept: everything went green on the fresh copy, with no lockfile drift under `--frozen-lockfile` or
`--locked`; it found no `ci.yml` step that would fail or not run on GitHub.
Changed: the numbers our own documents got wrong after three rounds of new tests. The deliberate
422 → 400 break now fails five tests, not three (we re-ran it: 41 passed, 5 failed) and `PA2.md`
lists all five; `plan.test.ts` has 8 tests, not 9. README said "four commands" next to five jobs,
and installed with `pnpm install` where CI uses `--frozen-lockfile`; both fixed.
Changed: `actions/checkout` and `actions/setup-node` moved to v5 (v4 runs on Node 20, past end of
life: warnings, not failures).
Rejected: nothing.
By hand: Nothing. The fresh-copy run was the agent's; we relied on CI repeating it on GitHub.

## 2026-11-24 — review round 5

Tool: Claude Code, a fifth grading agent that rehearsed our own plan for the GitHub evidence: it
cloned the repo, made the commits of each planned PR in order, and ran the four checks after each.
Kept: the plan works as written. PR-A is green with `--passWithNoTests` still in place; PR-B's
first commit is red only on `test` ("No test files found, exiting with code 1") and its second
commit green; each of the three contract-PR commits is green on its own; the deliberate break
fails exactly the five tests listed in `PA2.md`.
Changed: the zip step. `Compress-Archive` skips the hidden `.git` folder of a fresh clone (it
tested: zero `.git` entries), and the submission must contain `.git`; we use `tar.exe` instead.
Changed: squash merging off as well as rebase merging, so the red commit of the blocked PR stays
in the history of `main`.
Changed: contract §2 said "no other values" for `type` while §5 allows adding one; a client now
treats an unknown `type` by its status.
Rejected: nothing.
By hand: Duy set the repository to merge commits only (squash and rebase off) and created the ruleset
"Protect main"; we merged every PR with a merge commit and the break PR #5 was closed unmerged.

## 2026-11-25 — the GitHub evidence

Tool: Claude Code (git, `gh`, and Chrome for the screenshot).
Asked for: carry out our TODO plan on GitHub: one PR at a time, never merging itself.
Kept: PR #2 (rules and CI, green), PR #3 (red on `test` and blocked, then green), PR #4
(contract, code, tests in three commits, green), PR #5 (one-line 422 → 400 break: `test` red,
41 passed and 5 failed, the five names `PA2.md` lists; closed unmerged).
Changed: our plan said to screenshot Settings → Branches, but protection is a ruleset; the
screenshot is of the ruleset page, which a collaborator without admin rights can read.
Changed: commits carry no AI co-author trailer, at our request.
Rejected: merging by the assistant. Every merge was ours.
By hand: we merged PR #2, #3 and #4; Duy turned on the required checks between PR #2 and PR #3.
