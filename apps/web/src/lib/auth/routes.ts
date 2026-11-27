export const SIGN_IN_PATH = '/sign-in';
export const SIGN_UP_PATH = '/sign-up';
export const AUTH_CALLBACK_PATH = '/auth/callback';
export const DEFAULT_AFTER_SIGN_IN = '/';

/** Route cần đăng nhập. Thêm tiền tố khi làm trang mới (lịch trình, nhóm, sổ địa điểm...). */
export const PROTECTED_PREFIXES = ['/trips', '/groups', '/notebook', '/profile'] as const;

export function isProtectedPath(pathname: string): boolean {
  return PROTECTED_PREFIXES.some((p) => pathname === p || pathname.startsWith(`${p}/`));
}

/** Trang đăng nhập/đăng ký: đã đăng nhập thì không cần vào nữa. */
export function isAuthPage(pathname: string): boolean {
  return pathname === SIGN_IN_PATH || pathname === SIGN_UP_PATH;
}

/**
 * Chỉ cho phép quay về đường dẫn nội bộ sau khi đăng nhập, chặn open redirect
 * (`//evil.com`, `https://evil.com`, `/\evil.com`).
 */
export function safeNextPath(next: unknown): string {
  if (typeof next !== 'string' || next.length === 0) return DEFAULT_AFTER_SIGN_IN;
  if (!next.startsWith('/') || next.startsWith('//') || next.startsWith('/\\')) {
    return DEFAULT_AFTER_SIGN_IN;
  }
  // Ký tự điều khiển (xuống dòng, tab...) có thể làm trình duyệt hiểu sai URL.
  if ([...next].some((ch) => ch.charCodeAt(0) < 0x20)) return DEFAULT_AFTER_SIGN_IN;
  if (isAuthPage(next.split(/[?#]/)[0] ?? '')) return DEFAULT_AFTER_SIGN_IN;
  return next;
}
