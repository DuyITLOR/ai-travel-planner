# AI Travel Planner

Lập lịch trình du lịch TP.HCM bằng AI, kèm chia sẻ và đánh giá địa điểm.
Proposal: [`docs/proposal/`](docs/proposal/) (bản đầy đủ: `proposal-full.docx`).

## Thành viên

| MSSV     | Họ tên            | Phụ trách chính                                                 |
| -------- | ----------------- | --------------------------------------------------------------- |
| 23127178 | Lê Nhựt Duy       | Dữ liệu & menu, sổ địa điểm, bạn đồng hành, đo chất lượng Agent |
| 23127334 | Nguyễn Thành Dâng | Giao diện, lịch trình & chi tiêu, bài đăng                      |
| 23127344 | Trương Thành Đạt  | Hạ tầng, nhóm, Agent & tool                                     |

## Cấu trúc

```
apps/
  web/              Next.js 15 (App Router) — deploy Vercel
    src/app/        route và layout
    src/components/ component dùng lại
    src/lib/        supabase client, api client
  api/              Rust (Axum + Tokio) — deploy Render
    src/main.rs     khởi động server, logging, graceful shutdown
    src/lib.rs      build_app: router, CORS, trace, AppState
    src/config.rs   đọc biến môi trường
    src/db.rs       pool Postgres của Supabase (`sqlx`) + ping
    src/routes/     mỗi tính năng một module; health.rs = GET /health
    tests/          integration test (gọi router trực tiếp, không mở port)
    package.json    chỉ để `pnpm` ở root gọi được cargo
packages/
  shared/           schema Zod cho web; khớp tay với struct Rust của api
docs/               proposal, spec, báo cáo
supabase/           config.toml (Supabase CLI) và migrations/ (schema SQL)
render.yaml         blueprint deploy api
```

## Bắt đầu

Yêu cầu: Node 20 (`.nvmrc`), pnpm 10 (`corepack enable`), Rust stable (`rustup`).

```bash
pnpm install
cp apps/api/.env.example apps/api/.env
cp apps/web/.env.example apps/web/.env.local
pnpm dev            # web :3000, api :4000 (cargo run)
```

Kiểm tra api: `curl localhost:4000/health` → `db` là `connected`, `not_configured` hoặc `error`.

Thêm tính năng cho api: tạo `src/routes/<ten>.rs`, khai báo trong `src/routes/mod.rs` và gắn route vào `router()`.
Truy vấn trong handler qua `State<AppState>` → `state.db.pool()` rồi dùng `sqlx::query(...)`.
Khi đổi JSON trả về, sửa schema tương ứng trong `packages/shared` trong cùng PR.

### Database: Supabase (Postgres)

Api nối thẳng vào Postgres bằng `sqlx`; web dùng `@supabase/supabase-js` cho đăng nhập (Supabase Auth) và realtime.

- **Project trên cloud:** tạo project ở supabase.com, bấm **Connect**, lấy chuỗi **Session pooler** (cổng 5432)
  điền vào `DATABASE_URL` trong `apps/api/.env`. Lấy Project URL và anon key điền vào `apps/web/.env.local`.
  Không dùng Transaction pooler (cổng 6543) cho api.
- **Chạy local (cần Docker):** `supabase start`, rồi đặt
  `DATABASE_URL=postgresql://postgres:postgres@127.0.0.1:54322/postgres`. URL và key cho web xem bằng `supabase status`.
- **Đổi schema:** `supabase migration new <ten>` tạo file SQL trong `supabase/migrations/`;
  áp dụng local bằng `supabase db reset`, lên cloud bằng `supabase db push` (sau khi `supabase link`).
  Bảng nào web đọc trực tiếp thì phải bật Row Level Security.

## Lệnh

| Lệnh                           | Việc                        |
| ------------------------------ | --------------------------- |
| `pnpm dev`                     | Chạy web và api             |
| `pnpm lint`                    | ESLint toàn repo            |
| `pnpm format` / `format:check` | Prettier                    |
| `pnpm typecheck`               | `tsc --noEmit` từng package |
| `pnpm test`                    | Vitest từng package         |
| `pnpm build`                   | Build tất cả                |

CI (GitHub Actions) có hai job song song: TypeScript (Prettier, ESLint, typecheck, test, build web) và Rust (rustfmt, Clippy, test).

## Quy trình

- Mỗi việc là một issue trên GitHub Projects; nhánh đặt tên `feat/<issue>-mo-ta`, `fix/...`, `chore/...`.
- Mở PR vào `main`, CI xanh và một người review rồi mới merge.
- Không commit `.env` hay key dịch vụ; key chỉ nằm ở backend.

## Deploy

- **Web:** import repo vào Vercel, Root Directory = `apps/web`, đặt `NEXT_PUBLIC_*`.
- **Api:** Render → New → Blueprint, chọn repo (đọc `render.yaml`, runtime Rust); điền `WEB_ORIGIN` và `DATABASE_URL`.
