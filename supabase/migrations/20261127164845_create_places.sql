-- Bộ dữ liệu địa điểm dùng chung (data/places.json, menu_items.json, travel_time_matrix.json).
-- Chỉ đọc với client: RLS cho anon/authenticated SELECT, không có policy ghi.
-- Nạp/cập nhật dữ liệu bằng `cargo run --bin seed_places` (apps/api), chạy lại được (upsert).

create table public.places (
  id                   text primary key check (id ~ '^p[0-9]{3}$'),
  name                 text not null,
  name_en              text,
  category             text not null check (category in (
                         'museum', 'landmark', 'religious_site', 'park_nature', 'entertainment',
                         'art_culture', 'shopping_mall', 'market', 'street_food', 'restaurant',
                         'cafe', 'nightlife')),
  category_vi          text not null,
  -- Tên quận cũ (trước 2025), xem data/README.md.
  district             text not null,
  address              text not null,
  lat                  double precision not null check (lat between -90 and 90),
  lon                  double precision not null check (lon between -180 and 180),
  coord_precision      text not null check (coord_precision in ('poi', 'street')),
  description          text not null,
  indoor_outdoor       text not null check (indoor_outdoor in ('indoor', 'outdoor')),
  ticket_price_vnd     bigint not null check (ticket_price_vnd >= 0),
  price_level          smallint not null check (price_level between 1 and 4),
  typical_duration_min integer not null check (typical_duration_min > 0),
  -- mon..sun → [["HH:MM","HH:MM"], ...]; giờ sau nửa đêm ghi quá 24 ("26:00").
  opening_hours        jsonb not null check (jsonb_typeof(opening_hours) = 'object'),
  opening_hours_note   text,
  interests            text[] not null default '{}',
  images               jsonb not null default '[]' check (jsonb_typeof(images) = 'array'),
  sources              jsonb not null default '[]' check (jsonb_typeof(sources) = 'array'),
  checked_at           date not null,
  verification         jsonb not null,
  notes                text
);

create index places_category_idx on public.places (category);
create index places_interests_idx on public.places using gin (interests);

create table public.menu_items (
  item_id   text primary key,
  place_id  text not null references public.places (id) on delete cascade,
  name      text not null,
  kind      text not null check (kind in ('ticket', 'food', 'drink', 'service')),
  price_vnd bigint not null check (price_vnd >= 0)
);

create index menu_items_place_id_idx on public.menu_items (place_id);

-- Thời gian đi xe máy giữa mọi cặp địa điểm (kể cả đường chéo = 0).
create table public.travel_times (
  from_place_id text not null references public.places (id) on delete cascade,
  to_place_id   text not null references public.places (id) on delete cascade,
  minutes       integer not null check (minutes >= 0),
  distance_m    integer not null check (distance_m >= 0),
  primary key (from_place_id, to_place_id)
);

create index travel_times_to_place_id_idx on public.travel_times (to_place_id);

alter table public.places enable row level security;
alter table public.menu_items enable row level security;
alter table public.travel_times enable row level security;

create policy "places are public" on public.places
  for select to anon, authenticated using (true);
create policy "menu items are public" on public.menu_items
  for select to anon, authenticated using (true);
create policy "travel times are public" on public.travel_times
  for select to anon, authenticated using (true);
