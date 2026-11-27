import type { Metadata } from 'next';
import { authErrorMessage } from '@/lib/auth/errors';
import { safeNextPath } from '@/lib/auth/routes';
import { isSupabaseConfigured } from '@/lib/supabase/config';
import { AuthForm } from '../AuthForm';
import styles from '../auth.module.css';

export const metadata: Metadata = { title: 'Đăng nhập · AI Travel Planner' };

type SearchParams = Promise<{ next?: string; error?: string }>;

export default async function SignInPage({ searchParams }: { searchParams: SearchParams }) {
  const { next, error } = await searchParams;

  return (
    <>
      <div className={styles.heading}>
        <h1>Đăng nhập</h1>
        <p>Lịch trình, nhóm và sổ địa điểm đồng bộ giữa web và điện thoại.</p>
      </div>
      <AuthForm
        mode="sign-in"
        next={safeNextPath(next)}
        initialError={initialError(error)}
        disabled={!isSupabaseConfigured}
      />
    </>
  );
}

function initialError(code: string | undefined): string | undefined {
  if (!isSupabaseConfigured)
    return 'Đăng nhập chưa được cấu hình (thiếu biến NEXT_PUBLIC_SUPABASE_*).';
  if (!code) return undefined;
  if (code === 'access_denied') return 'Bạn đã huỷ đăng nhập bằng Google.';
  return authErrorMessage({ code });
}
