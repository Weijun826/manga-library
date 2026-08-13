# 漫畫書庫 MVP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 建立一個可安裝、完全本機優先的 Windows 漫畫收藏管理器，支援系列／版本／冊別、批次建庫、ISBN 新增、缺卷與待補清單、封面、備份及還原。

**Architecture:** 使用官方 Tauri 2 `react-ts` 範本建立殼層；React 只透過 TypeScript service ports 操作資料，Rust 以 Tauri commands、`rusqlite` 與版本化 migration 負責 SQLite transaction、封面及備份。純 TypeScript domain 模組負責 ISBN、冊別排序及收藏衍生規則，metadata providers 經正規化管線回傳候選資料，不直接寫入收藏。

**Tech Stack:** Tauri 2、React 19、TypeScript 5.9、Vite、Vitest、Testing Library、Rust 2021、rusqlite bundled SQLite、serde、uuid、reqwest、zip、sha2、Lucide React、CSS Modules／全域 design tokens。

## Global Constraints

- 只支援 Windows 桌面；不建立手機、macOS、Linux 或瀏覽器產品。
- 核心收藏功能完全離線可用；不需要登入、帳號或雲端同步。
- 工作名稱與視窗名稱使用「漫畫書庫」。
- 技術主版本固定為 Tauri 2、React、TypeScript、Vite、Rust 與 SQLite。
- 使用官方 `create-tauri-app` 的 `react-ts` 範本；不採用第三方全配模板。
- 不加入 Tailwind、shadcn/ui、TanStack Query、Zustand 或前端路由套件；MVP 使用 React state、context 與明確 service ports。
- SQLite 主要資料以不可變 UUID 為主鍵；時間戳以 UTC ISO 8601 保存。
- 擁有、已讀、想買分開記錄；擁有會取消想買，取消擁有不清除已讀。
- 缺卷只包含 `availability_status = released` 且未擁有的冊別。
- 一般修改只有在 SQLite transaction 成功後才能顯示「已保存」。
- 初版只允許 Google Books、Open Library 和本機快取；不爬取出版社網站。
- 不使用 `csp: null`；正式設定必須有最小 CSP 與最小 Tauri capabilities。
- 視覺採「現代日本編輯部 × 私人漫畫收藏櫃」，單一淺色主題。
- 初始視窗 1280 × 820，最小視窗 1040 × 700；使用 Windows 系統標題列。
- 支援 `Ctrl+K`、`Ctrl+N`、`Esc` 與 Windows 減少動態效果。
- 正式版本在目標桌機 2 秒內出現可操作畫面；一般畫面與轉場以穩定 60 FPS 為驗收目標。
- 自動備份在資料有變更且距上次成功備份至少 24 小時後建立，保留最近 7 份。
- 手動備份副檔名為 `.mangashelf-backup`，還原前必須先建立可恢復備份。
- 每一個產品變更依 TDD 順序完成：先失敗測試、確認失敗、最小實作、確認通過、再 commit。

---

## File Structure

### Project foundation

- `package.json`：前端命令與依賴；唯一套件管理器是 pnpm。
- `pnpm-lock.yaml`：鎖定實際套件版本。
- `vite.config.ts`、`tsconfig*.json`、`index.html`：官方 React TypeScript 範本設定。
- `src-tauri/Cargo.toml`、`src-tauri/tauri.conf.json`：Windows Tauri、Rust 依賴與 bundle 設定。
- `src-tauri/capabilities/default.json`：主視窗所需的最小權限。
- `src-tauri/src/lib.rs`、`src-tauri/src/main.rs`：組裝 Tauri plugins、managed state 與 commands。

### Frontend domain and services

- `src/domain/model.ts`：共享 TypeScript domain types 與 union literals。
- `src/domain/isbn.ts`：ISBN 正規化及校驗。
- `src/domain/volumeOrder.ts`：冊別標籤轉自然排序鍵。
- `src/domain/collectionRules.ts`：擁有／已讀／想買規則、缺卷與完成度。
- `src/services/libraryPort.ts`：UI 使用的資料操作介面。
- `src/services/tauriLibrary.ts`：把 Tauri invoke 映射成 `LibraryPort`。
- `src/services/metadata/types.ts`：候選資料與 provider error types。
- `src/services/metadata/providers.ts`：Google Books、Open Library 與 fallback pipeline。

### Frontend application and UI

- `src/app/App.tsx`：應用程式組裝、頂層頁面狀態與全域快捷鍵。
- `src/app/LibraryContext.tsx`：注入 `LibraryPort` 並協調 refresh。
- `src/app/navigation.ts`：四個固定導覽目的地與 selection types。
- `src/components/AppShell.tsx`：系統標題列下方的側邊導覽與內容框架。
- `src/components/SeriesCard.tsx`、`VolumeCard.tsx`、`TextCover.tsx`：收藏視覺元件。
- `src/components/StatusBadge.tsx`、`SaveStatus.tsx`、`EmptyState.tsx`、`ErrorNotice.tsx`、`ConfirmDialog.tsx`：跨頁共用狀態元件。
- `src/features/dashboard/DashboardPage.tsx`：摘要、最近新增與未收齊系列。
- `src/features/library/LibraryPage.tsx`：搜尋、篩選與系列格狀書庫。
- `src/features/library/SeriesDetailPage.tsx`：版本切換、冊別格與批次模式。
- `src/features/wishlist/WishlistPage.tsx`：缺卷與想買清單。
- `src/features/add/AddMangaDialog.tsx`：搜尋作品／輸入 ISBN 的入口。
- `src/features/add/SeriesBatchFlow.tsx`：系列、版本、冊別與批次收藏流程。
- `src/features/add/IsbnFlow.tsx`：ISBN 查找、候選確認與手動 fallback。
- `src/features/volume/VolumeDetailDialog.tsx`：單冊收藏資料編輯。
- `src/features/settings/SettingsPage.tsx`：備份、還原、封面快取及版本資訊。
- `src/styles/tokens.css`、`src/styles/global.css`：單一主題、版面、焦點與 reduced-motion。

### Rust backend

- `src-tauri/migrations/0001_initial.sql`：完整初版 schema、constraints 與 indexes。
- `src-tauri/migrations/0002_backup_settings.sql`：自動備份與資料變更時間設定。
- `src-tauri/src/error.rs`：穩定可序列化錯誤碼。
- `src-tauri/src/db/mod.rs`：SQLite connection、foreign keys、migration 與 transaction helper。
- `src-tauri/src/db/models.rs`：Rust row/DTO types。
- `src-tauri/src/db/repository.rs`：查詢與 mutation commands 的資料實作。
- `src-tauri/src/commands/library.rs`：library query/mutation Tauri commands。
- `src-tauri/src/commands/assets.rs`：安全封面匯入與快取 commands。
- `src-tauri/src/commands/backup.rs`：一致快照、ZIP manifest、hash、保留與還原。
- `src-tauri/src/commands/mod.rs`：commands exports。
- `src-tauri/src/state.rs`：`AppState`、資料目錄與資料庫路徑。

### Tests and fixtures

