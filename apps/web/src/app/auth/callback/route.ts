import { NextResponse, type NextRequest } from 'next/server';
import { SIGN_IN_PATH, safeNextPath } from '@/lib/auth/routes';
import { isSupabaseConfigured } from '@/lib/supabase/config';
import { supabaseServer } from '@/lib/supabase/server';

/**
 * Supabase chuyển người dùng về đây sau khi đăng nhập Google hoặc bấm link xác nhận email,
 * kèm `code` (PKCE). Đổi code lấy phiên, ghi cookie, rồi về trang `next`.
 */
export async function GET(request: NextRequest) {
  const { searchParams } = request.nextUrl;
  const next = safeNextPath(searchParams.get('next'));
  const code = searchParams.get('code');

  const fail = (reason: string) => {
    const url = request.nextUrl.clone();
    url.pathname = SIGN_IN_PATH;
    url.search = new URLSearchParams({ error: reason }).toString();
    return NextResponse.redirect(url);
  };

  // Người dùng huỷ ở màn Google, hoặc link hết hạn.
  if (searchParams.get('error')) return fail(searchParams.get('error_code') ?? 'oauth_failed');
  if (!code || !isSupabaseConfigured) return fail('oauth_failed');

  const supabase = await supabaseServer();
  const { error } = await supabase.auth.exchangeCodeForSession(code);
  if (error) return fail(error.code ?? 'oauth_failed');

  const url = request.nextUrl.clone();
  const [pathname, query = ''] = next.split('?');
  url.pathname = pathname ?? '/';
  url.search = query;
  return NextResponse.redirect(url);
}
