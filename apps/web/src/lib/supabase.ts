import { createClient, type SupabaseClient } from '@supabase/supabase-js';

// URL và anon key là công khai theo thiết kế; dữ liệu được bảo vệ bằng Row Level Security.
// Tuyệt đối không đưa service_role key hay DATABASE_URL vào web.
const url = process.env.NEXT_PUBLIC_SUPABASE_URL;
const anonKey = process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY;

export const isSupabaseConfigured = Boolean(url && anonKey);

let client: SupabaseClient | null = null;

/** Client dùng cho đăng nhập (Supabase Auth) và realtime. Gọi lazily, không gọi lúc build. */
export function supabase(): SupabaseClient {
  if (!url || !anonKey) {
    throw new Error('Supabase chưa cấu hình (xem apps/web/.env.example)');
  }
  client ??= createClient(url, anonKey);
  return client;
}
