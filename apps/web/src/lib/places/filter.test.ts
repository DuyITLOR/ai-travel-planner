import { describe, expect, it } from 'vitest';
import { filterPlaces, fold, fullVnd, parseFilters, priceLabel, shortVnd } from './filter';
import type { OpeningHours, PlaceSummary } from './types';

const day = (slots: [string, string][]): OpeningHours => ({
  mon: slots,
  tue: slots,
  wed: slots,
  thu: slots,
  fri: slots,
  sat: slots,
  sun: slots,
});

const place = (over: Partial<PlaceSummary>): PlaceSummary => ({
  id: 'p001',
  name: 'Bảo tàng Chứng tích Chiến tranh',
  name_en: 'War Remnants Museum',
  category: 'museum',
  district: 'Quận 3',
  lat: 10.78,
  lon: 106.69,
  indoor_outdoor: 'indoor',
  ticket_price_vnd: 60000,
  price_level: 2,
  opening_hours: day([['07:30', '17:30']]),
  menu_items: [{ price_vnd: 60000 }],
  ...over,
});

const pho = place({
  id: 'p002',
  name: 'Phở Hoà Pasteur',
  name_en: null,
  category: 'restaurant',
  district: 'Quận 3',
  ticket_price_vnd: 0,
  price_level: 2,
  opening_hours: day([['06:00', '23:00']]),
  menu_items: [{ price_vnd: 0 }, { price_vnd: 95000 }, { price_vnd: 85000 }],
});

const all = [place({}), pho];
const noon = { day: 'mon' as const, minutes: 12 * 60 };
const none = parseFilters({});

describe('parseFilters', () => {
  it('bỏ qua giá trị lạ', () => {
    expect(parseFilters({ category: 'casino', price: '9', open: 'yes' })).toEqual(none);
  });

  it('đọc giá trị hợp lệ', () => {
    expect(parseFilters({ q: ' pho ', category: 'cafe', price: '2', open: '1' })).toMatchObject({
      q: 'pho',
      category: 'cafe',
      priceLevel: 2,
      openNow: true,
    });
  });
});

describe('filterPlaces', () => {
  it('tìm không phân biệt dấu', () => {
    expect(fold('Phở Hoà Đakao')).toBe('pho hoa dakao');
    expect(filterPlaces(all, { ...none, q: 'pho hoa' }, noon).map((p) => p.id)).toEqual(['p002']);
  });

  it('tìm cả tên tiếng Anh', () => {
    expect(filterPlaces(all, { ...none, q: 'war' }, noon).map((p) => p.id)).toEqual(['p001']);
  });

  it('lọc theo loại và đang mở', () => {
    expect(filterPlaces(all, { ...none, category: 'restaurant' }, noon)).toHaveLength(1);
    const night = { day: 'mon' as const, minutes: 20 * 60 };
    expect(filterPlaces(all, { ...none, openNow: true }, night).map((p) => p.id)).toEqual(['p002']);
  });
});

describe('giá', () => {
  it('ưu tiên giá vé, không thì món rẻ nhất có trả tiền', () => {
    expect(priceLabel(place({}))).toBe('Vé 60k');
    expect(priceLabel(pho)).toBe('Từ 85k');
    expect(priceLabel(place({ ticket_price_vnd: 0, menu_items: [{ price_vnd: 0 }] }))).toBe(
      'Miễn phí',
    );
  });

  it('định dạng tiền', () => {
    expect(shortVnd(1_500_000)).toBe('1,5tr');
    expect(shortVnd(45_500)).toBe('45,5k');
    expect(fullVnd(60000)).toBe('60.000đ');
    expect(fullVnd(0)).toBe('Miễn phí');
  });
});
