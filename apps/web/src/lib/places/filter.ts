import { openStatus, type LocalTime } from './hours';
import { CATEGORY_LABEL, type Category, type PlaceSummary } from './types';

export type PlaceFilters = {
  q: string;
  category: Category | '';
  district: string;
  priceLevel: number | null;
  openNow: boolean;
};

/** Đọc bộ lọc từ query string; giá trị lạ bị bỏ qua thay vì báo lỗi. */
export function parseFilters(params: Record<string, string | string[] | undefined>): PlaceFilters {
  const one = (k: string) => {
    const v = params[k];
    return (Array.isArray(v) ? v[0] : v)?.trim() ?? '';
  };
  const category = one('category');
  const price = Number(one('price'));
  return {
    q: one('q').slice(0, 100),
    category: category in CATEGORY_LABEL ? (category as Category) : '',
    district: one('district').slice(0, 60),
    priceLevel: Number.isInteger(price) && price >= 1 && price <= 4 ? price : null,
    openNow: one('open') === '1',
  };
}

/** Bỏ dấu tiếng Việt, chữ thường: "Phở Hoà" → "pho hoa". */
export function fold(text: string): string {
  return text
    .normalize('NFD')
    .replace(/[̀-ͯ]/g, '')
    .replace(/đ/g, 'd')
    .replace(/Đ/g, 'D')
    .toLowerCase();
}

export function filterPlaces(
  places: PlaceSummary[],
  f: PlaceFilters,
  now: LocalTime,
): PlaceSummary[] {
  const words = fold(f.q).split(/\s+/).filter(Boolean);
  return places.filter((p) => {
    if (f.category && p.category !== f.category) return false;
    if (f.district && p.district !== f.district) return false;
    if (f.priceLevel !== null && p.price_level !== f.priceLevel) return false;
    if (words.length) {
      const hay = fold(`${p.name} ${p.name_en ?? ''} ${p.district}`);
      if (!words.every((w) => hay.includes(w))) return false;
    }
    if (f.openNow && !openStatus(p.opening_hours, now).open) return false;
    return true;
  });
}

/** Giá hiển thị trong danh sách: vé nếu có, không thì món rẻ nhất. */
export function priceLabel(p: Pick<PlaceSummary, 'ticket_price_vnd' | 'menu_items'>): string {
  if (p.ticket_price_vnd > 0) return `Vé ${shortVnd(p.ticket_price_vnd)}`;
  const paid = p.menu_items.map((m) => m.price_vnd).filter((v) => v > 0);
  return paid.length ? `Từ ${shortVnd(Math.min(...paid))}` : 'Miễn phí';
}

/** 60000 → "60k", 1500000 → "1,5tr", 0 → "0đ". */
export function shortVnd(vnd: number): string {
  if (vnd >= 1_000_000) {
    return `${(vnd / 1_000_000).toLocaleString('vi-VN', { maximumFractionDigits: 1 })}tr`;
  }
  if (vnd >= 1000) return `${(vnd / 1000).toLocaleString('vi-VN', { maximumFractionDigits: 1 })}k`;
  return `${vnd}đ`;
}

/** 60000 → "60.000đ", 0 → "Miễn phí". */
export function fullVnd(vnd: number): string {
  return vnd === 0 ? 'Miễn phí' : `${vnd.toLocaleString('vi-VN')}đ`;
}
