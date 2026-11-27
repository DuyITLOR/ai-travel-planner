//! Nạp bộ dữ liệu địa điểm (`data/*.json` ở gốc repo) vào Postgres của Supabase.
//!
//! ```text
//! cd apps/api && cargo run --bin seed_places            # đọc ../../data
//! cargo run --bin seed_places -- /đường/dẫn/tới/data    # thư mục khác
//! ```
//!
//! Cần `DATABASE_URL` (đọc từ `apps/api/.env` như api) và schema của migration
//! `supabase/migrations/*_create_places.sql`. Chạy lại được: places, menu_items và
//! travel_times được upsert trong một transaction; món và cặp đường đi không còn trong
//! file thì bị xoá để database khớp đúng bộ dữ liệu.

use std::{collections::HashSet, path::PathBuf};

use anyhow::{Context, bail, ensure};
use serde::Deserialize;
use serde_json::Value;
use sqlx::postgres::PgPoolOptions;

#[derive(Deserialize)]
struct Place {
    id: String,
    name: String,
    name_en: Option<String>,
    category: String,
    category_vi: String,
    district: String,
    address: String,
    lat: f64,
    lon: f64,
    coord_precision: String,
    description: String,
    indoor_outdoor: String,
    ticket_price_vnd: i64,
    price_level: i16,
    typical_duration_min: i32,
    opening_hours: Value,
    opening_hours_note: Option<String>,
    interests: Vec<String>,
    images: Value,
    sources: Value,
    checked_at: String,
    verification: Value,
    notes: Option<String>,
}

#[derive(Deserialize)]
struct MenuItem {
    item_id: String,
    place_id: String,
    name: String,
    kind: String,
    price_vnd: i64,
}

#[derive(Deserialize)]
struct Matrix {
    place_ids: Vec<String>,
    minutes: Vec<Vec<i32>>,
    distance_m: Vec<Vec<i32>>,
}

fn read_json<T: for<'de> Deserialize<'de>>(dir: &std::path::Path, file: &str) -> anyhow::Result<T> {
    let path = dir.join(file);
    let text = std::fs::read_to_string(&path).with_context(|| format!("đọc {}", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))
}

