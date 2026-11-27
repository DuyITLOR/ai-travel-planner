---
name: next-web
description: "Rules for apps/web (Next.js 15 App Router, React 19, TypeScript strict, Supabase SSR auth) and packages/shared (Zod schemas): server versus client components, forms and server actions, calling the api, problem-details errors, secrets, integer VND, accessibility, Vitest. Load BEFORE writing, planning or reviewing anything under apps/web or packages/shared."
user-invocable: false
---

# Next web conventions

The reference implementation is sign-in. When unsure, copy its shape: `src/app/(auth)/` (page, client form, server actions, CSS module), `src/lib/supabase/` (clients), `src/lib/auth/` (routes, error messages), `src/middleware.ts`, `packages/shared/src/auth.ts` (input schemas).

## Layout

| What                                   | Where                                                                            |
| -------------------------------------- | -------------------------------------------------------------------------------- |
| Routes                                 | `apps/web/src/app/<route>/page.tsx` (+ `layout.tsx`, `loading.tsx`, `error.tsx`) |
| Pieces of one route                    | next to its `page.tsx`: `<Name>.tsx`, `actions.ts`, `<name>.module.css`          |
| Components used by more than one route | `apps/web/src/components/<Name>.tsx`                                             |
| Api client, Supabase clients, helpers  | `apps/web/src/lib/`                                                              |
| Schemas of api responses and of forms  | `packages/shared/src/<resource>.ts`, exported from `index.ts`                    |

Import with `@/…` inside the web app and `@atp/shared` for schemas. One component per file, named export, file name = component name in PascalCase. Only `page.tsx`, `layout.tsx` and the other Next file conventions use a default export. Route groups (`(auth)`) share a layout without adding a URL segment.

## Secrets

- The only environment values the web reads are public by design: `NEXT_PUBLIC_API_URL`, `NEXT_PUBLIC_SUPABASE_URL`, `NEXT_PUBLIC_SUPABASE_ANON_KEY`, `NEXT_PUBLIC_SITE_URL`. Next.js inlines every `NEXT_PUBLIC_*` into the browser bundle.
- Never a model key, `DATABASE_URL` or the Supabase `service_role` key in `apps/web`, under any name, server-side code included. A new env var is added to `apps/web/.env.example` with an empty value and a comment.
- The web talks to **our api and Supabase Auth only**. Weather, the LLM, anything with a key is called by the backend.

## Server and client components

- A component is a Server Component unless it needs state, an effect, an event handler or a browser API. Put `'use client'` on the smallest leaf that needs it (`AuthForm.tsx`), not on the page.
- `params` and `searchParams` are Promises in Next 15: `const { next } = await searchParams`.
- No `fetch` in a `useEffect` without cancelling on unmount (`AbortController`). Loading, error and empty states are all designed, not left blank.

## Session and Supabase

- The session lives in cookies, shared by server and browser. Use the client for where the code runs: `supabaseServer()` in Server Components, Server Actions and Route Handlers (a new one per request, never cached in a module); `supabaseBrowser()` in Client Components.
- On the server, the signed-in user is `currentUser()` (it calls `auth.getUser()`, which verifies with Supabase). Never decide access from `getSession()` alone or from anything the browser sent.
- A page that needs sign-in adds its prefix to `PROTECTED_PREFIXES` in `src/lib/auth/routes.ts`; the middleware does the redirect. Do not hand-roll a redirect in the page.
- In `middleware.ts`, nothing goes between `createServerClient` and `auth.getUser()`: that call refreshes the token.
- A redirect target taken from input (`next`) always goes through `safeNextPath`. Path constants come from `routes.ts`, not string literals.
- Both clients check `isSupabaseConfigured`; a page must still render, with a clear message, when the env values are missing.

## Forms and server actions

- A form posts to a Server Action in `actions.ts` (`'use server'`) next to the route; the client form uses `useActionState` and `useFormStatus`.
- The action validates with the shared Zod schema (`Schema.safeParse`), and returns a state object (`error`, `notice`, `fieldErrors`, the values to keep) instead of throwing. `redirect()` only after success.
- Validation messages live in the Zod schema, in Vietnamese. Messages for upstream errors are mapped from an error **code** in one function (`authErrorMessage`); raw error text is never shown to the user.
- Log the code and status, never the email, the password or a token.

## Calling the api

