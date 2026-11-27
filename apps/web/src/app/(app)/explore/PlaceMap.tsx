import Link from 'next/link';
import { layoutPoints } from '@/lib/places/map';
import type { PlaceSummary } from '@/lib/places/types';
import styles from '../app.module.css';

/**
 * Sơ đồ vị trí tương đối: ghim đặt theo toạ độ, khung tự co theo các điểm đang hiện.
 * Chưa phải bản đồ nền thật (MapLibre/OSM sẽ thay sau), nhưng số ghim khớp với danh sách.
 */
export function PlaceMap({ places }: { places: PlaceSummary[] }) {
  if (places.length === 0) {
    return (
      <section aria-label="Sơ đồ vị trí" className={styles.map}>
        <span className={styles.mapNote}>Không có điểm nào để hiện</span>
      </section>
    );
  }

  const positions = layoutPoints(places);
  const outsideCount = positions.filter((p) => p.outside).length;

  return (
    <section aria-label="Sơ đồ vị trí" className={styles.map}>
      {places.map((p, i) => {
        const pos = positions[i] ?? { x: 0.5, y: 0.5, outside: false };
        return (
          <Link
            key={p.id}
            href={`/places/${p.id}`}
            className={`${styles.mapPin} ${pos.outside ? styles.mapPinFar : ''}`}
            // Chừa lề 6% để ghim ở mép không bị cắt.
            style={{ left: `${6 + pos.x * 88}%`, top: `${6 + pos.y * 88}%` }}
            aria-label={`${i + 1}. ${p.name}${pos.outside ? ' (ở xa, ngoài khung)' : ''}`}
            title={pos.outside ? `${p.name} · ở xa, ngoài khung` : p.name}
          >
            {i + 1}
          </Link>
        );
      })}
      <span className={styles.mapNote}>
        Sơ đồ vị trí tương đối
        {outsideCount > 0 && ` · ${outsideCount} điểm ở xa nằm ở mép (viền đứt)`} · toạ độ ©
        OpenStreetMap
      </span>
    </section>
  );
}
