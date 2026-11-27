// URL và anon key là công khai theo thiết kế; dữ liệu được bảo vệ bằng Row Level Security.
// Tuyệt đối không đưa service_role key hay DATABASE_URL vào web.
export const SUPABASE_URL = process.env.NEXT_PUBLIC_SUPABASE_URL ?? '';
export const SUPABASE_ANON_KEY = process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY ?? '';

export const isSupabaseConfigured = Boolean(SUPABASE_URL && SUPABASE_ANON_KEY);

export function assertSupabaseConfigured(): void {
  if (!isSupabaseConfigured) {
    throw new Error('Supabase chưa cấu hình (xem apps/web/.env.example)');
  }
}
