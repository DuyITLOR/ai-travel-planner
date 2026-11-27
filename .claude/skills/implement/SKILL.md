---
name: implement
description: "Execute an approved plan from docs/plans/: contract first, then Rust, shared schema, web and tests, then run the four CI commands. Step 2 of plan → implement → review."
argument-hint: "plan key (F2-03, issue-12)"
---

# Implement an approved plan

## Input

`$ARGUMENTS`: the plan key. Without one, take it from the `<!-- ATP-PLAN: ... -->` marker in the conversation.

## Before writing code

1. **Read the plan from disk**: `docs/plans/<KEY>.md`. The file is authoritative; the user may have edited it after approving. No file → stop: "No plan at `docs/plans/<KEY>.md`. Run `/plan <KEY>` first." Never invent a plan.

   Exception for a trivial change (one file, no endpoint behaviour, no "ask first" item): write a 5-line plan to that path with the marker and a `## Files to change` section, show it, get a yes, continue.

2. **Check the plan has `## Files to change`** with paths. Missing → stop and say so.

3. **Check the working tree.** Run `git branch --show-current` and `git status --short`.
   - On `main` → stop. Offer `git switch -c feat/<issue>-<slug>` (or `fix/`, `chore/`), do not run it unasked.
   - Uncommitted changes unrelated to this plan → stop and list them. Never stash, reset, restore or move work between branches yourself.

4. **Confirm the "Needs a yes first" list.** If the plan lists a dependency, a migration, or edits to tests, `validator.rs`, `docs/contract/`, `.github/` or `render.yaml`, approving the plan approved those and only those. Anything of that kind you discover later is a new question.

5. **Load the convention skills** for the areas in the marker: `rust-api` for `api`/`db`, `next-web` for `web`/`shared`. Read each target file before changing it.

6. **Decide who writes.** Delegate to the `implementer` subagent if any of these holds, otherwise work inline. Say which in one line.
   - `files` ≥ 3, or two or more areas
   - `complexity` ≥ 3
   - a migration, or a long test/build loop

   Give the subagent the plan path, not a retelling of it. Stay available to the user while it runs.

## Order of work

Follow the plan's steps. The order is fixed, because each step is checked against the one before:

1. **Contract.** Edit `docs/contract/<endpoint>.md` and nothing else. Then ask the user to commit it alone (`docs: ...`, saying why the contract changes) before any code. A contract and its handler never share a commit.
2. **Migration**, if planned: `supabase migration new <name>`. Never `supabase db push` or a reset from this skill.
3. **Rust**: model → store → handler → route registration. A new problem type goes into `ProblemType` only after it is in the contract table.
4. **Shared schema**: the Zod schema in `packages/shared` for every JSON shape the Rust change touched. Same PR, always.
5. **Web.**
6. **Tests**: contract tests in `apps/api/tests/`, schema tests in `packages/shared`, web tests next to the file. New behaviour gets a test; expected status and title come from the contract file.

## Rules while writing

- The plan is the scope. A refactor, rename or "while I'm here" fix that is not in it goes in the report as a suggestion, not in the diff.
- The plan is wrong (a file is not what it assumed, a step cannot work) → stop and say what you found. Do not improvise around it.
- A test fails → fix the code. A test, the eval data or a validator rule is never changed to make a check pass; if the test is wrong, the contract is fixed first, in its own commit.
- No comment that repeats the line under it. Comment the why: a contract section, an invariant, a unit, an ordering constraint.
- Never read a `.env` file; `.env.example` only.

## After writing

Run all four from the repo root, the same ones CI runs, and fix what they report:

```bash
pnpm format && pnpm lint && pnpm typecheck && pnpm test
```

`pnpm format` writes; the other three must pass unchanged. A failure that your change did not cause is reported, not hidden. Never add `--passWithNoTests`, `|| true`, an `#[allow]`, an `eslint-disable` or a `@ts-expect-error` to get to green.

For a web change, start `pnpm dev` and look at the screen in a browser before calling it done; say plainly if you could not.

## Report

- Files changed, grouped by area
- The four commands: pass or fail, with the failing output
- Anything in the plan that was skipped or done differently, and why
- What still needs a human: a manual check, a migration to push, an env var to set on Render or Vercel

Do not commit (beyond the contract commit the user approved), push or open a PR unless asked.

Then `AskUserQuestion`: **Review** → `Skill(review)` (recommended) · **Stop**. If a command failed: **Fix now** · **Stop**.
