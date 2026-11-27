'use client';

import Link from 'next/link';
import { useActionState } from 'react';
import { useFormStatus } from 'react-dom';
import { PASSWORD_MIN_LENGTH } from '@atp/shared';
import { SIGN_IN_PATH, SIGN_UP_PATH } from '@/lib/auth/routes';
import {
  type AuthFormState,
  signInWithGoogle,
  signInWithPassword,
  signUpWithPassword,
} from './actions';
import styles from './auth.module.css';

type Mode = 'sign-in' | 'sign-up';

const COPY: Record<
  Mode,
  { submit: string; pending: string; switchText: string; switchLink: string }
> = {
  'sign-in': {
    submit: 'Đăng nhập',
    pending: 'Đang đăng nhập…',
    switchText: 'Chưa có tài khoản?',
    switchLink: 'Đăng ký',
  },
  'sign-up': {
    submit: 'Tạo tài khoản',
    pending: 'Đang tạo tài khoản…',
    switchText: 'Đã có tài khoản?',
    switchLink: 'Đăng nhập',
  },
};

export function AuthForm({
  mode,
  next,
  initialError,
  disabled,
}: {
  mode: Mode;
  /** Trang quay về sau khi đăng nhập (đã lọc ở server). */
  next: string;
  /** Lỗi từ query string, vd. người dùng huỷ đăng nhập Google. */
  initialError?: string;
  /** Supabase chưa cấu hình. */
  disabled?: boolean;
}) {
  const action = mode === 'sign-in' ? signInWithPassword : signUpWithPassword;
  const [state, formAction] = useActionState<AuthFormState, FormData>(action, {
    error: initialError,
  });
  const copy = COPY[mode];
  const nextQuery = next === '/' ? '' : `?next=${encodeURIComponent(next)}`;
  const switchHref = (mode === 'sign-in' ? SIGN_UP_PATH : SIGN_IN_PATH) + nextQuery;
  const fe = state.fieldErrors ?? {};

  return (
    <>
      {state.error && (
        <p role="alert" className={`${styles.alert} ${styles.alertError}`}>
          {state.error}
        </p>
      )}
      {state.notice && (
        <p role="status" className={`${styles.alert} ${styles.alertNotice}`}>
          {state.notice}
        </p>
      )}

      <form action={signInWithGoogle}>
        <input type="hidden" name="next" value={next} />
        <GoogleButton disabled={disabled} />
      </form>

      <div className={styles.divider}>hoặc dùng email</div>

      <form action={formAction} className={styles.form} noValidate>
        <input type="hidden" name="next" value={next} />
        <div className={styles.field}>
          <label htmlFor="auth-email" className={styles.label}>
            Email
          </label>
          <input
            id="auth-email"
            name="email"
            type="email"
            autoComplete="email"
            inputMode="email"
            required
            placeholder="ban@student.hcmus.edu.vn"
            defaultValue={state.email}
            key={state.email}
            aria-invalid={fe.email ? true : undefined}
            aria-describedby={fe.email ? 'auth-email-error' : undefined}
            className={styles.input}
          />
          {fe.email && (
            <p id="auth-email-error" className={styles.fieldError}>
              {fe.email}
            </p>
          )}
        </div>

        <div className={styles.field}>
          <label htmlFor="auth-password" className={styles.label}>
            Mật khẩu
          </label>
          <input
            id="auth-password"
            name="password"
            type="password"
            autoComplete={mode === 'sign-in' ? 'current-password' : 'new-password'}
            required
            minLength={mode === 'sign-up' ? PASSWORD_MIN_LENGTH : undefined}
            placeholder={`Ít nhất ${PASSWORD_MIN_LENGTH} ký tự`}
            aria-invalid={fe.password ? true : undefined}
            aria-describedby={fe.password ? 'auth-password-error' : undefined}
            className={styles.input}
          />
          {fe.password && (
            <p id="auth-password-error" className={styles.fieldError}>
              {fe.password}
            </p>
          )}
        </div>

        <SubmitButton label={copy.submit} pendingLabel={copy.pending} disabled={disabled} />
      </form>

      <p className={styles.switch}>
        {copy.switchText}{' '}
        <Link href={switchHref} className={styles.link}>
          {copy.switchLink}
        </Link>
      </p>

      <div className={styles.privacy}>
        <svg
          aria-hidden="true"
          width="20"
          height="20"
          viewBox="0 0 24 24"
          fill="none"
          stroke="#0E7C66"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
        >
          <rect x="5" y="11" width="14" height="9" rx="2" />
          <path d="M8 11V8a4 4 0 0 1 8 0v3" />
        </svg>
        <p>
          Tên, email và ảnh của bạn không bao giờ được gửi cho mô hình AI. AI chỉ thấy sở thích và
          mã địa điểm.
        </p>
      </div>
    </>
  );
}

function SubmitButton({
  label,
  pendingLabel,
  disabled,
}: {
  label: string;
  pendingLabel: string;
  disabled?: boolean;
}) {
  const { pending } = useFormStatus();
  return (
    <button
      type="submit"
      className={`${styles.button} ${styles.primary}`}
      disabled={disabled || pending}
    >
      {pending ? pendingLabel : label}
    </button>
  );
}

function GoogleButton({ disabled }: { disabled?: boolean }) {
  const { pending } = useFormStatus();
  return (
    <button
      type="submit"
      className={`${styles.button} ${styles.secondary}`}
      disabled={disabled || pending}
    >
      <svg aria-hidden="true" width="20" height="20" viewBox="0 0 48 48">
        <path
          fill="#FFC107"
          d="M43.6 20.5H42V20H24v8h11.3C33.7 32.7 29.2 36 24 36c-6.6 0-12-5.4-12-12s5.4-12 12-12c3.1 0 5.8 1.2 7.9 3.1l5.7-5.7C34 6.1 29.3 4 24 4 12.9 4 4 12.9 4 24s8.9 20 20 20 20-8.9 20-20c0-1.3-.1-2.4-.4-3.5z"
        />
        <path
          fill="#FF3D00"
          d="m6.3 14.7 6.6 4.8C14.7 15.1 19 12 24 12c3.1 0 5.8 1.2 7.9 3.1l5.7-5.7C34 6.1 29.3 4 24 4 16.3 4 9.7 8.3 6.3 14.7z"
        />
        <path
          fill="#4CAF50"
          d="M24 44c5.2 0 9.9-2 13.4-5.2l-6.2-5.2C29.2 35.1 26.7 36 24 36c-5.2 0-9.6-3.3-11.3-7.9l-6.5 5C9.5 39.6 16.2 44 24 44z"
        />
        <path
          fill="#1976D2"
          d="M43.6 20.5H42V20H24v8h11.3c-.8 2.2-2.2 4.2-4.1 5.6l6.2 5.2C37 39.2 44 34 44 24c0-1.3-.1-2.4-.4-3.5z"
        />
      </svg>
      {pending ? 'Đang chuyển sang Google…' : 'Tiếp tục với Google'}
    </button>
  );
}