- `src/test/setup.ts`：jsdom 與 Testing Library 設定。
- `src/test/fakes/fakeLibrary.ts`：可控制成功／失敗的記憶體 `LibraryPort`。
- `src/test/fixtures/library.ts`：十冊系列、普通版／完全版與特殊卷號 fixtures。
- `src/**/*.test.ts(x)`：domain、services 與 UI unit tests。
- `src-tauri/tests/database.rs`：migration、constraint、transaction 與 restart persistence。
- `src-tauri/tests/backup.rs`：備份／還原 round trip 與損壞包拒絕。
- `tests/manual/windows-release-checklist.md`：只能在正式 Windows bundle 手動確認的驗收記錄。

---

### Task 1: Scaffold the official Tauri React TypeScript application

**Files:**
- Create: `package.json`
- Create: `pnpm-lock.yaml`
- Create: `index.html`
- Create: `vite.config.ts`
- Create: `tsconfig.json`
- Create: `tsconfig.app.json`
- Create: `tsconfig.node.json`
- Create: `src/main.tsx`
- Create: `src/app/App.tsx`
- Create: `src/test/setup.ts`
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/build.rs`
- Create: `src-tauri/src/main.rs`
- Create: `src-tauri/src/lib.rs`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/capabilities/default.json`
- Modify: `.gitignore`
- Test: `src/app/App.test.tsx`

**Interfaces:**
- Consumes: approved design at `docs/superpowers/specs/2026-08-13-manga-collection-design.md`.
- Produces: `App(): JSX.Element`; scripts `pnpm test`, `pnpm build`, `pnpm desktop:dev`, `pnpm desktop:build`.

- [ ] **Step 1: Generate the official template in a temporary child directory**

Run:

```powershell
pnpm create tauri-app@latest .scaffold --template react-ts
```

Expected: `.scaffold` contains the official React TypeScript Tauri project created through pnpm. If the current generator asks for an identifier, enter `com.mangashelf.desktop`; accept desktop-only generation. Do not overwrite `docs/` or `.git/`.

- [ ] **Step 2: Move only generated project files into the repository and remove template demo code**

Use `apply_patch` for text changes after copying the generated skeleton. Preserve `docs/`. Set scripts and test configuration to:

```json
{
  "scripts": {
    "dev": "vite",
    "build": "tsc -b && vite build",
    "test": "vitest run",
    "test:watch": "vitest",
    "tauri": "tauri",
    "desktop:dev": "tauri dev",
    "desktop:build": "tauri build"
  }
}
```

Add React Testing Library, user-event, jsdom, Vitest and Lucide React. Keep TypeScript at `5.9.x` if the generator selects a newer incompatible major.

- [ ] **Step 3: Write the failing shell test**

```tsx
// src/app/App.test.tsx
import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { App } from "./App";

describe("App", () => {
  it("renders the Manga Library product identity", () => {
    render(<App />);
    expect(screen.getByRole("heading", { name: "漫畫書庫" })).toBeVisible();
  });
});
```

- [ ] **Step 4: Run the test to verify it fails**

Run: `pnpm test -- src/app/App.test.tsx`

Expected: FAIL because `src/app/App.tsx` does not export the required heading.

- [ ] **Step 5: Implement the minimal application shell**

```tsx
// src/app/App.tsx
export function App() {
  return (
    <main>
      <h1>漫畫書庫</h1>
    </main>
  );
}
```

Configure `tauri.conf.json` with product name `漫畫書庫`, identifier `com.mangashelf.desktop`, Windows size 1280 × 820, minimum 1040 × 700, system decorations enabled, NSIS current-user installer, and a non-null CSP. Allow only the application itself, Tauri's required IPC schemes, and local `asset:`／`http://asset.localhost` images; do not add Google Books or Open Library to `connect-src` because Task 6 performs provider requests in Rust.

- [ ] **Step 6: Verify the foundation**

Run:

```powershell
pnpm test -- src/app/App.test.tsx
pnpm build
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: all three commands exit 0.

- [ ] **Step 7: Commit**

```powershell
git add .gitignore package.json pnpm-lock.yaml index.html vite.config.ts tsconfig.json tsconfig.app.json tsconfig.node.json src src-tauri
git commit -m "build: scaffold Manga Library desktop app"
```

### Task 2: Define domain types, ISBN validation, and volume ordering

**Files:**
- Create: `src/domain/model.ts`
- Create: `src/domain/isbn.ts`
- Create: `src/domain/isbn.test.ts`
- Create: `src/domain/volumeOrder.ts`
- Create: `src/domain/volumeOrder.test.ts`

**Interfaces:**
- Consumes: TypeScript and Vitest from Task 1.
- Produces: `normalizeIsbn(raw): string`, `parseIsbn(raw): ParsedIsbn`, `makeVolumeSortKey(label): string`, and the domain literal types used by all later frontend tasks.

- [ ] **Step 1: Define stable domain types**

```ts
// src/domain/model.ts
export type PublicationStatus = "ongoing" | "completed" | "hiatus" | "unknown";
export type EditionFormat = "tankobon" | "omnibus" | "deluxe" | "box_set" | "other";
export type AvailabilityStatus = "released" | "upcoming" | "unknown";
export type DatePrecision = "day" | "month" | "year" | "unknown";
export type BookCondition = "new" | "like_new" | "good" | "fair" | "poor" | "unknown";
export type MetadataSource = "manual" | "google_books" | "open_library" | "ncl_import";

export interface ParsedIsbn {
  normalized: string;
  kind: "isbn10" | "isbn13";
}

export interface CollectionFlags {
  isOwned: boolean;
  isRead: boolean;
  isWishlisted: boolean;
}
```

- [ ] **Step 2: Write failing ISBN tests**

```ts
import { describe, expect, it } from "vitest";
import { parseIsbn } from "./isbn";

describe("parseIsbn", () => {
  it("normalizes a hyphenated ISBN-13", () => {
    expect(parseIsbn("978-957-26-9001-7")).toEqual({ normalized: "9789572690017", kind: "isbn13" });
  });

  it("accepts ISBN-10 ending in X", () => {
    expect(parseIsbn("0-8044-2957-X")).toEqual({ normalized: "080442957X", kind: "isbn10" });
  });

  it("rejects a bad checksum", () => {
    expect(() => parseIsbn("9789572690018")).toThrow("invalid_checksum");
  });
});
```

- [ ] **Step 3: Run ISBN tests and confirm failure**

Run: `pnpm test -- src/domain/isbn.test.ts`

Expected: FAIL because `parseIsbn` does not exist.

- [ ] **Step 4: Implement ISBN normalization and checksums**

Implement `normalizeIsbn` by removing spaces and hyphens and uppercasing `x`. ISBN-10 uses weighted sum 10 through 1 modulo 11; ISBN-13 uses alternating weights 1 and 3 with the final checksum digit.

```ts
export function parseIsbn(raw: string): ParsedIsbn {
  const normalized = normalizeIsbn(raw);
  if (normalized.length === 10 && isValidIsbn10(normalized)) return { normalized, kind: "isbn10" };
  if (normalized.length === 13 && isValidIsbn13(normalized)) return { normalized, kind: "isbn13" };
  throw new Error(/^\d{9}[\dX]$|^\d{13}$/.test(normalized) ? "invalid_checksum" : "invalid_format");
}
```

- [ ] **Step 5: Write failing natural-order tests**

```ts
import { describe, expect, it } from "vitest";
import { compareVolumeLabels, makeVolumeSortKey } from "./volumeOrder";

