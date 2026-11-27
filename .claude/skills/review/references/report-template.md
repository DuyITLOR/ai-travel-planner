# Review report

```
## Review: <KEY or branch>

**Verdict: Ready | Not ready** — <n> critical, <n> high, <n> medium, <n> low

### What changed
Two lines: what the branch does and why.

### Checks
| Command             | Result                            |
| ------------------- | --------------------------------- |
| `pnpm format:check` | pass / fail: <first failing line> |
| `pnpm lint`         | ...                               |
| `pnpm typecheck`    | ...                               |
| `pnpm test`         | ... (<n> tests)                   |

### Investigation
- Files read in full: ...
- Callers checked: <function> → <n> call sites
- Hypotheses: <n> tested, <n> confirmed

### Findings

#### CRITICAL
- `path:line` [A security] [certain] — what goes wrong, with the input that triggers it. Fix: ...

#### HIGH
#### MEDIUM
#### LOW

### Requirements
For each requirement in the plan: met / not met / not checkable, with the file that shows it.
```

Omit empty severity sections. With no findings, say so and list what was verified.

## Severity

| Severity     | Means                                             | Examples                                                                                                                   |
| ------------ | ------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| **CRITICAL** | A leak, or data that ends up wrong for the group  | secret in `apps/web`, SQL built from a value, endpoint without `AuthUser`, replay before membership, a rejected plan saved |
| **HIGH**     | Wrong behaviour a user or a client will hit       | 500 for client input, problem type not in the contract, Rust and Zod disagree, float money, a test edited to pass          |
| **MEDIUM**   | Works, but breaks a convention or will bite later | check order differs from the contract, handler holds logic that belongs in the store, missing test for new behaviour       |
| **LOW**      | Minor                                             | naming, a comment that restates the code, a small simplification                                                           |

## Do not flag

- Style that Prettier, rustfmt, ESLint or clippy already decide
- Problems in code the branch did not touch (one line under "Noticed, not in scope" at most)
- A `-` line that the `+` line next to it fixes
- Missing tests file by file: once, in the summary