1. **Go through `src/lib/api.ts`.** No `fetch(` to the api anywhere else, and no hard-coded origin: `API_URL` only.
2. **Parse every response with its schema** from `@atp/shared`: `Plan.parse(await res.json())`. A type assertion (`as Plan`, or the `T` of `apiGet<T>`) is not validation; a response is `unknown` until a schema has parsed it.
3. **Authenticated calls** send `Authorization: Bearer <access token>` taken from the Supabase client for where the code runs. Never store the token yourself (no `localStorage`, no cookie you write).
4. **Errors are problem documents** (`application/problem+json`). Branch on the `type` slug, never on `title` or `detail` text. A `type` you do not know is handled by its `status`: 4xx shows `detail` and does not retry; 5xx is treated as `internal`.
   - `401` → the session is gone: sign in again.
   - `409 version-conflict` → refetch the resource and let the user review again; never resend automatically.
   - `422 plan-rejected` → show each violation's `message`; accept rule codes you do not know.
5. **`Idempotency-Key`**: `crypto.randomUUID()`, created once when the user commits to the action and reused for every retry of that same action. A new key per request defeats it.
6. The client never sends an amount the server computes, and never trusts a cost it computed itself over the one in the response.

## Shared schemas (`packages/shared`)

- One Zod schema per JSON shape the api returns and per form the web submits, with `export type X = z.infer<typeof X>`. For a response, the Rust view struct is the other half: both change in the same PR.
- Response objects end in `.passthrough()`: clients ignore fields they do not know.
- Rust `Option<T>` sent as `null` is `.nullable()`; a member that may be absent is `.optional()`. They are not interchangeable.
- Money is `z.number().int().min(0).max(Number.MAX_SAFE_INTEGER)`; field names end in `Vnd`. Never `z.number()` alone for an amount.
- Open sets the contract says may grow (violation rule codes, problem types) are `z.string()`, not `z.enum`.
- A limit that also exists elsewhere (`PASSWORD_MIN_LENGTH` and `supabase/config.toml`) is one exported constant with a comment naming the other place.
- A schema has a test, and a response schema's test checks it against the contract file. Editing an existing test is "ask first".

## TypeScript

- `strict`, `noUncheckedIndexedAccess` and `verbatimModuleSyntax` are on: an indexed read is `T | undefined`, and type-only imports use `import type`.
- No `any`, no `as` to silence an error, no non-null `!` on data from the network, no `@ts-ignore` / `@ts-expect-error`, no `eslint-disable`. Narrow instead.
- Props are typed inline or with a `type` right above the component; no `React.FC`.
- Unused parameters start with `_`.

## Money, dates, text

- Amounts stay integers end to end. Display with `new Intl.NumberFormat('vi-VN').format(vnd)` plus `đ`; no division, no `toFixed`, no rounding to thousands in logic.
- Times from the api (`HH:MM`, `YYYY-MM-DD`) are Asia/Ho_Chi_Minh wall-clock strings: show them as they are. Do not pass them through `new Date()`, which applies the browser's zone. Timestamps (`savedAt`) are UTC and are formatted with `timeZone: 'Asia/Ho_Chi_Minh'`.
- UI text is Vietnamese (`<html lang="vi">`). Identifiers and commit messages are English.

## UI

- Build a screen from its artboard on the design canvas: same structure, states and wording. Say which artboard at the top of the CSS module.
- Styling is a CSS Module next to the route (`<name>.module.css`); `globals.css` is for resets and tokens only. Colours are CSS variables, not literals scattered in rules. A CSS framework or a component library is a new dependency: ask first.
- Real elements: `<button type="button">` for actions, `<Link>` / `<a href>` for navigation, `<input>` with a `<label>`. Never an `onClick` on a `div` or `span`.
- Icon-only buttons have an `aria-label`; decorative SVG has `aria-hidden="true"`; toggles expose `aria-pressed`; a field's error is tied to it with `aria-describedby`.
- Touch targets at least 44 px; text contrast at least 4.5:1; every page works at 390 px wide.
- Images through `next/image` with `alt`; a place without a licensed photo shows its category placeholder (`data/README.md`).
- Lists rendered from data have a stable `key` (the id, never the index).

## Tests

- Vitest, file next to the code as `<name>.test.ts(x)`. No network: stub `fetch` with `vi.stubGlobal` and restore it in `afterEach`.
- Pure helpers (`safeNextPath`, schemas) are tested with the hostile inputs they exist to stop, not only the happy path.
- Test what a caller observes: the request sent (URL, headers), the parsed result, the error for a non-2xx answer or a body the schema rejects.
- `vitest run` must fail when there are no tests: never `--passWithNoTests`.

## Before saying it is done

`pnpm format && pnpm lint && pnpm typecheck && pnpm test` from the repo root, and for anything visible, `pnpm dev` and a look at the screen at phone and desktop width.