describe("volume ordering", () => {
  it("orders numeric, decimal, directional and one-shot labels", () => {
    const labels = ["10", "下", "1.5", "全一冊", "2", "上", "1"];
    expect(labels.sort(compareVolumeLabels)).toEqual(["1", "1.5", "2", "10", "上", "下", "全一冊"]);
  });

  it("creates equal-width numeric sort keys", () => {
    expect(makeVolumeSortKey("2")).toBe("0:000002.000");
  });
});
```

- [ ] **Step 6: Implement deterministic volume keys**

Use `0:` for numeric labels, `1:上`, `1:下`, `2:全一冊`, and `9:<normalized label>` for other labels. Numeric keys use six integer digits and three decimal digits. Reject empty labels.

- [ ] **Step 7: Run domain tests and commit**

Run: `pnpm test -- src/domain/isbn.test.ts src/domain/volumeOrder.test.ts`

Expected: PASS.

```powershell
git add src/domain
git commit -m "feat: add manga domain primitives"
```

### Task 3: Implement collection state rules and selectors

**Files:**
- Create: `src/domain/collectionRules.ts`
- Create: `src/domain/collectionRules.test.ts`
- Create: `src/test/fixtures/library.ts`

**Interfaces:**
- Consumes: `CollectionFlags`, `AvailabilityStatus` and volume ordering from Task 2.
- Produces: `applyCollectionPatch(current, patch)`, `isMissingVolume(volume)`, `summarizeEdition(volumes)`.

- [ ] **Step 1: Write failing rule tests**

```ts
it("owning a wished-for volume removes it from the wishlist", () => {
  expect(applyCollectionPatch(
    { isOwned: false, isRead: false, isWishlisted: true },
    { isOwned: true },
  )).toEqual({ isOwned: true, isRead: false, isWishlisted: false });
});

it("removing ownership preserves read state", () => {
  expect(applyCollectionPatch(
    { isOwned: true, isRead: true, isWishlisted: false },
    { isOwned: false },
  ).isRead).toBe(true);
});

it("counts only released unowned volumes as missing", () => {
  expect(summarizeEdition(tenVolumeFixture).missing).toBe(4);
  expect(isMissingVolume(upcomingUnownedFixture)).toBe(false);
  expect(isMissingVolume(unknownUnownedFixture)).toBe(false);
});
```

- [ ] **Step 2: Run tests and confirm failure**

Run: `pnpm test -- src/domain/collectionRules.test.ts`

Expected: FAIL because the rule functions do not exist.

- [ ] **Step 3: Implement pure rules**

```ts
export function applyCollectionPatch(
  current: CollectionFlags,
  patch: Partial<CollectionFlags>,
): CollectionFlags {
  const next = { ...current, ...patch };
  if (next.isOwned) next.isWishlisted = false;
  return next;
}

export function isMissingVolume(volume: VolumeWithCollection): boolean {
  return volume.availabilityStatus === "released" && !volume.collection.isOwned;
}
```

`summarizeEdition` returns `{ known, released, owned, read, wishlisted, missing }` and never derives ownership from read state.

- [ ] **Step 4: Run tests and commit**

Run: `pnpm test -- src/domain/collectionRules.test.ts`

Expected: PASS.

```powershell
git add src/domain/collectionRules.ts src/domain/collectionRules.test.ts src/test/fixtures/library.ts
git commit -m "feat: define collection state rules"
```

### Task 4: Create the SQLite schema and migration runner

**Files:**
- Create: `src-tauri/migrations/0001_initial.sql`
- Create: `src-tauri/src/error.rs`
- Create: `src-tauri/src/state.rs`
- Create: `src-tauri/src/db/mod.rs`
- Create: `src-tauri/src/db/models.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/Cargo.toml`
- Test: `src-tauri/tests/database.rs`

**Interfaces:**
- Consumes: database schema in the approved design.
- Produces: `Database::open(path)`, `Database::in_memory()`, `Database::with_transaction`, `AppError { code, message }`, and all physical tables.

- [ ] **Step 1: Write a failing migration integration test**

```rust
#[test]
fn migration_creates_schema_and_enforces_foreign_keys() {
    let db = Database::in_memory().expect("database");
    let names = db.table_names().expect("table names");
    assert!(names.contains(&"series".to_string()));
    assert!(names.contains(&"editions".to_string()));
    assert!(names.contains(&"volumes".to_string()));
    assert!(names.contains(&"collection_items".to_string()));
    assert_eq!(db.foreign_keys_enabled().unwrap(), true);
}
```

- [ ] **Step 2: Run the test and confirm failure**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test database migration_creates_schema_and_enforces_foreign_keys`

Expected: FAIL because `Database` and the migration do not exist.

- [ ] **Step 3: Write `0001_initial.sql` with exact constraints**

Create `schema_migrations`, `series`, `contributors`, `series_contributors`, `editions`, `cover_assets`, `volumes`, `collection_items`, `metadata_cache`, and `metadata_provenance`. Use `TEXT PRIMARY KEY`, `CHECK` constraints matching the spec literals, partial unique indexes for non-null ISBN values, and `ON DELETE CASCADE` from series → editions → volumes → collection items.

The collection table must include:

```sql
CREATE TABLE collection_items (
  id TEXT PRIMARY KEY NOT NULL,
  volume_id TEXT NOT NULL UNIQUE REFERENCES volumes(id) ON DELETE CASCADE,
  is_owned INTEGER NOT NULL DEFAULT 0 CHECK (is_owned IN (0, 1)),
  is_read INTEGER NOT NULL DEFAULT 0 CHECK (is_read IN (0, 1)),
  is_wishlisted INTEGER NOT NULL DEFAULT 0 CHECK (is_wishlisted IN (0, 1)),
  purchase_price_amount INTEGER CHECK (purchase_price_amount IS NULL OR purchase_price_amount >= 0),
  purchase_price_currency TEXT,
  acquired_on TEXT,
  condition TEXT NOT NULL DEFAULT 'unknown' CHECK (condition IN ('new','like_new','good','fair','poor','unknown')),
  storage_location TEXT,
  notes TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  CHECK (NOT (is_owned = 1 AND is_wishlisted = 1))
);
```

Define the command error payload once and reuse it in every backend module:

```rust
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: String,
    pub message: String,
}
```

Internal errors may add details to logs, but the serialized `message` must not reveal arbitrary file paths, SQL text or remote response bodies.

- [ ] **Step 4: Implement the migration runner**

`Database::open` enables `PRAGMA foreign_keys = ON`, creates the migration ledger, embeds migration SQL with `include_str!`, and applies each migration once inside an immediate transaction. `Database::in_memory` uses the same migration path as production.

- [ ] **Step 5: Add failing constraint and restart tests**

