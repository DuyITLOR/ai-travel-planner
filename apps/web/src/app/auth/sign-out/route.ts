import { NextResponse, type NextRequest } from 'next/server';
import { isSupabaseConfigured } from '@/lib/supabase/config';
import { supabaseServer } from '@/lib/supabase/server';

/** Chỉ nhận POST (form), để link hay prefetch không vô tình đăng xuất người dùng. */
export async function POST(request: NextRequest) {
  if (isSupabaseConfigured) {
    const supabase = await supabaseServer();
    await supabase.auth.signOut();
  }
  const url = request.nextUrl.clone();
  url.pathname = '/';
  url.search = '';
  // 303 để trình duyệt chuyển POST thành GET.
  return NextResponse.redirect(url, { status: 303 });
}
