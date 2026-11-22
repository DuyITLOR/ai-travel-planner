import { expect, it } from 'vitest';
import { HealthResponse } from '../src';

it('HealthResponse chấp nhận payload hợp lệ', () => {
  const ok = { status: 'ok', uptimeSec: 1, commit: 'local', db: 'not_configured' };
  expect(HealthResponse.safeParse(ok).success).toBe(true);
  expect(HealthResponse.safeParse({ ...ok, db: 'x' }).success).toBe(false);
});