Test that duplicate ISBN-13 fails, normal and deluxe editions can both contain label `1`, a failed multi-row transaction leaves zero inserted rows, and reopening a temporary file preserves inserted rows.

- [ ] **Step 6: Run backend database tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test database`

Expected: all schema, constraint, transaction and restart tests PASS.

- [ ] **Step 7: Commit**

```powershell
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/migrations src-tauri/src src-tauri/tests/database.rs
git commit -m "feat: add versioned manga database"
```

### Task 5: Implement library repositories and typed Tauri commands

**Files:**
- Create: `src-tauri/src/db/repository.rs`
- Create: `src-tauri/src/commands/mod.rs`
- Create: `src-tauri/src/commands/library.rs`
- Modify: `src-tauri/src/db/models.rs`
- Modify: `src-tauri/src/db/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Create: `src/services/libraryPort.ts`
- Create: `src/services/tauriLibrary.ts`
- Create: `src/services/tauriLibrary.test.ts`
- Modify: `src/domain/model.ts`
- Test: `src-tauri/tests/database.rs`

**Interfaces:**
- Consumes: migrated `Database` from Task 4 and domain literals from Task 2.
- Produces: Rust commands `get_dashboard`, `list_series`, `get_series_detail`, `create_series_batch`, `update_collection_item`, `find_volume_by_isbn`, `delete_series`; TypeScript `LibraryPort` with matching camelCase DTOs.

- [ ] **Step 1: Define the complete TypeScript DTO and port contract before its adapter**

```ts
export interface SeriesFilter {
  query: string;
  publicationStatus: PublicationStatus | "all";
  collection: "all" | "complete" | "incomplete";
  reading: "all" | "complete" | "incomplete";
}

export interface DashboardSummary {
  seriesCount: number;
  ownedVolumeCount: number;
  readVolumeCount: number;
  missingVolumeCount: number;
  recentVolumes: VolumeWithCollection[];
  incompleteSeries: SeriesSummary[];
}

export interface SeriesSummary {
  id: string;
  title: string;
  originalTitle: string | null;
  publicationStatus: PublicationStatus;
  contributors: Array<{ name: string; role: "author" | "story" | "art" }>;
  publishers: string[];
  representativeCover: CoverAsset | null;
  knownVolumeCount: number;
  ownedVolumeCount: number;
  readVolumeCount: number;
  missingVolumeCount: number;
}

export interface CoverAsset {
  id: string;
  relativePath: string;
  mimeType: "image/jpeg" | "image/png" | "image/webp";
  byteSize: number;
  sha256: string;
}

export interface CollectionItemView extends CollectionFlags {
  purchasePriceAmount: number | null;
  purchasePriceCurrency: string | null;
  acquiredOn: string | null;
  condition: BookCondition;
  storageLocation: string | null;
  notes: string | null;
}

export interface VolumeWithCollection {
  id: string;
  editionId: string;
  displayLabel: string;
  sortKey: string;
  titleOverride: string | null;
  isbn10: string | null;
  isbn13: string | null;
  translator: string | null;
  availabilityStatus: AvailabilityStatus;
  releaseDate: string | null;
  releaseDatePrecision: DatePrecision;
  listPriceAmount: number | null;
  listPriceCurrency: string | null;
  cover: CoverAsset | null;
  collection: CollectionItemView;
}

export interface EditionDetail {
  id: string;
  name: string;
  languageCode: string;
  regionCode: string;
  publisher: string;
  format: EditionFormat;
  releaseStatus: PublicationStatus;
  knownVolumeCount: number | null;
  volumes: VolumeWithCollection[];
}

export interface SeriesDetail extends SeriesSummary {
  description: string | null;
  editions: EditionDetail[];
}

export interface CreateSeriesBatchInput {
  series: {
    title: string;
    originalTitle: string | null;
    description: string | null;
    publicationStatus: PublicationStatus;
    contributors: Array<{ name: string; role: "author" | "story" | "art"; sortOrder: number }>;
  };
  edition: {
    name: string;
    languageCode: string;
    regionCode: string;
    publisher: string;
    format: EditionFormat;
    releaseStatus: PublicationStatus;
    knownVolumeCount: number | null;
  };
  volumes: Array<{
    displayLabel: string;
    sortKey: string;
    titleOverride: string | null;
    isbn10: string | null;
    isbn13: string | null;
    translator: string | null;
    availabilityStatus: AvailabilityStatus;
    releaseDate: string | null;
    releaseDatePrecision: DatePrecision;
    listPriceAmount: number | null;
    listPriceCurrency: string | null;
    collection: CollectionItemView;
    provenance: Record<string, MetadataSource>;
  }>;
}

export type CollectionItemPatch = Partial<CollectionItemView>;

export interface BackupInfo {
  path: string;
  createdAt: string;
  sha256: string;
  byteSize: number;
}

export interface RestoreResult {
  restoredAt: string;
  preRestoreBackupPath: string;
}

export interface LibraryPort {
  getDashboard(): Promise<DashboardSummary>;
  listSeries(filter: SeriesFilter): Promise<SeriesSummary[]>;
  getSeriesDetail(seriesId: string): Promise<SeriesDetail>;
  createSeriesBatch(input: CreateSeriesBatchInput): Promise<SeriesDetail>;
  updateCollectionItem(volumeId: string, patch: CollectionItemPatch): Promise<VolumeWithCollection>;
  findVolumeByIsbn(isbn: string): Promise<VolumeWithCollection | null>;
  deleteSeries(seriesId: string): Promise<void>;
  exportBackup(destination: string): Promise<BackupInfo>;
  restoreBackup(source: string): Promise<RestoreResult>;
  importCover(sourcePath: string): Promise<CoverAsset>;
  clearUnusedCoverCache(): Promise<number>;
}
```

- [ ] **Step 2: Write failing repository tests for the ten-volume acceptance case**

Insert a 10-volume released edition with 1–6 owned in one `create_series_batch` call. Assert dashboard counts, missing labels `7`–`10`, and that updating volume 7 from wishlisted to owned makes missing equal 3 and wishlisted equal 0.

