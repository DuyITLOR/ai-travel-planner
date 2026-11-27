import Link from 'next/link';
import type { ReactNode } from 'react';
import { SIGN_IN_PATH } from '@/lib/auth/routes';
import { beVietnamPro } from '@/lib/fonts';
import { currentUser } from '@/lib/supabase/server';
import styles from './app.module.css';
import { NavLink } from './NavLink';

export default async function AppLayout({ children }: { children: ReactNode }) {
  const user = await currentUser();

  return (
    <div className={`${styles.shell} ${beVietnamPro.className}`}>
      <header className={styles.header}>
        <Link href="/" className={styles.brand}>
          <span className={styles.brandMark}>
            <svg
              aria-hidden="true"
              width="20"
              height="20"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <path d="M12 21s-7-6.2-7-11a7 7 0 0 1 14 0c0 4.8-7 11-7 11z" />
              <circle cx="12" cy="10" r="2.5" />
            </svg>
          </span>
          AI Travel Planner
        </Link>
        <nav aria-label="Điều hướng chính" className={styles.nav}>
          <NavLink href="/explore" match={['/places']} className={styles.navLink}>
            Khám phá
          </NavLink>
        </nav>
        {user ? (
          <form action="/auth/sign-out" method="post" className={styles.account}>
            <span>{user.email}</span>
            <button type="submit" className={styles.ghostButton}>
              Đăng xuất
            </button>
          </form>
        ) : (
          <Link href={SIGN_IN_PATH} className={styles.primaryButton}>
            Đăng nhập
          </Link>
        )}
      </header>
      {children}
    </div>
  );
}
