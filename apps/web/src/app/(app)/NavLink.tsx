'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import type { ReactNode } from 'react';

/** Link điều hướng, tự gắn `aria-current="page"` khi đang ở trang đó hoặc trang con. */
export function NavLink({
  href,
  match = [],
  className,
  children,
}: {
  href: string;
  /** Tiền tố khác cũng tính là "đang ở đây" (vd. /places thuộc mục Khám phá). */
  match?: string[];
  className?: string;
  children: ReactNode;
}) {
  const pathname = usePathname();
  const active = [href, ...match].some((p) => pathname === p || pathname.startsWith(`${p}/`));
  return (
    <Link href={href} className={className} aria-current={active ? 'page' : undefined}>
      {children}
    </Link>
  );
}