- [ ] **Step 3: Run repository tests and confirm failure**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test database repository_`

Expected: FAIL because repository functions do not exist.

- [ ] **Step 4: Implement repository transactions and DTOs**

Use a single transaction for `create_series_batch`; calculate dashboard and missing counts in SQL with `availability_status = 'released'`. Query volumes with a `LEFT JOIN collection_items`; when no row exists, return `is_owned = false`, `is_read = false`, `is_wishlisted = false` and null optional fields. Apply the same ownership rule in Rust before update and rely on the database CHECK as a second guard. `find_volume_by_isbn` accepts only normalized ISBN.

- [ ] **Step 5: Register Tauri commands**

Each command receives `State<AppState>`, maps `AppError` into `{ code, message }`, and uses `spawn_blocking` for SQLite/file operations so the UI thread is not blocked.

- [ ] **Step 6: Write and run the adapter contract test**

```ts
it("maps createSeriesBatch to the stable Tauri command", async () => {
  invokeMock.mockResolvedValue(seriesDetailFixture);
  await tauriLibrary.createSeriesBatch(batchInputFixture);
  expect(invokeMock).toHaveBeenCalledWith("create_series_batch", { input: batchInputFixture });
});
```

Run:

```powershell
pnpm test -- src/services/tauriLibrary.test.ts
cargo test --manifest-path src-tauri/Cargo.toml --test database
```

Expected: PASS.

- [ ] **Step 7: Commit**

```powershell
git add src/domain/model.ts src/services src-tauri/src src-tauri/tests/database.rs
git commit -m "feat: expose transactional library services"
```

### Task 6: Add metadata providers, normalization, fallback, and cache

**Files:**
- Create: `src/services/metadata/types.ts`
- Create: `src/services/metadata/normalize.ts`
- Create: `src/services/metadata/providers.ts`
- Create: `src/services/metadata/providers.test.ts`
- Create: `src/services/metadata/groupSeries.ts`
- Create: `src/services/metadata/groupSeries.test.ts`
- Create: `src-tauri/src/commands/metadata.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: `parseIsbn` and `MetadataSource` from Task 2; metadata cache table from Task 4.
- Produces: `MetadataProvider`, `MetadataCandidate`, `MetadataLookupError`, `MetadataCachePort`, `lookupIsbnWithFallback`, `searchSeriesWithFallback`, and Tauri commands `fetch_metadata_json`, `get_metadata_cache`, `put_metadata_cache` restricted to registered providers.

- [ ] **Step 1: Define candidate and error contracts**

```ts
export type MetadataErrorCode = "invalid_input" | "not_found" | "offline" | "rate_limited" | "provider_error";

export interface MetadataCandidate {
  source: MetadataSource;
  externalId: string;
  isbn10: string | null;
  isbn13: string | null;
  seriesTitle: string;
  originalTitle: string | null;
  volumeLabel: string | null;
  contributors: Array<{ name: string; role: "author" | "story" | "art" }>;
  publisher: string | null;
  languageCode: string | null;
  releaseDate: string | null;
  releaseDatePrecision: DatePrecision;
  description: string | null;
  coverUrl: string | null;
  warnings: string[];
  provenance: Record<string, MetadataSource>;
}

export interface MetadataCachePort {
  get(queryKey: string): Promise<MetadataCandidate[] | null>;
  put(queryKey: string, candidates: MetadataCandidate[], expiresAt: string): Promise<void>;
}

export interface MetadataProvider {
  readonly source: "google_books" | "open_library";
  lookupByIsbn(isbn: string): Promise<MetadataCandidate[]>;
  searchSeries(query: string): Promise<MetadataCandidate[]>;
}

export class MetadataLookupError extends Error {
  constructor(
    public readonly code: MetadataErrorCode,
    message: string,
    public readonly provider: MetadataSource | null = null,
  ) {
    super(message);
  }
}
```

- [ ] **Step 2: Write failing fallback tests**

Cover these exact cases:

- invalid ISBN makes zero provider calls and returns `invalid_input`;
- cached result returns before network providers;
- Google result returns without calling Open Library;
- Google `rate_limited` calls Open Library;
- both providers return no candidates and final code is `not_found`;
- raw HTML descriptions become plain text;
- malformed cover URLs are removed with a warning.

- [ ] **Step 3: Run tests and confirm failure**

Run: `pnpm test -- src/services/metadata/providers.test.ts`

Expected: FAIL because provider pipeline is absent.

- [ ] **Step 4: Implement providers and safe native fetch**

Frontend providers build fixed URLs only for `https://www.googleapis.com/books/v1/volumes` and `https://openlibrary.org/api/books`. The Rust command accepts `{ provider, pathAndQuery }`, selects a hard-coded base URL, sets timeout and user agent, caps the response at 2 MiB, and maps HTTP 429 to `rate_limited`. It must reject arbitrary schemes and hosts.

`get_metadata_cache` accepts a normalized query key and returns only unexpired JSON. `put_metadata_cache` validates that the serialized payload is at most 2 MiB, upserts source/retrieved/expiry metadata, and never stores raw response HTML. The TypeScript `MetadataCachePort` adapter maps these two commands and is injected into the fallback pipeline.

- [ ] **Step 5: Implement conservative series grouping**

Normalize punctuation and trailing numeric volume markers, but only group when normalized title plus author set, publisher, language and inferred edition are compatible. If any required evidence conflicts or is absent, return separate groups.

Test that `坂本日常 1` and `坂本日常 2` from the same publisher/author group, while `坂本日常 完全版 1` remains separate and same-title candidates with conflicting authors remain separate.

- [ ] **Step 6: Run metadata tests and backend checks**

Run:

```powershell
pnpm test -- src/services/metadata
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: PASS.

- [ ] **Step 7: Commit**

```powershell
git add src/services/metadata src-tauri/src/commands src-tauri/src/lib.rs
git commit -m "feat: add resilient manga metadata lookup"
```

### Task 7: Implement secure cover assets

**Files:**
- Create: `src-tauri/src/commands/assets.rs`
- Create: `src-tauri/tests/assets.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/services/tauriLibrary.ts`
- Create: `src/components/TextCover.tsx`
- Create: `src/components/TextCover.test.tsx`

**Interfaces:**
- Consumes: `cover_assets` schema and `LibraryPort.importCover`.
- Produces: commands `import_cover`, `cache_remote_cover`, `clear_unused_cover_cache`; safe local asset DTO; `TextCover({ title, label? })`.

- [ ] **Step 1: Write failing Rust asset validation tests**

Test rejection of non-HTTPS remote URLs, unsupported MIME, payloads larger than 8 MiB, images whose decoded dimensions exceed 6000 × 6000, and redirects beyond 3. Test that two identical images produce one SHA-256 asset and that an imported user image is copied under the app data `covers/` directory with a generated filename.

- [ ] **Step 2: Run tests and confirm failure**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test assets`

Expected: FAIL because asset commands do not exist.

- [ ] **Step 3: Implement validated cover storage**

Allow JPEG, PNG and WebP. Decode dimensions before persistence, compute SHA-256, write to a temporary sibling file, flush, then atomically rename. Save only a relative path in SQLite. Cache failure returns a typed error and never rolls back otherwise valid manga metadata.

- [ ] **Step 4: Write the failing text-cover UI test**

```tsx
it("renders an accessible fallback without a broken image", () => {
  render(<TextCover title="藍色監獄" label="31" />);
  expect(screen.getByRole("img", { name: "藍色監獄 第 31 冊文字封面" })).toBeVisible();
  expect(screen.queryByRole("img", { name: /載入失敗/ })).not.toBeInTheDocument();
});
```

- [ ] **Step 5: Implement `TextCover` and verify**

Use a semantic element with `role="img"`, deterministic initials/title layout and no network dependency.

Run:

```powershell
pnpm test -- src/components/TextCover.test.tsx
cargo test --manifest-path src-tauri/Cargo.toml --test assets
```

Expected: PASS.

- [ ] **Step 6: Commit**

```powershell
git add src/components/TextCover* src/services/tauriLibrary.ts src-tauri/src src-tauri/tests/assets.rs
git commit -m "feat: secure manga cover assets"
```

### Task 8: Build the application shell, theme, and accessibility primitives

