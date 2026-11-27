/** Khớp cột của bảng `places` (supabase/migrations/*_create_places.sql). */

export const DAYS = ['mon', 'tue', 'wed', 'thu', 'fri', 'sat', 'sun'] as const;
export type Day = (typeof DAYS)[number];

/** `["HH:MM","HH:MM"]`; giờ đóng sau nửa đêm ghi quá 24 (`"26:00"` = 02:00 hôm sau). */
export type Slot = [string, string];
export type OpeningHours = Record<Day, Slot[]>;

export type Category =
  | 'museum'
  | 'landmark'
  | 'religious_site'
  | 'park_nature'
  | 'entertainment'
  | 'art_culture'
  | 'shopping_mall'
  | 'market'
  | 'street_food'
  | 'restaurant'
  | 'cafe'
  | 'nightlife';

export const CATEGORY_LABEL: Record<Category, string> = {
  museum: 'Bảo tàng',
  landmark: 'Địa danh & công trình',
  religious_site: 'Chùa, nhà thờ, đền',
  park_nature: 'Công viên & thiên nhiên',
  entertainment: 'Vui chơi giải trí',
  art_culture: 'Nghệ thuật & show',
  shopping_mall: 'Trung tâm thương mại',
  market: 'Chợ',
  street_food: 'Ăn vặt & quán bình dân',
  restaurant: 'Nhà hàng',
  cafe: 'Cà phê & trà',
  nightlife: 'Bar & về đêm',
};

export const PRICE_LEVEL_LABEL: Record<number, string> = {
  1: 'Dưới 50k',
  2: '50k – 150k',
  3: '150k – 400k',
  4: 'Trên 400k',
};

/** Một dòng trong danh sách Khám phá. */
export type PlaceSummary = {
  id: string;
  name: string;
  name_en: string | null;
  category: Category;
  district: string;
  lat: number;
  lon: number;
  indoor_outdoor: 'indoor' | 'outdoor';
  ticket_price_vnd: number;
  price_level: number;
  opening_hours: OpeningHours;
  menu_items: { price_vnd: number }[];
};

export type MenuItem = {
  item_id: string;
  name: string;
  kind: 'ticket' | 'food' | 'drink' | 'service';
  price_vnd: number;
};

export type PlaceDetail = Omit<PlaceSummary, 'menu_items'> & {
  category_vi: string;
  address: string;
  coord_precision: 'poi' | 'street';
  description: string;
  typical_duration_min: number;
  opening_hours_note: string | null;
  interests: string[];
  images: { file: string; author: string; license: string; license_url: string; page: string }[];
  sources: { url: string; covers: string[] }[];
  checked_at: string;
  verification: { status: string; confidence: 'high' | 'medium' | 'low'; human_checked: boolean };
  notes: string | null;
  menu_items: MenuItem[];
};

export type Nearby = {
  minutes: number;
  distance_m: number;
  place: Pick<PlaceSummary, 'id' | 'name' | 'category' | 'district'>;
};
