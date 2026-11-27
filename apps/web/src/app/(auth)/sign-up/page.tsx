import type { Metadata } from 'next';
import { safeNextPath } from '@/lib/auth/routes';
import { isSupabaseConfigured } from '@/lib/supabase/config';
import { AuthForm } from '../AuthForm';
import styles from '../auth.module.css';

export const metadata: Metadata = { title: 'Đăng ký · AI Travel Planner' };

type SearchParams = Promise<{ next?: string }>;

export default async function SignUpPage({ searchParams }: { searchParams: SearchParams }) {
  const { next } = await searchParams;

  return (
    <>
      <div className={styles.heading}>
        <h1>Tạo tài khoản</h1>
        <p>Lưu lịch trình, rủ bạn vào nhóm và giữ sổ địa điểm của riêng bạn.</p>
      </div>
      <AuthForm
        mode="sign-up"
        next={safeNextPath(next)}
        initialError={
          isSupabaseConfigured
            ? undefined
            : 'Đăng ký chưa được cấu hình (thiếu biến NEXT_PUBLIC_SUPABASE_*).'
        }
        disabled={!isSupabaseConfigured}
      />
    </>
  );
}
