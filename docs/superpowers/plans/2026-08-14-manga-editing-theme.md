# 漫畫資料編輯、封面與主題實作計畫

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 讓使用者編輯既有漫畫資料、替連載系列新增冊數、使用本機真實封面，並套用已核准的現代漫畫封面主題。

**Architecture:** Rust repository 以 transaction 提供 `update_series_metadata` 與 `add_volume`，專用 cover command 驗證並保存圖片；TypeScript `LibraryPort` 映射 command，React 使用獨立編輯／新增冊數畫面。視覺只調整現有元件與 CSS，不改變資料語意。

**Tech Stack:** React 19、TypeScript 5.9、Vitest、Tauri 2、Rust、rusqlite、SQLite、NSIS。

## Global Constraints

- Windows 桌面、local-first；不得上傳漫畫資料或封面。
- 現有 series／edition／volume／collection IDs 與收藏狀態不得因編輯而改變。
- 封面只接受實際 JPG、PNG、WebP，最大 10 MiB；不信任副檔名。
- 新增冊數可使用數字、小數或文字標籤；不得刪除既有冊數。
- 每項 production 行為先取得正確 RED，再加入最小實作。

---

### Task 1: 系列基本資料編輯與新增冊數

**Files:**
- Modify: `src/domain/model.ts`
- Modify: `src/services/libraryPort.ts`
- Modify: `src/services/tauriLibrary.ts`
- Modify: `src/services/tauriLibrary.test.ts`
- Modify: `src-tauri/src/db/models.rs`
- Modify: `src-tauri/src/db/repository.rs`
- Modify: `src-tauri/src/commands/library.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/tests/database.rs`

**Interfaces:**
- Produces `UpdateSeriesMetadataInput`, `AddVolumeInput`, `updateSeriesMetadata(seriesId, input)`, and `addVolume(editionId, input)`.
- Rust commands use camelCase envelopes `{ seriesId, input }` and `{ editionId, input }`.

- [ ] Write repository tests proving metadata edits preserve all IDs/statuses, failures roll back, numeric/special volumes sort correctly, duplicate labels/ISBNs fail, and owned closes wishlist.
- [ ] Run `cargo test --manifest-path src-tauri/Cargo.toml --test database repository_update_series repository_add_volume`; expect missing APIs.
- [ ] Implement transaction-scoped repository functions and commands with non-empty validation and normalized/checksummed ISBN.
- [ ] Run targeted and full Cargo tests, check, and fmt.
- [ ] Add adapter tests for exact commands/envelopes/returns; run RED, implement two `LibraryPort` methods, then run targeted Vitest and TypeScript check.
- [ ] Commit `feat: edit manga metadata and add volumes`.

---

### Task 2: 安全的本機系列封面

**Files:**
- Modify: `src/domain/model.ts`
- Modify: `src/services/libraryPort.ts`
- Modify: `src/services/tauriLibrary.ts`
- Modify: `src/services/tauriLibrary.test.ts`
- Create: `src/services/coverPicker.ts`
- Create: `src/services/coverPicker.test.ts`
- Create: `src-tauri/src/commands/cover.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/db/repository.rs`
- Modify: `src-tauri/src/db/models.rs`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/Cargo.lock`
- Modify: `src-tauri/tauri.conf.json`
- Modify: `src-tauri/capabilities/default.json`
- Modify: `package.json`
- Modify: `pnpm-lock.yaml`
- Create: `src-tauri/tests/cover.rs`

**Interfaces:**
- Produces `selectCoverImage()`, `setSeriesCover(seriesId, sourcePath)`, and `removeSeriesCover(seriesId)`.
- Stored `CoverAsset.relativePath` remains relative to app data; frontend URL conversion receives only app-managed cover paths.

- [ ] Write Rust tests for magic bytes, 10 MiB boundary, empty/forged files, deduplication, failure cleanup, replacement/removal, and referenced-asset safety.
- [ ] Run cover RED; add minimal hashing/image validation, transactional asset binding, and safe cleanup.
- [ ] Write picker/adapter RED tests; add official dialog plugin with JPG/JPEG/PNG/WebP single-select filter and exact command mappings.
- [ ] Configure asset protocol scope to app-data `covers/**` only; preserve existing CSP.
- [ ] Run targeted/full frontend and Rust suites plus build.
- [ ] Commit `feat: manage local manga covers`.

---

### Task 3: 編輯畫面、新增冊數畫面與漫畫主題

**Files:**
- Create: `src/components/SeriesCover.tsx`
- Create: `src/components/SeriesCover.test.tsx`
- Create: `src/components/EditSeriesView.tsx`
- Create: `src/components/EditSeriesView.test.tsx`
- Create: `src/components/AddVolumeView.tsx`
- Create: `src/components/AddVolumeView.test.tsx`
- Modify: `src/components/TextCover.tsx`
- Modify: `src/app/App.tsx`
- Modify: `src/app/App.css`
- Modify: `src/app/App.test.tsx`

**Interfaces:**
- `SeriesCover` renders a real app-managed image and falls back to `TextCover` on missing/error.
- `EditSeriesView` saves one metadata request and optional cover set/remove action; cancel performs no mutation.
- `AddVolumeView` derives the suggested next integer from existing labels and submits one `AddVolumeInput`.

- [ ] Write component RED tests for real/fallback covers, edit prefill/cancel/save/error, cover preview/remove, next-volume suggestion, special labels, ISBN errors, and owned/wishlist mutual exclusion.
- [ ] Implement the three focused components and mount `edit`/`addVolume` screens from series detail.
- [ ] Add integration tests proving edited details refresh and a new volume updates detail/dashboard without changing existing collection state.
- [ ] Replace the dark content palette with approved paper/ink/red tokens, cover-first cards, hard print shadows, readable forms, and responsive desktop layout.
- [ ] Run full Vitest, TypeScript/Vite build, full Cargo tests/check/fmt, and signed NSIS debug build.
- [ ] Install with user confirmation; verify real edit, cover persistence after moving the source, added volume, restart persistence, desktop shortcut, and unchanged prior records.
- [ ] Commit `feat: deliver manga editing and cover theme`.

---

### Task 4: 回到發布與後續功能規劃

**Files:**
- Resume: `docs/superpowers/plans/2026-08-13-desktop-auto-update.md`

- [ ] Complete GitHub Release workflow and signed `0.2.0` publication.
- [ ] Prove in-app `0.2.0 -> 0.2.1` update with retained data and desktop shortcut.
- [ ] After the theme is installed and accepted, inspect the real UI and propose settings/other library features one decision at a time; do not implement them without separate approval.
