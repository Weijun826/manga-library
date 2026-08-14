# Volume Collection Editor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在系列詳情頁以右側抽屜編輯既有卷冊的書目與收藏資料，並永久保存到 SQLite。

**Architecture:** 前端以一個 `updateVolumeDetails` 呼叫送出完整表單；Tauri command 轉交 repository；repository 在單一 transaction 更新 `volumes` 與 `collection_items`。成功後 App 刷新系列、書庫與首頁資料。

**Tech Stack:** React 19、TypeScript 5.9、Vitest、Tauri 2、Rust、rusqlite／SQLite。

## Global Constraints

- Windows-only，不新增登入、同步、網路書目查詢、刪除卷冊或批次編輯。
- 價格為選填非負整數，幣別固定 TWD；日期格式為有效 `YYYY-MM-DD`。
- ISBN 必須正規化、通過檢查碼，且不得與其他卷冊重複。
- 同版本卷數標籤不得重複；`isOwned=true` 強制 `isWishlisted=false`。
- 所有資料更新必須在單一 SQLite transaction 內完成。

---

### Task 1: Rust 原子更新

**Files:**
- Modify: `src-tauri/src/db/models.rs`
- Modify: `src-tauri/src/db/repository.rs`
- Modify: `src-tauri/src/commands/library.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/tests/database.rs`

**Interfaces:**
- Produces: `repository::update_volume_details(&Database, &str, UpdateVolumeDetailsInput) -> Result<VolumeWithCollection, AppError>`
- Produces: Tauri command `update_volume_details(volume_id, input)`

- [ ] **Step 1: 先寫失敗的 repository tests**

測試成功更新所有欄位，並驗證：ISBN／標籤重複、無效日期、負價格會拒絕；更新第二張表失敗時第一張表也回滾；擁有會清除願望。

核心 input：

```rust
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateVolumeDetailsInput {
    pub display_label: String,
    pub isbn: Option<String>,
    pub availability_status: AvailabilityStatus,
    pub collection: CollectionItemView,
}
```

- [ ] **Step 2: 確認 RED**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test database update_volume_details`

Expected: FAIL，因 production API 尚不存在。

- [ ] **Step 3: 實作最小 transaction 與 command**

沿用現有 `normalize_isbn`、`make_volume_sort_key`、`with_transaction`、`enforce_ownership_rule`；唯一性 SQL 必須含 `id <> ?`。新增 `validate_iso_date` 檢查四位年、月份天數與閏年，空白字串轉 `None`。價格存在時 repository 強制幣別為 `TWD`；價格空白時金額與幣別都存為 `None`。

- [ ] **Step 4: 確認 GREEN 並提交**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test database update_volume_details`

Run: `cargo test --manifest-path src-tauri/Cargo.toml`

Commit: `feat: update volume collection details atomically`

---

### Task 2: 前端介面與右側抽屜

**Files:**
- Modify: `src/domain/model.ts`
- Modify: `src/services/libraryPort.ts`
- Modify: `src/services/tauriLibrary.ts`
- Modify: `src/services/tauriLibrary.test.ts`
- Create: `src/components/VolumeEditorDrawer.tsx`
- Create: `src/components/VolumeEditorDrawer.test.tsx`
- Modify: `src/app/App.tsx`
- Modify: `src/app/App.css`
- Modify: `src/app/App.test.tsx`

**Interfaces:**
- Consumes: Rust command `update_volume_details`
- Produces:

```ts
export interface UpdateVolumeDetailsInput {
  displayLabel: string;
  isbn: string | null;
  availabilityStatus: AvailabilityStatus;
  collection: CollectionItemView;
}

updateVolumeDetails(
  volumeId: string,
  input: UpdateVolumeDetailsInput,
): Promise<VolumeWithCollection>;
```

- [ ] **Step 1: 先寫失敗的 adapter、component 與 App tests**

必測：`invoke("update_volume_details", { volumeId, input })`；點卷冊開啟並預填；完整欄位送出；擁有與願望互斥；無效日期／價格；未儲存關閉確認；失敗時保留輸入；成功後刷新資料。

- [ ] **Step 2: 確認 RED**

Run: `pnpm test -- src/services/tauriLibrary.test.ts src/components/VolumeEditorDrawer.test.tsx src/app/App.test.tsx`

Expected: FAIL，因新 interface／drawer／method 尚不存在。

- [ ] **Step 3: 實作最小 UI**

`VolumeEditorDrawer` 自行管理表單與錯誤；App 只管理選取卷冊、呼叫 service、關閉抽屜與刷新。卷冊列保留既有快速切換按鈕，另提供清楚的「編輯資料」按鈕，避免點擊區域與 toggle 衝突。

- [ ] **Step 4: 確認 GREEN 並提交**

Run: `pnpm test -- src/services/tauriLibrary.test.ts src/components/VolumeEditorDrawer.test.tsx src/app/App.test.tsx`

Run: `pnpm exec tsc -b && pnpm build`

Commit: `feat: edit volume collection details`

---

### Task 3: 完整驗證、安裝與更新版本

**Files:**
- Modify: `package.json`
- Modify: `src-tauri/tauri.conf.json`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/Cargo.lock`

- [ ] **Step 1: 完整驗證**

Run: `pnpm test`

Run: `pnpm exec tsc -b && pnpm build`

Run: `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`

Run: `cargo test --manifest-path src-tauri/Cargo.toml`

Run: `cargo check --manifest-path src-tauri/Cargo.toml`

- [ ] **Step 2: 建置並覆蓋目前安裝程式**

先關閉 `manga-shelf.exe`，執行 Tauri release build；保留目前資料庫，不重建或刪除 `%APPDATA%\com.mangashelf.desktop\library.sqlite3`。將新 exe 安裝／覆蓋至 `%LOCALAPPDATA%\漫畫書庫`，確認桌面捷徑可直接啟動且關閉 PowerShell 不影響程式。

- [ ] **Step 3: 發布可自動更新的 0.3.0**

將 `package.json`、`tauri.conf.json`、`Cargo.toml` 與 lockfile 的 app package 版本由 `0.2.2` 同步改為 `0.3.0`。提交並推送 `v0.3.0` tag，讓既有 GitHub workflow 建立 signed NSIS 與 `latest.json`；確認 release installer、signature、manifest 都可下載後，再用已安裝的 `0.2.2` 執行一次站內更新驗收。

Commit: `chore: release volume editor update`

完成條件：實際安裝版可編輯卷冊、重開後資料仍存在，且完整前後端測試與 build 全部通過。