**Files:**
- Create: `src/app/navigation.ts`
- Create: `src/app/LibraryContext.tsx`
- Create: `src/components/AppShell.tsx`
- Create: `src/components/AppShell.test.tsx`
- Create: `src/components/StatusBadge.tsx`
- Create: `src/components/SaveStatus.tsx`
- Create: `src/components/EmptyState.tsx`
- Create: `src/components/ErrorNotice.tsx`
- Create: `src/components/ConfirmDialog.tsx`
- Create: `src/styles/tokens.css`
- Create: `src/styles/global.css`
- Create: `src-tauri/app-icon.svg`
- Modify: `src/app/App.tsx`
- Modify: `src/main.tsx`

**Interfaces:**
- Consumes: `LibraryPort`; product navigation from the spec.
- Produces: `NavigationTarget = "dashboard" | "library" | "wishlist" | "settings"`; accessible shell and shared status components.

- [ ] **Step 1: Write failing navigation and shortcut tests**

```tsx
it("offers exactly four primary destinations", () => {
  renderApp();
  expect(screen.getAllByRole("link")).toHaveLength(4);
  expect(screen.getByRole("link", { name: "收藏總覽" })).toBeVisible();
  expect(screen.getByRole("link", { name: "我的書庫" })).toBeVisible();
  expect(screen.getByRole("link", { name: "待補清單" })).toBeVisible();
  expect(screen.getByRole("link", { name: "設定" })).toBeVisible();
});

it("opens add manga with Ctrl+N and focuses search with Ctrl+K", async () => {
  const user = userEvent.setup();
  renderApp();
  await user.keyboard("{Control>}n{/Control}");
  expect(screen.getByRole("dialog", { name: "新增漫畫" })).toBeVisible();
  await user.keyboard("{Escape}");
  await user.keyboard("{Control>}k{/Control}");
  expect(screen.getByRole("searchbox")).toHaveFocus();
});
```

- [ ] **Step 2: Run tests and confirm failure**

Run: `pnpm test -- src/components/AppShell.test.tsx`

Expected: FAIL because shell and shortcuts are absent.

- [ ] **Step 3: Implement theme tokens and shell**

Use exact tokens:

```css
:root {
  --paper: #f4f0e7;
  --paper-strong: #fffdf8;
  --graphite: #20201f;
  --ink: #242321;
  --muted: #716d66;
  --vermilion: #b64232;
  --line: #d8d0c3;
  --focus: #1769aa;
  --radius-sm: 8px;
  --radius-md: 14px;
  --motion-fast: 160ms;
}
```

Add `@media (prefers-reduced-motion: reduce)` to disable non-essential transitions. Focus rings use at least 2 px `--focus`. Modal focus is trapped, initial focus is intentional, and `Esc` closes only the topmost transient UI.

Create a square SVG icon with a graphite background, a warm-paper vertical book shape and one restrained vermilion spine; it must remain legible without text at 32 × 32. Generate the platform icon set with `pnpm tauri icon src-tauri/app-icon.svg` and reference the resulting Windows `.ico` in the bundle configuration.

- [ ] **Step 4: Run shell tests and build**

Run:

```powershell
pnpm test -- src/components/AppShell.test.tsx
pnpm build
```

Expected: PASS.

- [ ] **Step 5: Commit**

```powershell
git add src/app src/components src/styles src/main.tsx
git commit -m "feat: add accessible Manga Library shell"
```

### Task 9: Implement dashboard, library, and series detail

**Files:**
- Create: `src/test/fakes/fakeLibrary.ts`
- Create: `src/components/SeriesCard.tsx`
- Create: `src/components/VolumeCard.tsx`
- Create: `src/features/dashboard/DashboardPage.tsx`
- Create: `src/features/dashboard/DashboardPage.test.tsx`
- Create: `src/features/library/LibraryPage.tsx`
- Create: `src/features/library/LibraryPage.test.tsx`
- Create: `src/features/library/SeriesDetailPage.tsx`
- Create: `src/features/library/SeriesDetailPage.test.tsx`
- Modify: `src/app/App.tsx`

**Interfaces:**
- Consumes: `LibraryPort.getDashboard`, `listSeries`, `getSeriesDetail`, `updateCollectionItem`; cards and rules from earlier tasks.
- Produces: the three core read/browse experiences and batch collection editing.

- [ ] **Step 1: Build the controllable fake library**

`FakeLibrary` stores fixtures in memory, records every call, can delay a call, and can reject the next call with `{ code, message }`. It implements every `LibraryPort` method so UI tests never invoke Tauri.

- [ ] **Step 2: Write failing dashboard tests**

Assert the sentence `2 個系列・6 本收藏・缺少 4 本`, recent additions, incomplete series, loading skeleton, empty state, and retry after a controlled repository error.

- [ ] **Step 3: Implement dashboard and verify**

Run: `pnpm test -- src/features/dashboard/DashboardPage.test.tsx`

Expected: PASS.

- [ ] **Step 4: Write failing library tests**

Assert title/author/publisher search, publication status filter, collection completion filter, reading completion filter, series cover grid, text-cover fallback, and no-results reset action.

- [ ] **Step 5: Implement library and verify**

Filtering inputs map to `SeriesFilter`; do not reimplement SQL matching rules in components. Keep series cards limited to cover, title, owned/known counts and status.

Run: `pnpm test -- src/features/library/LibraryPage.test.tsx`

Expected: PASS.

- [ ] **Step 6: Write failing series-detail and batch tests**

Cover version switching, normal/deluxe isolation, full-color owned cards, desaturated unowned cards, read and wishlist labels, entering batch mode, marking 1–6 owned in one call, and `Esc` exiting batch mode without saving unsubmitted changes.

- [ ] **Step 7: Implement series detail and verify**

Use an explicit batch toolbar. Do not render persistent checkboxes outside batch mode. Announce save results with an `aria-live="polite"` region.

Run:

```powershell
pnpm test -- src/features/dashboard src/features/library
pnpm build
```

Expected: PASS.

- [ ] **Step 8: Commit**

```powershell
git add src/components/SeriesCard.tsx src/components/VolumeCard.tsx src/features/dashboard src/features/library src/test/fakes
git commit -m "feat: browse manga collection by series"
```

### Task 10: Implement both add-manga workflows

**Files:**
- Create: `src/features/add/AddMangaDialog.tsx`
- Create: `src/features/add/AddMangaDialog.test.tsx`
- Create: `src/features/add/SeriesBatchFlow.tsx`
- Create: `src/features/add/SeriesBatchFlow.test.tsx`
- Create: `src/features/add/IsbnFlow.tsx`
- Create: `src/features/add/IsbnFlow.test.tsx`
- Modify: `src/app/App.tsx`

**Interfaces:**
- Consumes: metadata pipeline, ISBN parser, series grouping, `LibraryPort.findVolumeByIsbn`, `createSeriesBatch`, and cover cache.
- Produces: global `AddMangaDialog` with `mode = "series" | "isbn"`; complete manual fallback using the same `CreateSeriesBatchInput`.

- [ ] **Step 1: Write failing entry-dialog tests**

