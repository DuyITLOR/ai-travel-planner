---
name: review
description: "Readiness gate for the current branch before commit or PR: runs the four CI commands while a fresh reviewer checks the diff against the contract, the Rust and Next conventions and the CLAUDE.md rules. Step 3 of plan → implement → review."
argument-hint: "optional focus: auth, money, contract, web"
---

# Review the current branch

A second pair of eyes that did not write the code. Two tracks run together: the checks CI will run, and a review of the diff. One verdict at the end.

Scope: this branch against `main`, committed and uncommitted. Not other people's PRs.

## Step 1: Gather

Run in parallel: `git status --short`, `git diff main...HEAD`, `git diff HEAD`, `git log --oneline main..HEAD`. Nothing changed → stop.

Find the task: the `<!-- ATP-PLAN: KEY ... -->` marker, or `docs/plans/<KEY>.md` matching the branch. Take from the plan only `## Goal`, `## Requirements` and `## Contract changes`.

## Step 2: Start both tracks

**Checks.** Unless the diff is docs-only, run from the repo root, in the background:

```bash
pnpm format:check; pnpm lint; pnpm typecheck; pnpm test
```

Run all four even if one fails, and keep each one's result.

**Reviewer.** Reviewing a diff with the agent that wrote it is rubber-stamping. Spawn a fresh subagent with only: the diff, the goal/requirements/contract-changes text, and the instruction to follow Steps 3 to 6 of this file. Do not pass the plan's steps, your reasoning or what you think is fine. Skip the subagent only when the diff is under 10 lines and this session did not write it.

The reviewer loads `rust-api` when `apps/api/` is in the diff and `next-web` when `apps/web/` or `packages/shared/` is.

## Step 3: Read beyond the diff

- Read each changed file in full, not just the hunks.
- For every function, struct or schema whose shape or behaviour changed, find the callers and read them. A changed JSON field means: the Rust view struct, the Zod schema, the web code that reads it, and the contract.
- `+` lines are under review; `-` lines are context. Flag a removal only when it drops an auth check, a validation or an error path without a replacement.

## Step 4: Write hypotheses first

Before any checklist, write 3 to 5 concrete ways this change could be wrong, then check each in the code and record the result. Examples:

- "A non-member gets 403 here, which tells them the plan exists; the contract says 404."
- "This handler takes `Json<T>`: a malformed body answers axum's plain-text 422, not `invalid-json`."
- "The Zod schema gained a field the Rust view does not send, so `parse` throws on every response."

## Step 5: Five passes

One concern at a time. Checklists in [references/review-passes.md](references/review-passes.md).

- **A — Security and access**: `AuthUser`, membership before everything, 404 versus 403, secrets, what `detail` leaks.
- **B — Contract**: the contract changed first and alone; every problem `type`, status and title in its table; order of checks; Rust struct and Zod schema agree.
- **C — Logic and data**: money as integer VND, atomic section, idempotency, SQL through `.bind()`, time and id formats.
- **D — Conventions and design**: `rust-api` and `next-web` rules, proportionality, scope against the requirements.
- **E — Check integrity**: tests that can fail, read the contract, reach no network; nothing weakened to get to green.

## Step 6: Report

Use [references/report-template.md](references/report-template.md). Each finding: `file:line`, pass, confidence (`[certain]` · `[likely]` · `[possible]`), what goes wrong, and a suggested fix. Include the four command results. If nothing was found, say what was checked.

Verdict, one line: **Ready** (no CRITICAL or HIGH, all four commands pass) or **Not ready** with the count per severity.

Then `AskUserQuestion`:

- Findings or a failing command: **Fix** → fix inline for small fixes, `Skill(implement)` for ones that change the plan · **Stop**.
- Ready: **Stop here** (the user commits and opens the PR) · **Commit**, only on the user's say-so, on a branch, never to `main`.

The reviewer never edits files. Fixes happen after the report, by the main session.
