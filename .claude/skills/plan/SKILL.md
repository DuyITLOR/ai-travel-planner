---
name: plan
description: "Plan a task before any code is written: read the task, explore the code, write an implementation plan to docs/plans/ and wait for approval. Step 1 of plan → implement → review."
argument-hint: "task id (F2-03), issue number (#12) or a short description"
---

# Plan a task

Explore the codebase and produce a plan the user approves before anything is edited.

**Do not use `EnterPlanMode`/`ExitPlanMode`.** This skill owns its protocol: the plan is a file the user can edit, with one approval gate at the end.

**Reads only.** No `Edit` or `Write` in this skill except the plan file itself. Anything else belongs in `implement`, after approval.

## Input

`$ARGUMENTS`: a board task id (`F2-03`), a GitHub issue number (`#12`), or a description.

## Step 0: Existing plan

Work out the plan key: the task id, `issue-12`, or a short kebab-case slug of the description. If `docs/plans/<KEY>.md` exists, show it and ask whether to revise it or go to `/implement`.

## Step 1: Read the task

- **Board task:** the data is the JSON inside `<script type="application/json" id="tasks-data">` at the end of `docs/tasks.html`. Read the task (`title`, `doneWhen`, `assignee`), its feature (`description`, `owners`) and the other tasks of that feature.
- **Issue:** `gh issue view <n>`.
- **Description only:** restate it in two lines and confirm with the user before exploring.

If the task belongs to a feature the caller does not own (`owners`), say so before going further.

## Step 2: Load the convention skills

Load them before exploring, so the exploration looks for the right things:

- touches `apps/api/` → `rust-api`
- touches `apps/web/` or `packages/shared/` → `next-web`

## Step 3: Explore

Read what the task touches. For more than one area, use parallel `Explore` agents, one per area.

- The contract, if the endpoint has one: `docs/contract/<endpoint>.md`. If the task adds or changes an endpoint and there is no contract, the plan's first step writes it.
- The nearest existing example to copy: `apps/api/src/routes/plans.rs` + `apps/api/src/plans/` for an endpoint, `apps/api/tests/save_plan_contract.rs` for its tests, `packages/shared/src/plan.ts` for its schema, `apps/web/src/app/(auth)/` for a screen with a form.
- Shared place data when the task reads places, menus or travel times: `data/README.md` describes every field.
- The UI design canvas for a screen: match the artboard for that screen (ask the user for the link if it is not in the conversation).
- Existing tests that cover the code being changed.

## Step 4: Write the plan

Path: `docs/plans/<KEY>.md` (git-ignored: a working file, not a deliverable). Line 1 is a marker `implement` and `review` read; keep it on one line.

```markdown
<!-- ATP-PLAN: <KEY> complexity=<1-4> files=<N> areas=<api,web,shared,db,docs> -->
# Plan: <KEY> — <summary>

## Goal
What the user can do when this is done, and the task's "done when", quoted.

## Requirements
- One line each, **each with its source in brackets**: [tasks.html F2-03], [proposal §4],
  [contract save-plan §4.3], [design: 04 · Khám phá], [user request], [my own addition].
- Anything not asked for is marked [my own addition] so it can be cut.

## Contract changes
- The endpoint(s), and for each: new file or changed sections of `docs/contract/<endpoint>.md`.
- Every new problem `type`: slug, status, title, when.
- `_none_` if no endpoint behaviour changes.

## Files to change
- One repo-relative path per line, new files marked `(new)`. Mandatory: `implement` stops without it.
- List a file if unsure; a false positive is cheaper than a missed file.

## Steps
Numbered, each one small enough to review, in this order:
1. Contract (`docs/contract/`), committed alone
2. Migration, if any
3. Rust: model → store → handler → route
4. `packages/shared` schema (same PR as the Rust struct)
5. Web
6. Tests

## Needs a yes first
Everything CLAUDE.md lists as "ask first" that this plan does: new dependency (name, version, why),
migration, edits to tests, `validator.rs`, `docs/contract/`, `.github/`, `render.yaml`.
`_none_` if nothing.

## Tests
Which test files, which cases (name the contract section each case checks), and how to see it work by hand.

## Risks
Auth and membership, money, idempotency, data shape changes that break the web, anything irreversible.
```

### Complexity

Score from what exploration found, not from how the task sounds: 1 = one file, no behaviour change of an endpoint · 2 = one concern, one area · 3 = several files across areas, or a new endpoint · 4 = schema, auth or validator change, or anything hard to undo. Repeating one small change over many files is 2, not 3.

## Step 5: Challenge the plan (complexity ≥ 2)

Spawn a fresh `Explore` agent as a sceptical reviewer. Give it only the plan text and the task text, not your findings or reasoning. It reads the code to check the plan and returns 3 to 8 points, each `[blocker]` or `[suggestion]`:

- Do the named files, types and endpoints exist as the plan says?
- Is the footprint proportionate to the task? Is there a smaller plan?
- Does anything break a CLAUDE.md rule (contract first, money as integer VND, replay after membership, no 500 for client input, tests without network)?
- What did it miss: a problem type not in the contract table, the Zod schema, CORS headers, a protected route prefix, a caller?

Resolve every `[blocker]` in the plan file, or record why it is rejected. Append `## Challenge` listing each point and what was done about it.

## Step 6: Present and gate

1. Print the whole plan inline, then `Plan written to: docs/plans/<KEY>.md`.
2. `AskUserQuestion`: **Implement** (recommended) · **Revise** · **Stop**.

- **Implement** → invoke `Skill(implement)` with the key. Never start editing directly: `implement` owns the branch check, the contract-first order and the verification.
- **Revise** → the user edits the file or says what to change; update the file, show it again.
