import { describe, expect, it } from 'vitest';
import { layoutPoints } from './map';

describe('layoutPoints', () => {
  it('một điểm nằm giữa', () => {
    expect(layoutPoints([{ lat: 10.7, lon: 106.7 }])).toEqual([{ x: 0.5, y: 0.5, outside: false }]);
  });

  it('bắc ở trên, đông ở phải', () => {
    const [sw, ne] = layoutPoints([
      { lat: 10.7, lon: 106.6 },
      { lat: 10.8, lon: 106.8 },
    ]);
    expect(sw).toMatchObject({ x: 0, y: 1, outside: false });
    expect(ne).toMatchObject({ x: 1, y: 0, outside: false });
  });

  it('điểm ở xa bị kẹp vào mép, phần còn lại không bị dồn cục', () => {
    const center = Array.from({ length: 20 }, (_, i) => ({
      lat: 10.77 + (i % 5) * 0.005,
      lon: 106.69 + Math.floor(i / 5) * 0.005,
    }));
    const cuChi = { lat: 11.06, lon: 106.46 };
    const pos = layoutPoints([...center, cuChi]);
    const far = pos[pos.length - 1];
    expect(far).toMatchObject({ x: 0, y: 0, outside: true });
    const xs = pos.slice(0, -1).map((p) => p.x);
    expect(Math.max(...xs) - Math.min(...xs)).toBeGreaterThan(0.5);
  });
});
