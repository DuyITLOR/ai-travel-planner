import { Be_Vietnam_Pro } from 'next/font/google';
import Link from 'next/link';
import type { ReactNode } from 'react';
import styles from './auth.module.css';

const font = Be_Vietnam_Pro({
  subsets: ['latin', 'vietnamese'],
  weight: ['400', '500', '600', '700'],
  display: 'swap',
});

const PROMISES = [
  'Không đến nơi đang đóng cửa',
  'Không chạy vòng giữa các quận',
  'Không vượt trần chi của bất kỳ ai trong nhóm',
];

export default function AuthLayout({ children }: { children: ReactNode }) {
  return (
    <div className={`${styles.shell} ${font.className}`}>
      <aside className={styles.aside}>
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
        <div className={styles.pitch}>
          <h2>Một ngày ở Sài Gòn, xếp xong trong lúc uống ly cà phê.</h2>
          <ul>
            {PROMISES.map((p) => (
              <li key={p}>
                <svg
                  aria-hidden="true"
                  width="22"
                  height="22"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2.5"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                >
                  <path d="m5 12.5 4.5 4.5L19 7.5" />
                </svg>
                {p}
              </li>
            ))}
          </ul>
        </div>
        <span className={styles.credit}>Bản đồ © OpenStreetMap · Thời tiết: Open-Meteo</span>
      </aside>
      <main className={styles.main}>
        <div className={styles.card}>{children}</div>
      </main>
    </div>
  );
}
