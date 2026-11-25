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
| CI run for the tag `pa2`                 | https://github.com/DuyITLOR/ai-travel-planner/actions/workflows/ci.yml?query=branch%3Apa2                                                                                                 |

## Where each criterion lives

| #   | Criterion                   | Files                                                                                                                                                    |
| --- | --------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | Rules file                  | [`CLAUDE.md`](CLAUDE.md): Stack, Commands, Conventions, Never, Permissions, Incidents                                                                    |
| 2   | Permissions and secrets     | `CLAUDE.md` Permissions, [`.claude/settings.json`](.claude/settings.json), `apps/*/.env.example`, README "Model và spending cap"                         |
| 3   | One command for every check | `pnpm lint`, `pnpm format:check`, `pnpm typecheck`, `pnpm test`: README "Lệnh", [`ci.yml`](.github/workflows/ci.yml)                                     |
| 4   | CI that blocks a merge      | `ci.yml` jobs `lint`, `format`, `typecheck`, `test` = required checks on `main`                                                                          |
| 5   | Contract                    | [`docs/contract/save-plan.md`](docs/contract/save-plan.md): `POST /plans/{planId}/save`                                                                  |
| 6   | Contract tests              | [`apps/api/tests/save_plan_contract.rs`](apps/api/tests/save_plan_contract.rs), [`packages/shared/test/plan.test.ts`](packages/shared/test/plan.test.ts) |
| 7   | AI-LOG                      | [`AI-LOG.md`](AI-LOG.md)                                                                                                                                 |

## The deliberate contract break

The contract (section 2) says a plan the validator rejects answers `422 plan-rejected`. In the
break PR the code answers `400` instead, without touching the contract or the tests. The `test`
job goes red on five tests (41 passed, 5 failed; run locally before opening the PR, to be
confirmed against the red run linked above):

- `s2_problem_table_matches_the_code`: the problem table in `docs/contract/save-plan.md` no longer
  matches `apps/api/src/problem.rs`.
- `s4_4_over_the_lowest_member_cap_is_rejected_and_not_shared`,
  `s4_4_both_rules_are_reported_in_order`, `s4_4_a_tie_on_the_lowest_cap_names_the_smallest_user_id`
  and `s3_amounts_never_exceed_2_pow_53_minus_1`: each gets `400` where the contract names `422`.

The PR was closed without merging; `main` never had the break.

## How to re-run the checks

```bash
git clone https://github.com/DuyITLOR/ai-travel-planner && cd ai-travel-planner
git checkout pa2
pnpm install            # Node 20, pnpm 10, rustup (installs Rust 1.100.0 from rust-toolchain.toml)
pnpm lint && pnpm format:check && pnpm typecheck && pnpm test
```
