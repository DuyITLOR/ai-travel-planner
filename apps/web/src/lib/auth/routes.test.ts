import { describe, expect, it } from 'vitest';
import { isProtectedPath, safeNextPath } from './routes';

describe('safeNextPath', () => {
  it('giữ đường dẫn nội bộ, kể cả query', () => {
    expect(safeNextPath('/trips/42?tab=chi-tieu')).toBe('/trips/42?tab=chi-tieu');
  });

  it.each([
    undefined,
    '',
    'https://evil.com',
    '//evil.com',
    '/\\evil.com',
    'trips',
    '/sign-in',
    '/sign-up?next=/x',
    '/trips\n',
  ])('đưa %j về trang chủ', (next) => {
    expect(safeNextPath(next)).toBe('/');
  });
});

describe('isProtectedPath', () => {
  it('khớp đúng tiền tố, không khớp nhầm', () => {
    expect(isProtectedPath('/trips')).toBe(true);
    expect(isProtectedPath('/trips/1')).toBe(true);
    expect(isProtectedPath('/tripsx')).toBe(false);
    expect(isProtectedPath('/')).toBe(false);
    expect(isProtectedPath('/sign-in')).toBe(false);
  });
});