Assert only two primary choices on the first screen: `搜尋作品` and `輸入 ISBN`; manual creation remains available as a lower-emphasis action. Assert closing preserves nothing only after an explicit cancel confirmation when form data is dirty.

- [ ] **Step 2: Implement entry dialog and verify**

Run: `pnpm test -- src/features/add/AddMangaDialog.test.tsx`

Expected: PASS.

- [ ] **Step 3: Write failing series-batch flow tests**

Cover search loading/error/empty results, conservative groups, selecting or creating a series, selecting/creating edition, generating 10 released volume rows, batch marking 1–6 owned, optionally marking read, and one `createSeriesBatch` call on confirmation.

- [ ] **Step 4: Implement series-batch flow and verify**

When metadata lacks volume coverage, accept known count and generate labels `1` through `N` with `availabilityStatus = "released"` only after the user confirms they are already published. Allow editing every generated label before save.

Run: `pnpm test -- src/features/add/SeriesBatchFlow.test.tsx`

Expected: PASS.

- [ ] **Step 5: Write failing ISBN-flow tests**

Cover valid hyphenated ISBN, invalid checksum with zero network calls, duplicate opening existing item, Google success, Google 429 + Open Library success, both providers empty, offline retry, preserved input, editable candidate, and complete manual save.

- [ ] **Step 6: Implement ISBN flow and verify**

The flow order is local validation → duplicate lookup → cached/provider lookup → candidate confirmation → save → non-blocking cover cache. Save metadata even if the cover cache fails, then show a recoverable cover warning.

Run:

```powershell
pnpm test -- src/features/add
pnpm build
```

Expected: PASS.

- [ ] **Step 7: Commit**

```powershell
git add src/features/add src/app/App.tsx
git commit -m "feat: add batch and ISBN manga intake"
```

### Task 11: Implement volume details and wishlist conversion

**Files:**
- Create: `src/features/volume/VolumeDetailDialog.tsx`
- Create: `src/features/volume/VolumeDetailDialog.test.tsx`
- Create: `src/features/wishlist/WishlistPage.tsx`
- Create: `src/features/wishlist/WishlistPage.test.tsx`
- Modify: `src/features/library/SeriesDetailPage.tsx`
- Modify: `src/app/App.tsx`

**Interfaces:**
- Consumes: `LibraryPort.updateCollectionItem`, derived missing rules, `SaveStatus`.
- Produces: editable purchase/condition/location details and missing/wishlist actions.

- [ ] **Step 1: Write failing volume-detail tests**

Test independent owned/read toggles, wishlisted → owned conversion, price stored as integer TWD, optional acquired date, each condition literal, storage location, notes, saving/saved/error UI, and retained form after failure.

- [ ] **Step 2: Implement details dialog and verify**

Convert displayed decimal currency to integer minor units before calling the port. For TWD, reject fractional input. Never show saved until the resolved DTO returns.

Run: `pnpm test -- src/features/volume/VolumeDetailDialog.test.tsx`

Expected: PASS.

- [ ] **Step 3: Write failing wishlist tests**

Assert released unowned volumes appear as missing, upcoming and unknown do not; wishlist is visually distinct; `加入想買` updates one item; `已購買` sets owned and clears wishlist; optional purchase fields appear before purchase confirmation.

- [ ] **Step 4: Implement wishlist and verify**

Render groups by series and edition. Use the exact labels `缺少`, `想買`, `即將出版`; do not call unknown availability missing.

Run:

```powershell
pnpm test -- src/features/volume src/features/wishlist
pnpm build
```

Expected: PASS.

- [ ] **Step 5: Commit**

```powershell
git add src/features/volume src/features/wishlist src/features/library/SeriesDetailPage.tsx src/app/App.tsx
git commit -m "feat: manage volume details and wishlist"
```

### Task 12: Implement backup, restore, retention, and settings

**Files:**
- Create: `src-tauri/src/commands/backup.rs`
- Create: `src-tauri/tests/backup.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/services/libraryPort.ts`
- Modify: `src/services/tauriLibrary.ts`
- Create: `src/features/settings/SettingsPage.tsx`
- Create: `src/features/settings/SettingsPage.test.tsx`
- Create: `src/components/ConfirmDialog.tsx`

**Interfaces:**
- Consumes: production database path, covers directory, library adapter, Tauri dialog plugin.
- Produces: `.mangashelf-backup` ZIP v1, `BackupInfo`, `RestoreResult`, automatic retention and settings controls.

- [ ] **Step 1: Add the official Tauri dialog plugin**

Run:

```powershell
pnpm add @tauri-apps/plugin-dialog@^2
cargo add tauri-plugin-dialog@2 --manifest-path src-tauri/Cargo.toml
```

Register `tauri_plugin_dialog::init()` and grant only open/save dialog permissions to the main window capability.

- [ ] **Step 2: Define and test backup manifest**

```rust
#[derive(Serialize, Deserialize)]
struct BackupManifest {
    format: String,          // "mangashelf-backup"
    version: u32,            // 1
    created_at: String,
    database_sha256: String,
    assets: Vec<BackupAsset>,
}

#[derive(Serialize, Deserialize)]
struct BackupAsset {
    relative_path: String,
    sha256: String,
    byte_size: u64,
}
```

Write failing tests for export/import round trip, manifest version rejection, database hash mismatch, asset hash mismatch, corrupt ZIP, and failed restore preserving the live database.

- [ ] **Step 3: Run backup tests and confirm failure**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test backup`

Expected: FAIL because backup commands are absent.

- [ ] **Step 4: Implement consistent export and atomic restore**

Create the database snapshot with SQLite backup API while holding the database coordination lock. Write ZIP to a temporary destination, hash database/assets, then rename. Restore extracts to a new temporary directory, validates manifest and hashes, runs `PRAGMA integrity_check`, creates a pre-restore backup, closes database handles, swaps files atomically, reopens the database, and rolls back the swap on error.

- [ ] **Step 5: Implement automatic retention**

Store `last_successful_backup_at` and `last_data_change_at` in a settings table added through `0002_backup_settings.sql`. Create automatic backup only when data changed and 24 hours elapsed. Sort automatic backups by manifest timestamp and retain the newest 7; never delete manual exports.

- [ ] **Step 6: Write failing settings UI tests**

Cover version display from Tauri app version, export destination selection, successful export, canceled dialog, restore confirmation, restore success requiring app refresh, restore failure, and clear-unused-cover-cache count.

- [ ] **Step 7: Implement settings UI and verify**

Use Tauri dialog plugin only for user-selected source/destination. Restore confirmation states that current data will be replaced and a pre-restore backup will be created.

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --test backup
pnpm test -- src/features/settings/SettingsPage.test.tsx
```

Expected: PASS.

- [ ] **Step 8: Commit**

```powershell
git add src-tauri/migrations/0002_backup_settings.sql src-tauri/src src-tauri/tests/backup.rs src/services src/features/settings src/components/ConfirmDialog.tsx
git commit -m "feat: add verified backup and restore"
```

### Task 13: Harden configuration and complete automated acceptance coverage

