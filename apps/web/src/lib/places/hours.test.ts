import { describe, expect, it } from 'vitest';
import { formatClock, openLabel, openStatus, saigonNow, slotsLabel } from './hours';
import type { OpeningHours } from './types';

const every = (slots: [string, string][]): OpeningHours => ({
  mon: slots,
  tue: slots,
  wed: slots,
  thu: slots,
  fri: slots,
  sat: slots,
  sun: slots,
});

describe('openStatus', () => {
  const museum = every([['07:30', '17:30']]);

  it('đang trong giờ mở cửa', () => {
    expect(openStatus(museum, { day: 'mon', minutes: 9 * 60 })).toEqual({
      open: true,
      closesAt: 17 * 60 + 30,
    });
  });

  it('giờ đóng là loại trừ', () => {
    expect(openStatus(museum, { day: 'mon', minutes: 17 * 60 + 30 }).open).toBe(false);
  });

  it('trước giờ mở: báo giờ mở hôm nay', () => {
    expect(openStatus(museum, { day: 'mon', minutes: 6 * 60 })).toEqual({
      open: false,
      opensAt: 7 * 60 + 30,
    });
  });

  it('khung giờ qua nửa đêm của hôm trước vẫn tính là mở', () => {
    const bar: OpeningHours = { ...every([]), fri: [['17:00', '26:00']] };
    expect(openStatus(bar, { day: 'sat', minutes: 60 })).toEqual({ open: true, closesAt: 120 });
    expect(openStatus(bar, { day: 'sat', minutes: 150 }).open).toBe(false);
  });

  it('mở cả ngày không có giờ đóng', () => {
    const always = every([['00:00', '24:00']]);
    expect(openStatus(always, { day: 'wed', minutes: 23 * 60 + 59 })).toEqual({
      open: true,
      closesAt: null,
    });
  });

  it('ngày nghỉ', () => {
    const closedSun: OpeningHours = { ...museum, sun: [] };
    expect(openStatus(closedSun, { day: 'sun', minutes: 600 })).toEqual({
      open: false,
      opensAt: null,
    });
  });

  it('nhiều khung trong ngày', () => {
    const church = every([
      ['07:00', '12:00'],
      ['14:00', '18:00'],
    ]);
    expect(openLabel(openStatus(church, { day: 'tue', minutes: 13 * 60 }))).toBe('Mở lúc 14:00');
  });
});

describe('formatting', () => {
  it('giờ quá 24 quay về hôm sau', () => {
    expect(formatClock(26 * 60)).toBe('02:00');
    expect(slotsLabel([['14:00', '26:00']])).toBe('14:00 – 02:00');
    expect(slotsLabel([])).toBe('Nghỉ');
    expect(slotsLabel([['00:00', '24:00']])).toBe('Cả ngày');
  });

  it('saigonNow dùng múi giờ UTC+7', () => {
    // 2026-11-23T17:30:00Z = Thứ Ba 00:30 ở TP.HCM.
    expect(saigonNow(new Date('2026-11-23T17:30:00Z'))).toEqual({ day: 'tue', minutes: 30 });
  });
});
