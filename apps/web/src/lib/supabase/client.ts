'use client';

import { createBrowserClient } from '@supabase/ssr';
import type { SupabaseClient } from '@supabase/supabase-js';
import { SUPABASE_ANON_KEY, SUPABASE_URL, assertSupabaseConfigured } from './config';

let client: SupabaseClient | null = null;

/**
 * Client cho Client Component (realtime, đọc bảng có RLS).
 * Phiên đăng nhập nằm trong cookie nên dùng chung với server.
 */
export function supabaseBrowser(): SupabaseClient {
  assertSupabaseConfigured();
  client ??= createBrowserClient(SUPABASE_URL, SUPABASE_ANON_KEY);
  return client;
}
