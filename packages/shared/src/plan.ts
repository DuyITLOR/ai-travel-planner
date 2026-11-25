import { z } from 'zod';

/**
 * Response shapes of `POST /plans/{planId}/save` — docs/contract/save-plan.md.
 * Matches `PlanView` in apps/api/src/plans/model.rs and `Problem` in apps/api/src/problem.rs.
 * `packages/shared/test/plan.test.ts` checks this file against the contract.
 */

/** 2^53 - 1: the largest integer the contract sends. */
export const MAX_SAFE = Number.MAX_SAFE_INTEGER;

/** Integer number of dong, 0 to 2^53 - 1. Never a float. */
const Vnd = z.number().int().min(0).max(MAX_SAFE);
const Uuid = z.string().uuid();
/** `HH:MM`, 00:00 to 23:59. */
const Time = z.string().regex(/^([01]\d|2[0-3]):[0-5]\d$/);

export const PlanItem = z
  .object({
    itemId: z.string(),
    name: z.string(),
    unitPriceVnd: Vnd,
    quantity: z.number().int().min(1),
  })
  .passthrough();

export const PlanStop = z
  .object({
    placeId: z.string(),
    day: z.string().regex(/^\d{4}-\d{2}-\d{2}$/),
    start: Time,
    end: Time,
    items: z.array(PlanItem),
    costPerPersonVnd: Vnd,
  })
  .passthrough();

/** Clients ignore fields they do not know (contract section 1), hence `passthrough`. */
export const Plan = z
  .object({
    id: Uuid,
    tripId: Uuid,
    status: z.enum(['draft', 'saved']),
    version: z.number().int().min(1).max(MAX_SAFE),
    createdBy: Uuid,
    /** RFC 3339, UTC, milliseconds: `2026-12-05T02:15:00.000Z`. */
    savedAt: z.string().datetime({ precision: 3 }).nullable(),
    savedBy: Uuid.nullable(),
    stops: z.array(PlanStop),
    cost: z
      .object({
        perPersonVnd: Vnd,
        memberCount: z.number().int().min(1),
        totalVnd: Vnd,
        budgetVnd: Vnd,
      })
      .passthrough(),
  })
  .passthrough();
export type Plan = z.infer<typeof Plan>;

export const PROBLEM_TYPE_BASE = 'https://atp.example/problems/';

type ProblemSpec = {
  status: number;
  title: string;
  extra?: 'errors' | 'currentVersion' | 'violations';
};

/** Section 2 of the contract: every problem type, its status, title and extra member. */
export const SAVE_PLAN_PROBLEMS = {
  'invalid-json': { status: 400, title: 'Malformed JSON' },
  validation: { status: 400, title: 'Invalid request', extra: 'errors' },
  'idempotency-key-missing': { status: 400, title: 'Idempotency-Key required' },
  unauthenticated: { status: 401, title: 'Authentication required' },
  'not-plan-owner': { status: 403, title: "Not the plan's creator" },
  'plan-not-found': { status: 404, title: 'Plan not found' },
  'method-not-allowed': { status: 405, title: 'Method not allowed' },
  'version-conflict': {
    status: 409,
    title: 'Plan changed since review',
    extra: 'currentVersion',
  },
  'body-too-large': { status: 413, title: 'Request body too large' },
  'unsupported-media-type': { status: 415, title: 'Unsupported media type' },
  'plan-rejected': {
    status: 422,
    title: 'Plan rejected by the validator',
    extra: 'violations',
  },
  'idempotency-key-reused': { status: 422, title: 'Idempotency-Key reused' },
  internal: { status: 500, title: 'Internal error' },
} as const satisfies Record<string, ProblemSpec>;
export type SavePlanProblemSlug = keyof typeof SAVE_PLAN_PROBLEMS;

export const Violation = z
  .object({
    /** Open set: clients must accept rule codes they do not know. */
    rule: z.string(),
    message: z.string(),
    userId: Uuid.optional(),
    amountVnd: Vnd,
    limitVnd: Vnd,
  })
  .passthrough()
  .superRefine((v, ctx) => {
    if (v.rule === 'over-member-cap' && v.userId === undefined) {
      ctx.addIssue({ code: 'custom', path: ['userId'], message: 'required on over-member-cap' });
    }
    if (v.rule === 'over-trip-budget' && v.userId !== undefined) {
      ctx.addIssue({ code: 'custom', path: ['userId'], message: 'absent on over-trip-budget' });
    }
  });

const EXTRA_SCHEMAS = {
  errors: z.array(z.object({ pointer: z.string(), detail: z.string() })).min(1),
  currentVersion: z.number().int().min(1).max(MAX_SAFE),
  violations: z.array(Violation).min(1),
} as const;

/** RFC 9457 problem details as this endpoint sends them (contract section 2). */
export const Problem = z
  .object({
    type: z.string().startsWith(PROBLEM_TYPE_BASE),
    title: z.string(),
    status: z.number().int().min(400).max(599),
    detail: z.string().min(1),
  })
  .catchall(z.unknown())
  .superRefine((p, ctx) => {
    const slug = p.type.slice(PROBLEM_TYPE_BASE.length);
    const spec: ProblemSpec | undefined = (SAVE_PLAN_PROBLEMS as Record<string, ProblemSpec>)[slug];
    // A type this version does not know is allowed (contract sections 2 and 5): the client
    // treats it by its status, so there is nothing more to check here.
    if (!spec) return;
    if (p.status !== spec.status) {
      ctx.addIssue({ code: 'custom', path: ['status'], message: `${slug} is ${spec.status}` });
    }
    if (p.title !== spec.title) {
      ctx.addIssue({ code: 'custom', path: ['title'], message: `${slug} is "${spec.title}"` });
    }
    const base = new Set(['type', 'title', 'status', 'detail']);
    for (const key of Object.keys(p)) {
      if (base.has(key)) continue;
      if (key !== spec.extra) {
        ctx.addIssue({ code: 'custom', path: [key], message: `${slug} has no ${key}` });
        continue;
      }
      const parsed = EXTRA_SCHEMAS[spec.extra].safeParse(p[key]);
      if (!parsed.success) {
        ctx.addIssue({ code: 'custom', path: [key], message: parsed.error.message });
      }
    }
    if (spec.extra && !(spec.extra in p)) {
      ctx.addIssue({
        code: 'custom',
        path: [spec.extra],
        message: `${slug} carries ${spec.extra}`,
      });
    }
  });
export type Problem = z.infer<typeof Problem>;

/** The slug of a problem `type`, e.g. `plan-rejected`. */
export function problemSlug(problem: Problem): string {
  return problem.type.slice(PROBLEM_TYPE_BASE.length);
}