/// Kiểm tra trước khi ghi để lỗi dữ liệu không làm hỏng giữa chừng.
fn validate(places: &[Place], menu: &[MenuItem], matrix: &Matrix) -> anyhow::Result<()> {
    let ids: HashSet<&str> = places.iter().map(|p| p.id.as_str()).collect();
    ensure!(ids.len() == places.len(), "places.json có id trùng");

    let mut item_ids = HashSet::new();
    for m in menu {
        ensure!(
            item_ids.insert(m.item_id.as_str()),
            "menu_items.json trùng item_id {}",
            m.item_id
        );
        ensure!(
            ids.contains(m.place_id.as_str()),
            "{} trỏ tới place_id lạ {}",
            m.item_id,
            m.place_id
        );
    }

    let n = matrix.place_ids.len();
    ensure!(
        matrix.minutes.len() == n && matrix.distance_m.len() == n,
        "ma trận không vuông: {n} place_ids"
    );
    for (i, (mins, dist)) in matrix.minutes.iter().zip(&matrix.distance_m).enumerate() {
        ensure!(
            mins.len() == n && dist.len() == n,
            "hàng {i} của ma trận không đủ {n} cột"
        );
    }
    for id in &matrix.place_ids {
        ensure!(ids.contains(id.as_str()), "ma trận có place_id lạ {id}");
    }
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let dir = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data")));

    let places: Vec<Place> = read_json(&dir, "places.json")?;
    let menu: Vec<MenuItem> = read_json(&dir, "menu_items.json")?;
    let matrix: Matrix = read_json(&dir, "travel_time_matrix.json")?;
    validate(&places, &menu, &matrix)?;

    let Some(url) = std::env::var("DATABASE_URL").ok().filter(|s| !s.is_empty()) else {
        bail!("thiếu DATABASE_URL (xem apps/api/.env.example)");
    };
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await
        .context("không kết nối được Postgres")?;

    let mut tx = pool.begin().await?;

    // Mỗi cột là một mảng, ghép lại bằng unnest: một câu lệnh cho cả bảng.
    // jsonb và text[] lồng nhau đi qua dạng chuỗi JSON rồi ép kiểu trong SQL.
    let col = |f: fn(&Place) -> String| places.iter().map(f).collect::<Vec<_>>();
    let json =
        |f: fn(&Place) -> &Value| places.iter().map(|p| f(p).to_string()).collect::<Vec<_>>();
    sqlx::query(
        r#"
        insert into public.places (
          id, name, name_en, category, category_vi, district, address, lat, lon,
          coord_precision, description, indoor_outdoor, ticket_price_vnd, price_level,
          typical_duration_min, opening_hours, opening_hours_note, interests, images,
          sources, checked_at, verification, notes)
        select t.id, t.name, t.name_en, t.category, t.category_vi, t.district, t.address,
               t.lat, t.lon, t.coord_precision, t.description, t.indoor_outdoor,
               t.ticket_price_vnd, t.price_level, t.typical_duration_min,
               t.opening_hours::jsonb, t.opening_hours_note,
               array(select jsonb_array_elements_text(t.interests::jsonb)),
               t.images::jsonb, t.sources::jsonb, t.checked_at::date,
               t.verification::jsonb, t.notes
        from unnest(
          $1::text[], $2::text[], $3::text[], $4::text[], $5::text[], $6::text[], $7::text[],
          $8::float8[], $9::float8[], $10::text[], $11::text[], $12::text[], $13::int8[],
          $14::int2[], $15::int4[], $16::text[], $17::text[], $18::text[], $19::text[],
          $20::text[], $21::text[], $22::text[], $23::text[]
        ) as t(id, name, name_en, category, category_vi, district, address, lat, lon,
               coord_precision, description, indoor_outdoor, ticket_price_vnd, price_level,
               typical_duration_min, opening_hours, opening_hours_note, interests, images,
               sources, checked_at, verification, notes)
        on conflict (id) do update set
          name = excluded.name, name_en = excluded.name_en, category = excluded.category,
          category_vi = excluded.category_vi, district = excluded.district,
          address = excluded.address, lat = excluded.lat, lon = excluded.lon,
          coord_precision = excluded.coord_precision, description = excluded.description,
          indoor_outdoor = excluded.indoor_outdoor, ticket_price_vnd = excluded.ticket_price_vnd,
          price_level = excluded.price_level, typical_duration_min = excluded.typical_duration_min,
          opening_hours = excluded.opening_hours, opening_hours_note = excluded.opening_hours_note,
          interests = excluded.interests, images = excluded.images, sources = excluded.sources,
          checked_at = excluded.checked_at, verification = excluded.verification,
          notes = excluded.notes
        "#,
    )
    .bind(col(|p| p.id.clone()))
    .bind(col(|p| p.name.clone()))
    .bind(places.iter().map(|p| p.name_en.clone()).collect::<Vec<_>>())
    .bind(col(|p| p.category.clone()))
    .bind(col(|p| p.category_vi.clone()))
    .bind(col(|p| p.district.clone()))
    .bind(col(|p| p.address.clone()))
    .bind(places.iter().map(|p| p.lat).collect::<Vec<_>>())
    .bind(places.iter().map(|p| p.lon).collect::<Vec<_>>())
    .bind(col(|p| p.coord_precision.clone()))
    .bind(col(|p| p.description.clone()))
    .bind(col(|p| p.indoor_outdoor.clone()))
    .bind(
        places
            .iter()
            .map(|p| p.ticket_price_vnd)
            .collect::<Vec<_>>(),
    )
    .bind(places.iter().map(|p| p.price_level).collect::<Vec<_>>())
    .bind(
        places
            .iter()
            .map(|p| p.typical_duration_min)
            .collect::<Vec<_>>(),
    )
    .bind(json(|p| &p.opening_hours))
    .bind(
        places
            .iter()
            .map(|p| p.opening_hours_note.clone())
            .collect::<Vec<_>>(),
    )
    .bind(
        places
            .iter()
            .map(|p| serde_json::to_string(&p.interests))
            .collect::<Result<Vec<_>, _>>()?,
    )
    .bind(json(|p| &p.images))
    .bind(json(|p| &p.sources))
    .bind(col(|p| p.checked_at.clone()))
    .bind(json(|p| &p.verification))
    .bind(places.iter().map(|p| p.notes.clone()).collect::<Vec<_>>())
    .execute(&mut *tx)
    .await
    .context("ghi places")?;

    let item_ids: Vec<&str> = menu.iter().map(|m| m.item_id.as_str()).collect();
    let removed_items =
        sqlx::query("delete from public.menu_items where item_id <> all($1::text[])")
            .bind(&item_ids)
            .execute(&mut *tx)
            .await?
            .rows_affected();
    sqlx::query(
        r#"
        insert into public.menu_items (item_id, place_id, name, kind, price_vnd)
        select * from unnest($1::text[], $2::text[], $3::text[], $4::text[], $5::int8[])
        on conflict (item_id) do update set
          place_id = excluded.place_id, name = excluded.name,
          kind = excluded.kind, price_vnd = excluded.price_vnd
        "#,
    )
    .bind(&item_ids)
    .bind(menu.iter().map(|m| m.place_id.as_str()).collect::<Vec<_>>())
    .bind(menu.iter().map(|m| m.name.as_str()).collect::<Vec<_>>())
    .bind(menu.iter().map(|m| m.kind.as_str()).collect::<Vec<_>>())
    .bind(menu.iter().map(|m| m.price_vnd).collect::<Vec<_>>())
    .execute(&mut *tx)
    .await
    .context("ghi menu_items")?;

    // Ma trận n×n → n² hàng (from, to).
    let n = matrix.place_ids.len();
    let (mut from, mut to, mut minutes, mut distance) = (
        Vec::with_capacity(n * n),
        Vec::with_capacity(n * n),
        Vec::with_capacity(n * n),
        Vec::with_capacity(n * n),
    );
    for (i, a) in matrix.place_ids.iter().enumerate() {
        for (j, b) in matrix.place_ids.iter().enumerate() {
            from.push(a.as_str());
            to.push(b.as_str());
            minutes.push(matrix.minutes[i][j]);
            distance.push(matrix.distance_m[i][j]);
        }
    }
    let removed_pairs = sqlx::query(
        "delete from public.travel_times where from_place_id <> all($1::text[]) \
         or to_place_id <> all($1::text[])",
    )
    .bind(&matrix.place_ids)
    .execute(&mut *tx)
    .await?
    .rows_affected();
    sqlx::query(
        r#"
        insert into public.travel_times (from_place_id, to_place_id, minutes, distance_m)
        select * from unnest($1::text[], $2::text[], $3::int4[], $4::int4[])
        on conflict (from_place_id, to_place_id) do update set
          minutes = excluded.minutes, distance_m = excluded.distance_m
        "#,
    )
    .bind(&from)
    .bind(&to)
    .bind(&minutes)
    .bind(&distance)
    .execute(&mut *tx)
    .await
    .context("ghi travel_times")?;

    tx.commit().await?;

    println!(
        "Đã nạp {} địa điểm, {} món ({} món cũ bị xoá), {} cặp đường đi ({} cặp cũ bị xoá).",
        places.len(),
        menu.len(),
        removed_items,
        from.len(),
        removed_pairs
    );
    Ok(())
}
