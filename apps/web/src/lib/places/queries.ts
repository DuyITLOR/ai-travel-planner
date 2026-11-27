import { supabaseServer } from '@/lib/supabase/server';
import type { Nearby, PlaceDetail, PlaceSummary } from './types';

// Bảng chỉ đọc (RLS cho phép SELECT), gọi bằng anon key qua client server-side.

const SUMMARY_COLUMNS =
  'id, name, name_en, category, district, lat, lon, indoor_outdoor, ticket_price_vnd, price_level, opening_hours, menu_items(price_vnd)';

export async function listPlaces(): Promise<PlaceSummary[]> {
  const supabase = await supabaseServer();
  const { data, error } = await supabase.from('places').select(SUMMARY_COLUMNS).order('id');
  if (error) throw new Error(`Không đọc được danh sách địa điểm: ${error.message}`);
  return data as unknown as PlaceSummary[];
}

export async function getPlace(id: string): Promise<PlaceDetail | null> {
  if (!/^p\d{3}$/.test(id)) return null;
  const supabase = await supabaseServer();
  const { data, error } = await supabase
    .from('places')
    .select('*, menu_items(item_id, name, kind, price_vnd)')
    .eq('id', id)
    .order('item_id', { referencedTable: 'menu_items' })
    .maybeSingle();
  if (error) throw new Error(`Không đọc được địa điểm ${id}: ${error.message}`);
  return data as unknown as PlaceDetail | null;
}

/** Các điểm gần nhất theo thời gian đi xe máy. */
export async function nearbyPlaces(id: string, limit = 5): Promise<Nearby[]> {
  const supabase = await supabaseServer();
  const { data, error } = await supabase
    .from('travel_times')
    .select(
      'minutes, distance_m, place:places!travel_times_to_place_id_fkey(id, name, category, district)',
    )
    .eq('from_place_id', id)
    .neq('to_place_id', id)
    .order('minutes')
    .order('distance_m')
    .limit(limit);
  if (error) throw new Error(`Không đọc được điểm gần ${id}: ${error.message}`);
  return data as unknown as Nearby[];
}