**Files:**
- Modify: `src-tauri/tauri.conf.json`
- Modify: `src-tauri/capabilities/default.json`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/lib.rs`
- Create: `src/app/App.acceptance.test.tsx`
- Create: `tests/manual/windows-release-checklist.md`
- Create: `README.md`

**Interfaces:**
- Consumes: all previous tasks.
- Produces: fully configured Windows MVP, automated acceptance suite, truthful manual verification checklist, and local usage documentation.

- [ ] **Step 1: Write the end-to-end component acceptance test**

Using `FakeLibrary`, execute in one test suite:

1. create 10 released volumes and mark 1–6 owned;
2. observe missing 7–10;
3. wishlist 7 then mark it owned;
4. mark an unowned volume read and confirm ownership stays false;
5. add by ISBN and surface duplicate on second attempt;
6. simulate Google 429/Open Library empty and complete manual add;
7. show `1.5`, `上`, `下`, `全一冊` in correct order;
8. switch between ordinary and deluxe editions;
9. edit price/date/condition/location;
10. show text cover after image failure.

- [ ] **Step 2: Run the acceptance test and fix only uncovered integration seams**

Run: `pnpm test -- src/app/App.acceptance.test.tsx`

Expected: PASS without changing domain rules established in earlier tasks.

- [ ] **Step 3: Harden Tauri configuration**

Run `cargo add tauri-plugin-single-instance@2 --manifest-path src-tauri/Cargo.toml`, register the plugin before other plugins, and focus/unminimize the existing `main` window when a second process launches. Ensure the main window is the only default window, capabilities allow only core event/window, dialog selection and the registered commands, and CSP has no wildcard sources, `unsafe-eval`, or arbitrary external `connect-src`.

- [ ] **Step 4: Write the Windows release checklist**

The checklist contains dated result fields for:

- NSIS install and uninstall;
- Windows「已安裝的應用程式」中能找到並解除安裝「漫畫書庫」；
- launch under the target Windows account;
- 1280 × 820 initial size and 1040 × 700 minimum;
- Windows scale factors used on the target machine;
- `Ctrl+K`, `Ctrl+N`, `Esc`, tab order and visible focus;
- ordinary keyboard and USB scanner ISBN entry;
- close/reopen persistence;
- real export, mutation and restore round trip;
- offline add and cover failure;
- measured time until first usable UI;
- measured or profiled frame stability for primary page transitions and a representative cover-grid scroll, with the 60 FPS target recorded honestly;
- animation inspection with reduced motion enabled and disabled.

Unchecked items must remain explicitly `未驗證`; do not pre-fill passes.

- [ ] **Step 5: Document local development and data safety**

`README.md` includes prerequisites, `pnpm install`, `pnpm test`, `pnpm build`, `cargo test`, `pnpm desktop:dev`, `pnpm desktop:build`, local-first scope, application data location discovery through Tauri, and a warning not to edit SQLite while the app is running.

- [ ] **Step 6: Run the full automated verification suite**

Run:

```powershell
pnpm test
pnpm build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
pnpm desktop:build
```

Expected: every command exits 0 and produces an NSIS installer. If `desktop:build` is blocked by missing Windows tooling or network access, record the exact blocker and do not claim the installer is verified.

- [ ] **Step 7: Commit**

```powershell
git add README.md src src-tauri tests/manual/windows-release-checklist.md
git commit -m "test: complete Manga Library MVP acceptance"
```

### Task 14: Perform the Windows release verification and completion audit

**Files:**
- Modify: `tests/manual/windows-release-checklist.md`
- Create: `artifacts/release-manifest.json`
- Create: `scripts/New-ReleaseManifest.ps1`

**Interfaces:**
- Consumes: NSIS installer and all 13 acceptance requirements.
- Produces: current-state evidence for every explicit MVP requirement; no requirement is considered met from source code alone when runtime behavior is required.

- [ ] **Step 1: Install the generated NSIS bundle on the target Windows machine**

Record installer filename, product/file version, byte size and SHA-256. Install for current user and verify Start Menu entry, taskbar behavior and normal launch.

- [ ] **Step 2: Execute every manual checklist item**

Enter an actual observation, timestamp and pass/fail for each item. Use at least one real Taiwan manga ISBN and one intentionally unknown valid ISBN. Do not convert a skipped item into a pass.

- [ ] **Step 3: Verify persistence and backup with real files**

Create a 10-volume series, close the app, reopen it, export `.mangashelf-backup`, change volume 7, restore, and verify the pre-export state. Check that a pre-restore backup exists.

- [ ] **Step 4: Add and run a deterministic release-manifest script**

Create the script with `apply_patch` using this exact behavior:

```powershell
param(
  [Parameter(Mandatory = $true)][string]$InstallerPath,
  [Parameter(Mandatory = $true)][string]$OutputPath
)

$resolvedInstaller = (Resolve-Path -LiteralPath $InstallerPath).Path
$installerItem = Get-Item -LiteralPath $resolvedInstaller
$digest = (Get-FileHash -Algorithm SHA256 -LiteralPath $resolvedInstaller).Hash.ToLowerInvariant()
$manifest = [ordered]@{
  product = '漫畫書庫'
  version = '0.1.0'
  installer = $installerItem.Name
  installerBytes = $installerItem.Length
  sha256 = $digest
  verifiedAt = (Get-Date).ToUniversalTime().ToString('o')
  automated = [ordered]@{
    pnpmTest = $true
    pnpmBuild = $true
    cargoClippy = $true
    cargoTest = $true
    desktopBuild = $true
  }
  manualChecklist = 'tests/manual/windows-release-checklist.md'
}

$json = $manifest | ConvertTo-Json -Depth 4
[System.IO.File]::WriteAllText($OutputPath, $json + [Environment]::NewLine, [System.Text.UTF8Encoding]::new($false))
```

Run it only after all automated flags and the manual checklist are genuinely successful:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts\New-ReleaseManifest.ps1 -InstallerPath src-tauri\target\release\bundle\nsis\漫畫書庫_0.1.0_x64-setup.exe -OutputPath artifacts\release-manifest.json
```

Expected: the output contains the observed installer filename, byte size, SHA-256 and current UTC timestamp. Do not run the script or create the manifest if the installer was not built or any recorded boolean would be false.

- [ ] **Step 5: Run the requirement-by-requirement completion audit**

Map each of the 13 minimum acceptance items in the design spec to one or more automated test names plus the exact manual checklist evidence where required. Mark missing or indirect evidence as incomplete and fix/retest before proceeding.

- [ ] **Step 6: Commit verified release evidence**

```powershell
git add tests/manual/windows-release-checklist.md artifacts/release-manifest.json scripts/New-ReleaseManifest.ps1
git commit -m "chore: verify Manga Library Windows release"
```

## Definition of Done

- All 14 tasks are committed in order and the working tree is clean.
- `pnpm test`, `pnpm build`, `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`, and `pnpm desktop:build` pass on the current checkout.
- The NSIS installer is installed and exercised on the target Windows machine.
- Every one of the 13 minimum acceptance requirements has direct evidence.
- Any test or manual check that was not run remains reported as unverified; no inferred pass is accepted.
- The application contains none of the explicitly excluded features from the design spec.
