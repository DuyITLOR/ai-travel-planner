import { DAYS, type Day, type OpeningHours } from './types';

const DAY_LABEL: Record<Day, string> = {
  mon: 'Thứ Hai',
  tue: 'Thứ Ba',
  wed: 'Thứ Tư',
  thu: 'Thứ Năm',
  fri: 'Thứ Sáu',
  sat: 'Thứ Bảy',
  sun: 'Chủ nhật',
};

export function dayLabel(day: Day): string {
  return DAY_LABEL[day];
}

/** Thời điểm theo giờ TP.HCM: thứ trong tuần và số phút từ 00:00. */
export type LocalTime = { day: Day; minutes: number };

export function saigonNow(date: Date = new Date()): LocalTime {
  const parts = new Intl.DateTimeFormat('en-GB', {
    timeZone: 'Asia/Ho_Chi_Minh',
    weekday: 'short',
    hour: '2-digit',
    minute: '2-digit',
    hourCycle: 'h23',
  }).formatToParts(date);
  const get = (type: string) => parts.find((p) => p.type === type)?.value ?? '';
  const day = get('weekday').toLowerCase().slice(0, 3) as Day;
  return { day, minutes: Number(get('hour')) * 60 + Number(get('minute')) };
}

function toMinutes(hhmm: string): number {
  const [h = '0', m = '0'] = hhmm.split(':');
  return Number(h) * 60 + Number(m);
}

/** `1530` phút → `"01:30"` (giờ quá 24 quay về hôm sau). */
export function formatClock(minutes: number): string {
  const m = ((minutes % 1440) + 1440) % 1440;
  return `${String(Math.floor(m / 60)).padStart(2, '0')}:${String(m % 60).padStart(2, '0')}`;
}

function previousDay(day: Day): Day {
  return DAYS[(DAYS.indexOf(day) + 6) % 7] as Day;
}

export type OpenStatus =
  { open: true; closesAt: number | null } | { open: false; opensAt: number | null };

/**
 * Đang mở hay không tại `now`, tính cả khung giờ của hôm trước kéo quá nửa đêm.
 * `closesAt` = null khi mở cả ngày (00:00–24:00); `opensAt` = giờ mở tiếp theo trong hôm nay.
 */
export function openStatus(hours: OpeningHours, now: LocalTime): OpenStatus {
  const today = (hours[now.day] ?? []).map(([a, b]) => [toMinutes(a), toMinutes(b)] as const);
  const spill = (hours[previousDay(now.day)] ?? [])
    .map(([a, b]) => [toMinutes(a), toMinutes(b)] as const)
    .filter(([, end]) => end > 1440)
    .map(([, end]) => [0, end - 1440] as const);

  for (const [start, end] of [...spill, ...today]) {
    if (now.minutes >= start && now.minutes < end) {
      const allDay = start === 0 && end >= 1440;
      return { open: true, closesAt: allDay ? null : end };
    }
  }
  const later = today.map(([start]) => start).filter((s) => s > now.minutes);
  return { open: false, opensAt: later.length ? Math.min(...later) : null };
}

export function openLabel(status: OpenStatus): string {
  if (status.open) {
    return status.closesAt === null ? 'Mở cả ngày' : `Mở đến ${formatClock(status.closesAt)}`;
  }
  return status.opensAt === null ? 'Đã đóng cửa' : `Mở lúc ${formatClock(status.opensAt)}`;
}

/** `[["07:30","17:30"]]` → `"07:30 – 17:30"`; ngày nghỉ → `"Nghỉ"`. */
export function slotsLabel(slots: [string, string][]): string {
  if (slots.length === 0) return 'Nghỉ';
  return slots
    .map(([a, b]) =>
      a === '00:00' && b === '24:00'
        ? 'Cả ngày'
        : `${formatClock(toMinutes(a))} – ${formatClock(toMinutes(b))}`,
    )
    .join(', ');
}
