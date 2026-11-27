export type Point = { lat: number; lon: number };
export type MapPosition = { x: number; y: number; outside: boolean };

function quantile(sorted: number[], q: number): number {
  const i = (sorted.length - 1) * q;
  const lo = Math.floor(i);
  const hi = Math.ceil(i);
  return (sorted[lo] ?? 0) + ((sorted[hi] ?? 0) - (sorted[lo] ?? 0)) * (i - lo);
}

/** Khoảng [min, max] theo phân vị 10–90, nới ra 60% mỗi bên nhưng không quá giá trị thật. */
function robustRange(values: number[]): [number, number] {
  const s = [...values].sort((a, b) => a - b);
  const lo = quantile(s, 0.1);
  const hi = quantile(s, 0.9);
  const pad = (hi - lo) * 0.6 || 0.005;
  return [Math.max(s[0] ?? lo, lo - pad), Math.min(s[s.length - 1] ?? hi, hi + pad)];
}

/**
 * Đặt các điểm vào khung 0..1 (x sang phải, y xuống dưới). Khung bám theo vùng tập trung
 * để vài điểm ở xa (Củ Chi, Cần Giờ) không dồn phần còn lại thành một cục; điểm nằm ngoài
 * khung bị kẹp vào mép và đánh dấu `outside`.
 */
export function layoutPoints(points: Point[]): MapPosition[] {
  if (points.length === 0) return [];
  if (points.length === 1) return [{ x: 0.5, y: 0.5, outside: false }];

  const [minLat, maxLat] = robustRange(points.map((p) => p.lat));
  const [minLon, maxLon] = robustRange(points.map((p) => p.lon));
  const spanLat = maxLat - minLat || 1;
  const spanLon = maxLon - minLon || 1;

  return points.map((p) => {
    const x = (p.lon - minLon) / spanLon;
    const y = (maxLat - p.lat) / spanLat;
    const outside = x < 0 || x > 1 || y < 0 || y > 1;
    const clamp = (v: number) => Math.min(1, Math.max(0, v));
    return { x: clamp(x), y: clamp(y), outside };
  });
}
