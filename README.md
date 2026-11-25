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

Yêu cầu: Node 20 (`.nvmrc`), pnpm 10 (`corepack enable`), `rustup`: lần đầu chạy `cargo`, rustup tự cài Rust 1.100.0
theo `apps/api/rust-toolchain.toml` (cùng bản với CI).

```bash
pnpm install --frozen-lockfile
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

Mỗi lệnh kiểm tra chạy cả phần TypeScript lẫn Rust.

| Lệnh                           | Việc                                                                    |
| ------------------------------ | ----------------------------------------------------------------------- |
| `pnpm dev`                     | Chạy web và api                                                         |
| `pnpm lint`                    | ESLint toàn repo + `cargo clippy --all-targets --locked -- -D warnings` |
| `pnpm format` / `format:check` | Prettier + `cargo fmt` / `cargo fmt --check`                            |
| `pnpm typecheck`               | `tsc --noEmit` từng package + `cargo check --all-targets --locked`      |
| `pnpm test`                    | Vitest từng package + `cargo test --locked`                             |
| `pnpm build`                   | Build tất cả (`next build`, `cargo build --release --locked`)           |

CI (`.github/workflows/ci.yml`) chạy **đúng các lệnh trên** (bốn lệnh kiểm tra + build), mỗi lệnh một job: `lint` → `pnpm lint`,
`format` → `pnpm format:check`, `typecheck` → `pnpm typecheck`, `test` → `pnpm test`, cộng job `build` → `pnpm build` (web và api).
Bốn job đầu là required check của `main`: PR đỏ thì không merge được.

Clone mới, cài Node 20 + pnpm 10 + rustup (Rust 1.100.0 tự cài theo `rust-toolchain.toml`), chạy `pnpm install --frozen-lockfile` (như CI) rồi bốn lệnh kiểm tra là xanh; không cần `.env`.

## Contract và contract test

- `POST /plans/{planId}/save` (bước Save chia sẻ lịch trình cho nhóm) và `GET /plans/{planId}` (đọc plan và `version` trước khi save): [`docs/contract/save-plan.md`](docs/contract/save-plan.md).
- Test: `apps/api/tests/save_plan_contract.rs` (mỗi problem `type`, quyền sở hữu, idempotency, validator)
  và `packages/shared/test/plan.test.ts` (schema Zod khớp contract). Cả hai chạy trong `pnpm test`.
- Bảng lỗi trong contract được test đọc trực tiếp: sửa code mà không sửa contract (hoặc ngược lại) thì CI đỏ.

## Model và spending cap

- Model chính: **Qwen 2.5 7B qua Ollama** trên máy demo, gọi qua Cloudflare Tunnel có token bí mật: không tốn tiền theo lượt.
- Dự phòng: **Gemini 3.5 Flash-Lite** (Google AI Studio). Spending cap: key `GEMINI_API_KEY` thuộc một Google Cloud
  project **không gắn billing account** (free tier). Không có billing thì Google không tính tiền được: vượt quota
  free thì request bị từ chối (429), không phát sinh chi phí. Đây là cap cứng duy nhất; budget của Google Cloud
  chỉ gửi cảnh báo, không chặn. Nếu sau này phải bật billing: đặt budget US$5/tháng có cảnh báo ở 50/90/100% và
  hạ quota requests/ngày của Generative Language API trong Cloud Console.
- Ước tính xấu nhất khi chạy toàn bộ trên Gemini trả phí: ~30k token vào + 4k ra ≈ **US$0.02 cho một lịch trình**.
- Key chỉ nằm ở môi trường của api (`apps/api/.env`, dashboard Render). Không bao giờ trong `apps/web` hay `NEXT_PUBLIC_*`.
- Agent chưa gọi model ở PA#2; test không bao giờ gọi model thật.

## Làm việc với AI assistant

- Luật cho assistant: [`CLAUDE.md`](CLAUDE.md) (stack, lệnh, quy ước, điều cấm, quyền 3 mức).
- Quyền được máy thực thi: [`.claude/settings.json`](.claude/settings.json) (`allow` / `ask` / `deny`).
- Nhật ký dùng AI: [`AI-LOG.md`](AI-LOG.md).

## Quy trình

- Mỗi việc là một issue trên GitHub Projects; nhánh đặt tên `feat/<issue>-mo-ta`, `fix/...`, `chore/...`.
- Mở PR vào `main`; ruleset "Protect main" chỉ cho merge khi 4 check `lint`, `format`, `typecheck`, `test` xanh. Chưa bắt buộc approval: nhóm 3 người, merge bằng merge commit.
- Không commit `.env` hay key dịch vụ; key chỉ nằm ở backend.

## Deploy

- **Web:** import repo vào Vercel, Root Directory = `apps/web`, đặt `NEXT_PUBLIC_*`.
- **Api:** Render → New → Blueprint, chọn repo (đọc `render.yaml`, runtime Rust); điền `WEB_ORIGIN`, `DATABASE_URL`
  và `SUPABASE_JWT_SECRET`.
