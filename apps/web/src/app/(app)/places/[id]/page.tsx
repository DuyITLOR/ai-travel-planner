import type { Metadata } from 'next';
import Link from 'next/link';
import { notFound } from 'next/navigation';
import { fullVnd } from '@/lib/places/filter';
import { dayLabel, openLabel, openStatus, saigonNow, slotsLabel } from '@/lib/places/hours';
import { getPlace, nearbyPlaces } from '@/lib/places/queries';
import { CATEGORY_LABEL, DAYS, PRICE_LEVEL_LABEL, type MenuItem } from '@/lib/places/types';
import { isSupabaseConfigured } from '@/lib/supabase/config';
import styles from '../../app.module.css';
import { CategoryIcon } from '../../CategoryIcon';

type Params = Promise<{ id: string }>;

export async function generateMetadata({ params }: { params: Params }): Promise<Metadata> {
  const { id } = await params;
  const place = isSupabaseConfigured ? await getPlace(id) : null;
  return { title: place ? `${place.name} · AI Travel Planner` : 'Địa điểm · AI Travel Planner' };
}

const KIND_LABEL: Record<MenuItem['kind'], string> = {
  ticket: 'Vé',
  food: 'Món ăn',
  drink: 'Đồ uống',
  service: 'Dịch vụ',
};

const CONFIDENCE_LABEL = { high: 'cao', medium: 'trung bình', low: 'thấp' } as const;

const COVERS_LABEL: Record<string, string> = {
  hours: 'giờ mở cửa',
  prices: 'giá',
  menu: 'menu',
  address: 'địa chỉ',
  status: 'tình trạng',
};

