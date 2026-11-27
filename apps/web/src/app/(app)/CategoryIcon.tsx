import type { Category } from '@/lib/places/types';

// Nét vẽ đơn giản theo nhóm category; dùng thay ảnh khi địa điểm không có ảnh giấy phép mở.
const PATHS: Record<Category, string[]> = {
  museum: ['M3 21h18', 'M4 10h16', 'M12 3 3 8h18z', 'M6 10v8M10 10v8M14 10v8M18 10v8'],
  landmark: ['M4 21V9l8-5 8 5v12', 'M9 21v-6h6v6'],
  religious_site: ['M12 3v4M10 5h4', 'M6 21V11l6-4 6 4v10', 'M10 21v-4a2 2 0 0 1 4 0v4'],
  park_nature: ['M12 21v-6', 'M7 15a5 5 0 1 1 10 0z', 'M4 21h16'],
  entertainment: ['M4 8h16v10H4z', 'M8 8V5M16 8V5', 'M10 12l4 2-4 2z'],
  art_culture: [
    'M12 3a9 9 0 1 0 0 18c1.5 0 2-1 2-2s-1-1.5-1-2.5S14 15 15 15h2a4 4 0 0 0 4-4c0-4.4-4-8-9-8z',
    'M7.5 11h.01M10 7.5h.01M14.5 7.5h.01',
  ],
  shopping_mall: ['M5 8h14l-1 13H6z', 'M9 8V6a3 3 0 0 1 6 0v2'],
  market: [
    'M3 9l2-5h14l2 5',
    'M3 9h18v2a3 3 0 0 1-6 0 3 3 0 0 1-6 0 3 3 0 0 1-6 0z',
    'M5 13v8h14v-8',
  ],
  street_food: ['M4 11h16a8 8 0 0 1-16 0z', 'M8 7c0-1 1-1 1-2M12 7c0-1 1-1 1-2M16 7c0-1 1-1 1-2'],
  restaurant: ['M7 3v8a2 2 0 0 0 2 2v8', 'M11 3v8a2 2 0 0 1-2 2', 'M17 21V3c-2 1-3 4-3 8h3'],
  cafe: ['M5 8h11v6a5 5 0 0 1-5 5h-1a5 5 0 0 1-5-5z', 'M16 10h2a2 2 0 0 1 0 4h-2', 'M8 3v2M12 3v2'],
  nightlife: ['M6 3h12l-6 8z', 'M12 11v8', 'M8 21h8'],
};

export function CategoryIcon({ category, size = 24 }: { category: Category; size?: number }) {
  return (
    <svg
      aria-hidden="true"
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.8"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      {PATHS[category].map((d) => (
        <path key={d} d={d} />
      ))}
    </svg>
  );
}
