import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import { Plan, Problem, SAVE_PLAN_PROBLEMS, problemSlug } from '../src';

const contract = readFileSync(
  fileURLToPath(new URL('../../../docs/contract/save-plan.md', import.meta.url)),
  'utf8',
);

/** The Plan example of section 3 (the ```json block that has a tripId). */
function planExample(): Plan {
  const blocks = [...contract.matchAll(/```json\n([\s\S]*?)```/g)].map((m) => JSON.parse(m[1]!));
  return blocks.find((b) => (b as { tripId?: string }).tripId) as Plan;
}

/** JSON bodies of the ```http examples (what follows the blank line after the headers). */
function httpBodies(): unknown[] {
  return [...contract.matchAll(/```http\n([\s\S]*?)```/g)]
    .map((m) => m[1]!.split('\n\n').slice(1).join('\n\n').trim())
    .filter((body) => body.startsWith('{'))
    .map((body) => JSON.parse(body));
}

describe('save-plan contract (docs/contract/save-plan.md)', () => {
  it('the Plan example of section 3 parses', () => {
    expect(Plan.safeParse(planExample()).success).toBe(true);
  });

  it('the 422 example of section 2 parses as a problem', () => {
    const problems = httpBodies().filter((b) => (b as { type?: string }).type);
    expect(problems.length).toBeGreaterThan(0);
    for (const p of problems) {
      const parsed = Problem.safeParse(p);
      expect(parsed.success, JSON.stringify(parsed.error?.issues)).toBe(true);
    }
  });

  it('SAVE_PLAN_PROBLEMS is the problem table of section 2: slug, status, title, extra', () => {
    const rows = [
      ...contract.matchAll(/^\| `([a-z-]+)`\s*\|\s*(\d{3})\s*\|([^|]+)\|[^|]*\|([^|]*)\|/gm),
    ];
    const fromDoc = Object.fromEntries(
      rows.map((m) => {
        const extra = /^\s*`([A-Za-z]+)`/.exec(m[4]!)?.[1];
        return [m[1], { status: Number(m[2]), title: m[3]!.trim(), ...(extra ? { extra } : {}) }];
      }),
    );
    expect(fromDoc).toEqual(SAVE_PLAN_PROBLEMS);
  });
});

describe('Problem schema', () => {
  const conflict = {
    type: 'https://atp.example/problems/version-conflict',
    title: 'Plan changed since review',
    status: 409,
    detail: 'The plan was edited after you reviewed it.',
    currentVersion: 4,
  };

  it('accepts a problem with the extra member of its type', () => {
    expect(problemSlug(Problem.parse(conflict))).toBe('version-conflict');
  });

  it('rejects a wrong status, a wrong title, a foreign or missing extra member', () => {
    expect(Problem.safeParse({ ...conflict, status: 400 }).success).toBe(false);
    expect(Problem.safeParse({ ...conflict, title: 'Conflict' }).success).toBe(false);
    expect(Problem.safeParse({ ...conflict, violations: [] }).success).toBe(false);
    const noExtra: Record<string, unknown> = { ...conflict };
    delete noExtra.currentVersion;
    expect(Problem.safeParse(noExtra).success).toBe(false);
    expect(Problem.safeParse({ error: 'not_found' }).success).toBe(false);
  });

  it('a type this version does not know is accepted, to be handled by its status', () => {
    const future = {
      type: 'https://atp.example/problems/trip-locked',
      title: 'Trip locked',
      status: 423,
      detail: 'The trip is locked.',
    };
    expect(Problem.safeParse(future).success).toBe(true);
    expect(Problem.safeParse({ ...future, status: 200 }).success).toBe(false);
  });

  it('userId is present only on over-member-cap', () => {
    const rejected = {
      type: 'https://atp.example/problems/plan-rejected',
      title: 'Plan rejected by the validator',
      status: 422,
      detail: 'The plan breaks 1 rule; it was not saved.',
      violations: [{ rule: 'over-trip-budget', message: 'x', amountVnd: 2, limitVnd: 1 }],
    };
    expect(Problem.safeParse(rejected).success).toBe(true);
    const withUser = {
      ...rejected,
      violations: [{ ...rejected.violations[0], userId: '9c4e2b1a-7d3f-4a6b-8e5c-1f2a3b4c5d6e' }],
    };
    expect(Problem.safeParse(withUser).success).toBe(false);
  });
});

describe('Plan schema', () => {
  it('money is an integer from 0 to 2^53 - 1', () => {
    const plan = planExample();
    for (const totalVnd of [240000.5, -1, Number.MAX_SAFE_INTEGER + 1]) {
      expect(Plan.safeParse({ ...plan, cost: { ...plan.cost, totalVnd } }).success).toBe(false);
    }
  });

  it('times are 00:00 to 23:59 and savedAt has milliseconds', () => {
    const plan = planExample();
    const stop = plan.stops[0]!;
    expect(Plan.safeParse({ ...plan, stops: [{ ...stop, start: '25:99' }] }).success).toBe(false);
    expect(Plan.safeParse({ ...plan, savedAt: '2026-12-05T02:15:00Z' }).success).toBe(false);
  });
});
