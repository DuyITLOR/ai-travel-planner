# PA#2 — Harness and quality gate

Team: 23127178 Lê Nhựt Duy · 23127334 Nguyễn Thành Dâng · 23127344 Trương Thành Đạt
Submitted tag: `pa2`

## Links

| What                                     | Link                                                                                                                                                                                      |
| ---------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Repository (public)                      | https://github.com/DuyITLOR/ai-travel-planner                                                                                                                                             |
| Protected-branch settings (screenshot)   | [`docs/pa2/branch-protection.png`](docs/pa2/branch-protection.png)                                                                                                                        |
| Pull request a red check stopped         | https://github.com/DuyITLOR/ai-travel-planner/pull/3 (red run: https://github.com/DuyITLOR/ai-travel-planner/actions/runs/45529193604)                                                    |
| The commit that turned it green          | https://github.com/DuyITLOR/ai-travel-planner/pull/3/commits/62be07a8c8222fb025a184618aa26766793a9bf2 (green run: https://github.com/DuyITLOR/ai-travel-planner/actions/runs/45529372378) |
| Deliberate contract break that CI caught | https://github.com/DuyITLOR/ai-travel-planner/pull/5 (red run: https://github.com/DuyITLOR/ai-travel-planner/actions/runs/45534308292, closed without merging)                            |
| CI run for the tag `pa2`                 | https://github.com/DuyITLOR/ai-travel-planner/actions/workflows/ci.yml?query=branch%3Apa2 (the runs for the tag; the newest one is for the submitted commit)                              |

## Where each criterion lives

| #   | Criterion                   | Files                                                                                                                                                    |
| --- | --------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | Rules file                  | [`CLAUDE.md`](CLAUDE.md): Stack, Commands, Conventions, Never, Permissions, Incidents                                                                    |
| 2   | Permissions and secrets     | `CLAUDE.md` Permissions, [`.claude/settings.json`](.claude/settings.json), `apps/*/.env.example`, README "Model và spending cap"                         |
| 3   | One command for every check | `pnpm lint`, `pnpm format:check`, `pnpm typecheck`, `pnpm test`: README "Lệnh", [`ci.yml`](.github/workflows/ci.yml)                                     |
| 4   | CI that blocks a merge      | `ci.yml` jobs `lint`, `format`, `typecheck`, `test` = required checks on `main`                                                                          |
| 5   | Contract                    | [`docs/contract/save-plan.md`](docs/contract/save-plan.md): `POST /plans/{planId}/save`, with the `GET /plans/{planId}` it depends on                    |
| 6   | Contract tests              | [`apps/api/tests/save_plan_contract.rs`](apps/api/tests/save_plan_contract.rs), [`packages/shared/test/plan.test.ts`](packages/shared/test/plan.test.ts) |
| 7   | AI-LOG                      | [`AI-LOG.md`](AI-LOG.md)                                                                                                                                 |

## The deliberate contract break

The contract (section 2) says a plan the validator rejects answers `422 plan-rejected`. In the
break PR the code answers `400` instead, without touching the contract or the tests. The `test`
job of [run 45534308292](https://github.com/DuyITLOR/ai-travel-planner/actions/runs/45534308292)
went red on five tests (41 passed, 5 failed), the same five we saw locally:

- `s2_problem_table_matches_the_code`: the problem table in `docs/contract/save-plan.md` no longer
  matches `apps/api/src/problem.rs`.
- `s4_4_over_the_lowest_member_cap_is_rejected_and_not_shared`,
  `s4_4_both_rules_are_reported_in_order`, `s4_4_a_tie_on_the_lowest_cap_names_the_smallest_user_id`
  and `s3_amounts_never_exceed_2_pow_53_minus_1`: each gets `400` where the contract names `422`.

The PR was closed without merging; `main` never had the break.

## Timeline, stated plainly

- The gate came first. Ruleset "Protect main" (no bypass, PR required, no force push or
  deletion) existed before PR #2; the four required checks were added right after PR #2 was
  merged, on 2026-11-25 at 01:51 UTC, because a check can only be required once it has run.
- PR #3 was opened at 01:54 UTC, after the checks were required. The bug it fixes
  (`--passWithNoTests`) was found on 2026-11-24 (AI-LOG); we kept its two halves apart on
  purpose so the gate could be seen refusing the first commit. The red is real: run
  45529193604 failed and GitHub reported the PR as blocked until 62be07a.
- PR #4's three commits were made in one sitting on 2026-11-25 from work done the day
  before. No branch was rebased or force-pushed after it was pushed; every PR reached `main`
  as a merge commit (`git log --first-parent main`).

## How to re-run the checks

```bash
git clone https://github.com/DuyITLOR/ai-travel-planner && cd ai-travel-planner
git checkout pa2
pnpm install --frozen-lockfile  # Node 20, pnpm 10, rustup (installs Rust 1.100.0 from rust-toolchain.toml)
pnpm lint && pnpm format:check && pnpm typecheck && pnpm test
```
