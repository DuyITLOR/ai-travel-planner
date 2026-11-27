import type { Metadata } from 'next';
import Link from 'next/link';
import { filterPlaces, parseFilters, priceLabel } from '@/lib/places/filter';
import { openLabel, openStatus, saigonNow } from '@/lib/places/hours';
import { listPlaces } from '@/lib/places/queries';
import { CATEGORY_LABEL, PRICE_LEVEL_LABEL, type Category } from '@/lib/places/types';
import { isSupabaseConfigured } from '@/lib/supabase/config';
import styles from '../app.module.css';
import { CategoryIcon } from '../CategoryIcon';
import { PlaceMap } from './PlaceMap';

export const metadata: Metadata = { title: 'Khám phá · AI Travel Planner' };

const PAGE = 20;

type SearchParams = Promise<Record<string, string | string[] | undefined>>;

export default async function ExplorePage({ searchParams }: { searchParams: SearchParams }) {
  const params = await searchParams;

  if (!isSupabaseConfigured) {
    return (
      <main className={styles.main}>
        <h1 className={styles.title}>Khám phá Sài Gòn</h1>
        <p className={styles.empty}>Chưa cấu hình Supabase (thiếu biến NEXT_PUBLIC_SUPABASE_*).</p>
      </main>
    );
  }

  const filters = parseFilters(params);
  const limit = Math.min(Math.max(Number(params.limit) || PAGE, PAGE), 150);
  const now = saigonNow();

  const all = await listPlaces();
  const results = filterPlaces(all, filters, now);
  const shown = results.slice(0, limit);
  const districts = districtOptions(all.map((p) => p.district));
  const hasFilters =
    filters.q || filters.category || filters.district || filters.priceLevel || filters.openNow;

  const moreHref = () => {
    const qs = new URLSearchParams();
    for (const [k, v] of Object.entries(params)) {
      if (typeof v === 'string' && k !== 'limit') qs.set(k, v);
    }
    qs.set('limit', String(limit + PAGE));
    return `/explore?${qs.toString()}`;
  };

  return (
    <main className={styles.main}>
      <h1 className={styles.title}>Khám phá Sài Gòn</h1>

      <form method="get" action="/explore" className={styles.filters} role="search">
        <div className={styles.search}>
          <label htmlFor="explore-q" className={styles.sr}>
            Tìm địa điểm
          </label>
          <svg
            aria-hidden="true"
            width="20"
            height="20"
            viewBox="0 0 24 24"
            fill="none"
            stroke="#475467"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <circle cx="11" cy="11" r="7" />
            <path d="m20 20-3.5-3.5" />
          </svg>
          <input
            id="explore-q"
            name="q"
            type="search"
            defaultValue={filters.q}
            placeholder="Tìm quán ăn, bảo tàng, cà phê…"
            className={styles.searchInput}
          />
        </div>

        <label className={styles.toggle}>
          <input type="checkbox" name="open" value="1" defaultChecked={filters.openNow} />
          Đang mở
        </label>

        <label htmlFor="explore-category" className={styles.sr}>
          Loại
        </label>
        <select
          id="explore-category"
          name="category"
          defaultValue={filters.category}
          className={styles.select}
        >
          <option value="">Mọi loại</option>
          {(Object.keys(CATEGORY_LABEL) as Category[]).map((c) => (
            <option key={c} value={c}>
              {CATEGORY_LABEL[c]}
            </option>
          ))}
        </select>

        <label htmlFor="explore-district" className={styles.sr}>
          Quận
        </label>
        <select
          id="explore-district"
          name="district"
          defaultValue={filters.district}
          className={styles.select}
        >
          <option value="">Mọi quận</option>
          {districts.map((d) => (
            <option key={d} value={d}>
              {d}
            </option>
          ))}
        </select>

        <label htmlFor="explore-price" className={styles.sr}>
          Mức giá
        </label>
        <select
          id="explore-price"
          name="price"
          defaultValue={filters.priceLevel ?? ''}
          className={styles.select}
        >
          <option value="">Mọi mức giá</option>
          {[1, 2, 3, 4].map((l) => (
            <option key={l} value={l}>
              {PRICE_LEVEL_LABEL[l]}
            </option>
          ))}
        </select>

        <button type="submit" className={styles.primaryButton}>
          Lọc
        </button>
        {hasFilters && (
          <Link href="/explore" className={styles.ghostButton}>
            Xoá lọc
          </Link>
        )}
      </form>

      <div className={styles.split}>
        <section aria-label="Kết quả" className={styles.list}>
          <span className={styles.count} aria-live="polite">
            <strong>{results.length} điểm</strong>
            {filters.openNow ? ' đang mở lúc này' : ` trên tổng ${all.length}`}
            {results.length > 0 && ' · số trên sơ đồ khớp với danh sách'}
          </span>

          {results.length === 0 ? (
            <p className={styles.empty}>Không có điểm nào khớp bộ lọc. Thử bỏ bớt điều kiện.</p>
          ) : (
            <ol className={styles.list}>
              {shown.map((p, i) => {
                const status = openStatus(p.opening_hours, now);
                return (
                  <li key={p.id}>
                    <Link href={`/places/${p.id}`} className={styles.card}>
                      <span className={styles.pinNumber} aria-hidden="true">
                        {i + 1}
                      </span>
                      <span className={styles.thumb}>
                        <CategoryIcon category={p.category} size={28} />
                      </span>
                      <span className={styles.cardBody}>
                        <span className={styles.cardName}>{p.name}</span>
                        <span className={styles.cardMeta}>
                          {CATEGORY_LABEL[p.category]} · {p.district} ·{' '}
                          {p.indoor_outdoor === 'indoor' ? 'trong nhà' : 'ngoài trời'}
                        </span>
                        <span className={styles.row}>
                          <span
                            className={`${styles.badge} ${status.open ? '' : styles.badgeClosed}`}
                          >
                            {openLabel(status)}
                          </span>
                          <span className={styles.price}>{priceLabel(p)}</span>
                        </span>
                      </span>
                    </Link>
                  </li>
                );
              })}
            </ol>
          )}

          {results.length > shown.length && (
            <Link href={moreHref()} className={styles.ghostButton} scroll={false}>
              Xem thêm {Math.min(PAGE, results.length - shown.length)} điểm
            </Link>
          )}
        </section>

        <PlaceMap places={shown} />
      </div>
    </main>
  );
}

/** Quận nhiều điểm lên trước. */
function districtOptions(districts: string[]): string[] {
  const count = new Map<string, number>();
  for (const d of districts) count.set(d, (count.get(d) ?? 0) + 1);
  return [...count.entries()]
    .sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0], 'vi'))
    .map(([d]) => d);
}
