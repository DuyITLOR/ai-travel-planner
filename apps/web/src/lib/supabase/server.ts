import { createServerClient } from '@supabase/ssr';
import type { SupabaseClient, User } from '@supabase/supabase-js';
import { cookies } from 'next/headers';
import {
  SUPABASE_ANON_KEY,
  SUPABASE_URL,
  assertSupabaseConfigured,
  isSupabaseConfigured,
} from './config';

/**
 * Client cho Server Component, Server Action và Route Handler.
 * Tạo mới cho mỗi request, không dùng lại giữa các request.
 */
export async function supabaseServer(): Promise<SupabaseClient> {
  assertSupabaseConfigured();
  const cookieStore = await cookies();

  return createServerClient(SUPABASE_URL, SUPABASE_ANON_KEY, {
    cookies: {
      getAll: () => cookieStore.getAll(),
      setAll(cookiesToSet) {
        try {
          for (const { name, value, options } of cookiesToSet) {
            cookieStore.set(name, value, options);
          }
        } catch {
          // Server Component không ghi được cookie; middleware đã làm mới phiên rồi.
        }
      },
    },
  });
}

/** Người dùng hiện tại (đã xác thực với Supabase Auth), hoặc null. */
export async function currentUser(): Promise<User | null> {
  if (!isSupabaseConfigured) return null;
  const supabase = await supabaseServer();
  const { data } = await supabase.auth.getUser();
  return data.user;
}