export default async function PlacePage({ params }: { params: Params }) {
  const { id } = await params;
  if (!isSupabaseConfigured) notFound();

  const [place, nearby] = await Promise.all([getPlace(id), nearbyPlaces(id)]);
  if (!place) notFound();

  const now = saigonNow();
  const status = openStatus(place.opening_hours, now);
  const checked = new Date(`${place.checked_at}T00:00:00`).toLocaleDateString('vi-VN');
  const osm = `https://www.openstreetmap.org/?mlat=${place.lat}&mlon=${place.lon}#map=17/${place.lat}/${place.lon}`;
  const credit = place.images[0];

  return (
    <main className={`${styles.main} ${styles.narrow}`}>
      <nav aria-label="Đường dẫn" className={styles.breadcrumb}>
        <Link href="/explore">Khám phá</Link>
        <span aria-hidden="true">/</span>
        <Link href={`/explore?district=${encodeURIComponent(place.district)}`}>
          {place.district}
        </Link>
        <span aria-hidden="true">/</span>
        <span style={{ color: 'var(--ink)' }}>{place.name}</span>
      </nav>

      <div className={styles.hero}>
        <CategoryIcon category={place.category} size={44} />
        {place.category_vi}
        <span className={styles.small}>
          {credit
            ? `Ảnh đang được bổ sung (${place.images.length} ảnh giấy phép mở)`
            : 'Chưa có ảnh'}
        </span>
      </div>

      <div className={styles.detail}>
        <div className={styles.detailMain}>
          <div className={styles.section} style={{ gap: 12 }}>
            <h1 className={styles.h1}>{place.name}</h1>
            <span className={styles.subtitle}>
              {CATEGORY_LABEL[place.category]} · {place.address}
              {place.name_en && <> · {place.name_en}</>}
            </span>
            <div className={styles.row}>
              <span className={`${styles.badge} ${status.open ? '' : styles.badgeClosed}`}>
                {status.open ? 'Đang mở · ' : ''}
                {openLabel(status)}
              </span>
              <span className={`${styles.badge} ${styles.badgeOutline}`}>
                {place.indoor_outdoor === 'indoor' ? 'Trong nhà' : 'Ngoài trời'}
              </span>
              <span className={`${styles.badge} ${styles.badgeOutline}`}>
                {PRICE_LEVEL_LABEL[place.price_level]} / người
              </span>
              <span className={`${styles.badge} ${styles.badgeOutline}`}>
                Khoảng {place.typical_duration_min} phút
              </span>
            </div>
          </div>

          <section className={styles.section}>
            <h2 className={styles.h2}>Giới thiệu</h2>
            <p className={styles.prose}>{place.description}</p>
            {place.interests.length > 0 && (
              <div className={styles.chips}>
                {place.interests.map((t) => (
                  <span key={t} className={`${styles.badge} ${styles.badgeOutline}`}>
                    #{t}
                  </span>
                ))}
              </div>
            )}
          </section>

          <section className={styles.section}>
            <div className={styles.sectionHead}>
              <h2 className={styles.h2}>Vé và giá</h2>
              <span className={styles.small}>
                Kiểm tra {checked} · giá ước tính để lên kế hoạch
              </span>
            </div>
            {place.menu_items.length === 0 ? (
              <p className={styles.small}>Chưa có giá.</p>
            ) : (
              <table className={styles.table}>
                <thead>
                  <tr>
                    <th scope="col">Mục</th>
                    <th scope="col" className={styles.kind}>
                      Loại
                    </th>
                    <th scope="col" className={styles.right}>
                      Giá
                    </th>
                  </tr>
                </thead>
                <tbody>
                  {place.menu_items.map((m) => (
                    <tr key={m.item_id}>
                      <td>{m.name}</td>
                      <td className={styles.kind}>{KIND_LABEL[m.kind]}</td>
                      <td className={styles.right}>{fullVnd(m.price_vnd)}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
            {place.notes && <p className={styles.small}>Ghi chú: {place.notes}</p>}
          </section>

          <section className={styles.section}>
            <div className={styles.sectionHead}>
              <h2 className={styles.h2}>Nguồn</h2>
              <span className={styles.small}>
                Độ tin cậy {CONFIDENCE_LABEL[place.verification.confidence]}
                {place.verification.human_checked ? ' · đã kiểm tra tay' : ''}
              </span>
            </div>
            <ul className={styles.sources}>
              {place.sources.map((s) => (
                <li key={s.url}>
                  <a href={s.url} target="_blank" rel="noopener noreferrer">
                    {new URL(s.url).hostname.replace(/^www\./, '')}
                  </a>{' '}
                  · {s.covers.map((c) => COVERS_LABEL[c] ?? c).join(', ')}
                </li>
              ))}
            </ul>
          </section>
        </div>

        <aside className={styles.detailAside}>
          <div className={styles.panel}>
            <div className={styles.sectionHead}>
              <h2 className={styles.h3}>Giờ mở cửa</h2>
              <span className={styles.small}>Kiểm tra {checked}</span>
            </div>
            <ul className={styles.hoursList}>
              {DAYS.map((d) => (
                <li
                  key={d}
                  className={`${styles.hoursRow} ${d === now.day ? styles.hoursToday : ''}`}
                >
                  <span>
                    {d === now.day ? 'Hôm nay · ' : ''}
                    {dayLabel(d)}
                  </span>
                  <span>{slotsLabel(place.opening_hours[d])}</span>
                </li>
              ))}
            </ul>
            {place.opening_hours_note && (
              <span className={styles.small}>{place.opening_hours_note}</span>
            )}
          </div>

          <div className={styles.panel}>
            <h2 className={styles.h3}>Vị trí</h2>
            <div className={styles.miniMap}>
              <span className={styles.miniPin} />
              <span className={styles.mapNote}>© OpenStreetMap</span>
            </div>
            <span className={styles.small}>{place.address}</span>
            {place.coord_precision === 'street' && (
              <span className={styles.small}>Toạ độ chỉ chính xác tới tên đường.</span>
            )}
            <a href={osm} target="_blank" rel="noopener noreferrer" className={styles.ghostButton}>
              Mở trên OpenStreetMap
            </a>
          </div>

          {nearby.length > 0 && (
            <div className={styles.panel}>
              <h2 className={styles.h3}>Gần đây (xe máy)</h2>
              <ul className={styles.nearby}>
                {nearby.map((n) => (
                  <li key={n.place.id}>
                    <Link href={`/places/${n.place.id}`}>
                      <span>{n.place.name}</span>
                      <span className={styles.muted}>
                        {n.minutes} phút · {(n.distance_m / 1000).toFixed(1).replace('.', ',')} km
                      </span>
                    </Link>
                  </li>
                ))}
              </ul>
            </div>
          )}
        </aside>
      </div>
    </main>
  );
}
