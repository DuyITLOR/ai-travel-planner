import { z } from 'zod';

/** Khớp với `HealthResponse` trong apps/api/src/routes/health.rs. */
export const HealthResponse = z.object({
  status: z.literal('ok'),
  uptimeSec: z.number(),
  commit: z.string(),
  db: z.enum(['connected', 'not_configured', 'error']),
});
export type HealthResponse = z.infer<typeof HealthResponse>;
