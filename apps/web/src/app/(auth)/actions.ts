'use server';

import { SignInInput, SignUpInput } from '@atp/shared';
import { headers } from 'next/headers';
import { redirect } from 'next/navigation';
import { authErrorMessage } from '@/lib/auth/errors';
import { AUTH_CALLBACK_PATH, safeNextPath } from '@/lib/auth/routes';
import { isSupabaseConfigured } from '@/lib/supabase/config';
import { supabaseServer } from '@/lib/supabase/server';

export type AuthFormState = {
  /** Lỗi chung, hiện phía trên form. */
  error?: string;
  /** Thông báo thành công (vd. đã gửi thư xác nhận). */
  notice?: string;
  fieldErrors?: { email?: string; password?: string };
  /** Giữ lại email đã nhập khi trả form về. */
  email?: string;
};

const NOT_CONFIGURED: AuthFormState = {
  error: 'Đăng nhập chưa được cấu hình trên máy chủ này.',
};

function readForm(formData: FormData) {
  return {
    email: String(formData.get('email') ?? ''),
    password: String(formData.get('password') ?? ''),
    next: safeNextPath(formData.get('next')),
  };
}

function firstFieldErrors(fe: Record<string, string[] | undefined>): AuthFormState['fieldErrors'] {
  return { email: fe.email?.[0], password: fe.password?.[0] };
}

/** Origin của web để Supabase gửi người dùng quay về (callback OAuth, link xác nhận email). */
async function siteOrigin(): Promise<string> {
  const fromEnv = process.env.NEXT_PUBLIC_SITE_URL;
  if (fromEnv) return fromEnv.replace(/\/+$/, '');
  const h = await headers();
  const host = h.get('x-forwarded-host') ?? h.get('host') ?? 'localhost:3000';
  const proto = h.get('x-forwarded-proto') ?? (host.startsWith('localhost') ? 'http' : 'https');
  return `${proto}://${host}`;
}

/** Ghi mã lỗi ra log server (không ghi email hay mật khẩu) để dò lỗi khi người dùng báo. */
function logAuthError<E extends { code?: string; status?: number; message: string }>(
  step: string,
  error: E,
): E {
  console.warn(`[auth] ${step} failed: code=${error.code} status=${error.status} ${error.message}`);
  return error;
}

function callbackUrl(origin: string, next: string): string {
  return `${origin}${AUTH_CALLBACK_PATH}?next=${encodeURIComponent(next)}`;
}

export async function signInWithPassword(
  _prev: AuthFormState,
  formData: FormData,
): Promise<AuthFormState> {
  if (!isSupabaseConfigured) return NOT_CONFIGURED;
  const { email, password, next } = readForm(formData);

  const parsed = SignInInput.safeParse({ email, password });
  if (!parsed.success) {
    return { email, fieldErrors: firstFieldErrors(parsed.error.flatten().fieldErrors) };
  }

  const supabase = await supabaseServer();
  const { error } = await supabase.auth.signInWithPassword(parsed.data);
  if (error) return { email, error: authErrorMessage(logAuthError('sign-in', error)) };

  redirect(next);
}

export async function signUpWithPassword(
  _prev: AuthFormState,
  formData: FormData,
): Promise<AuthFormState> {
  if (!isSupabaseConfigured) return NOT_CONFIGURED;
  const { email, password, next } = readForm(formData);

  const parsed = SignUpInput.safeParse({ email, password });
  if (!parsed.success) {
    return { email, fieldErrors: firstFieldErrors(parsed.error.flatten().fieldErrors) };
  }

  const supabase = await supabaseServer();
  const { data, error } = await supabase.auth.signUp({
    ...parsed.data,
    options: { emailRedirectTo: callbackUrl(await siteOrigin(), next) },
  });
  if (error) return { email, error: authErrorMessage(logAuthError('sign-up', error)) };

  // Project bật xác nhận email thì chưa có phiên: báo người dùng mở hộp thư.
  if (!data.session) {
    return {
      email,
      notice: `Đã gửi thư xác nhận tới ${parsed.data.email}. Mở thư và bấm link để hoàn tất đăng ký.`,
    };
  }
  redirect(next);
}

export async function signInWithGoogle(formData: FormData): Promise<void> {
  if (!isSupabaseConfigured) redirect('/sign-in?error=not_configured');
  const next = safeNextPath(formData.get('next'));

  const supabase = await supabaseServer();
  const { data, error } = await supabase.auth.signInWithOAuth({
    provider: 'google',
    options: { redirectTo: callbackUrl(await siteOrigin(), next) },
  });
  if (error || !data.url) {
    redirect(`/sign-in?error=${encodeURIComponent(error?.code ?? 'oauth_failed')}`);
  }
  redirect(data.url);
}
