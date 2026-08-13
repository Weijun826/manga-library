# 漫畫書庫桌面捷徑與自動更新實作計畫

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 發布可從桌面啟動、更新前備份資料，並能從 GitHub Releases 在程式內更新的 Windows 漫畫書庫 `0.2.0`。

**Architecture:** NSIS hook 建立目前使用者桌面捷徑；Rust command 以 SQLite online backup API 建立更新前備份；可注入的 TypeScript update port 封裝 Tauri updater/process API，React 元件負責呈現狀態。公開儲存庫 `Weijun826/manga-library` 的 GitHub Actions 只在全部測試通過後建置、簽章與發布 NSIS updater artifact。

**Tech Stack:** React 19、TypeScript 5.9、Vitest 3、Tauri 2、Rust、rusqlite 0.37、NSIS、GitHub Actions、GitHub Releases。

## Global Constraints

- Windows x86_64、Tauri 2、NSIS `currentUser`，不新增 macOS、行動版或 Web 發布。
- `0.1.0` 必須最後一次人工安裝 updater-enabled `0.2.0`；`0.2.0` 之後才使用程式內更新。
- SQLite 資料仍位於 Tauri app data 目錄，更新不得刪除、移動或重建現有資料庫。
- Tauri updater 私鑰不得寫入 Git、程式碼、建置產物或日誌。
- 更新前備份失敗時不得下載或安裝更新。
- 啟動檢查失敗不得阻止書庫使用；不得未經使用者按下「立即更新」便關閉應用程式。
- 更新端點固定為 `https://github.com/Weijun826/manga-library/releases/latest/download/latest.json`。

---

### Task 1: Desktop shortcut and installer contract

**Files:**
- Create: `src-tauri/windows/hooks.nsh`
- Modify: `src-tauri/tauri.conf.json`

**Interfaces:**
- Consumes: Tauri NSIS `NSIS_HOOK_POSTINSTALL` and `NSIS_HOOK_POSTUNINSTALL`.
- Produces: `%USERPROFILE%\\Desktop\\漫畫書庫.lnk` pointing to `$INSTDIR\\manga-shelf.exe`; uninstall removes only that shortcut.

- [ ] **Step 1: Add the minimal NSIS hook and configuration**

```nsh
!macro NSIS_HOOK_POSTINSTALL
  CreateShortCut "$DESKTOP\漫畫書庫.lnk" "$INSTDIR\manga-shelf.exe"
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  Delete "$DESKTOP\漫畫書庫.lnk"
!macroend
```

Set `bundle.windows.nsis.installerHooks` to `windows/hooks.nsh` without changing `currentUser`.

- [ ] **Step 2: Validate configuration and build a real installer**

Parse `tauri.conf.json`, run `cargo fmt --check`, and build an NSIS bundle. Configuration is treated as the approved TDD exception; source-text assertions are forbidden because they do not prove installer behavior.

Expected: JSON/schema validation and NSIS build PASS.

- [ ] **Step 3: Verify the real shortcut behavior**

With action-time confirmation, install the bundle for the current user. Resolve `%USERPROFILE%\\Desktop\\漫畫書庫.lnk` and assert its target is the installed `manga-shelf.exe`; launch from the shortcut once. Uninstall behavior is verified only in a disposable/manual acceptance run so existing user data is never deleted for a test.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/windows/hooks.nsh src-tauri/tauri.conf.json
git commit -m "feat: add Manga Library desktop shortcut"
```

---

### Task 2: Transactionally safe pre-update backup

**Files:**
- Create: `src-tauri/src/commands/backup.rs`
- Create: `src-tauri/tests/update_backup.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/db/mod.rs`
- Modify: `src-tauri/src/error.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/Cargo.lock`

**Interfaces:**
- Consumes: `Database`, Tauri `AppState`, application data directory and current SemVer string.
- Produces: `Database::backup_to(&Path) -> Result<(), AppError>` and async Tauri command `prepare_update_backup(state, app, current_version) -> Result<(), AppError>`.

- [ ] **Step 1: Write backup behavior tests first**

Test real temporary SQLite files and assert:

1. A backup opens successfully and contains the same series/edition/volume/collection rows as the source.
2. An unwritable destination returns `backup_failed` and leaves the source readable.
3. After four successful update backups only the newest three `library-before-*.sqlite3` files remain.
4. Cleanup happens only after the new backup opens and passes `PRAGMA integrity_check`.

- [ ] **Step 2: Run the backup RED**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test update_backup`

Expected: FAIL because `backup_to` and the backup rotation API do not exist.

- [ ] **Step 3: Add the SQLite backup feature and minimal implementation**

Change rusqlite features to `features = ["bundled", "backup"]`. Lock the existing connection, open a distinct destination `Connection`, call `rusqlite::backup::Backup::new`, and `run_to_completion(5, Duration::from_millis(50), None)`. Reopen the completed backup and require `PRAGMA integrity_check == "ok"` before rotating older files.

Use filenames shaped as `library-before-<semver>-<unix_utc_seconds>-<uuid>.sqlite3`; reject version text containing characters outside ASCII alphanumeric, dot, hyphen and plus.

- [ ] **Step 4: Expose a safe Tauri command**

`prepare_update_backup` resolves `app.path().app_data_dir()/backups`, performs blocking database work in `tauri::async_runtime::spawn_blocking`, and returns no filesystem path. Add `AppError::backup_failed()` with public code `backup_failed` and generic message `Unable to protect the library before updating.` Register the command in `lib.rs`.

- [ ] **Step 5: Run GREEN and Rust regression tests**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test update_backup
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

Expected: all PASS.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/commands/backup.rs src-tauri/tests/update_backup.rs src-tauri/src/commands/mod.rs src-tauri/src/db/mod.rs src-tauri/src/error.rs src-tauri/src/lib.rs src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "feat: protect library before updates"
```

---

### Task 3: Testable updater service

**Files:**
- Create: `src/services/updatePort.ts`
- Create: `src/services/tauriUpdateService.ts`
- Create: `src/services/tauriUpdateService.test.ts`
- Modify: `package.json`
- Modify: `pnpm-lock.yaml`

**Interfaces:**
- Produces:

```ts
export type UpdateProgress = {
  downloadedBytes: number;
  totalBytes: number | null;
  percent: number | null;
};

export type AvailableUpdate = {
  version: string;
  notes: string;
  install(onProgress: (progress: UpdateProgress) => void): Promise<void>;
};

export interface UpdatePort {
  currentVersion(): Promise<string>;
  check(): Promise<AvailableUpdate | null>;
}
```

- [ ] **Step 1: Write adapter tests before production code**

Inject `getVersion`, updater `check`, Tauri `invoke`, and process `relaunch`. Assert:

1. `currentVersion` returns Tauri's version.
2. `check` returns `null` when the plugin returns no update.
3. Available metadata maps exact `version` and plain-text `notes`.
4. `install` invokes `prepare_update_backup` with `{ currentVersion }` before `downloadAndInstall`.
5. `Started`, `Progress`, and `Finished` events produce bounded progress; unknown total produces `percent: null`.
6. `relaunch` occurs only after `downloadAndInstall` resolves.
7. Backup, download, or install failure prevents relaunch and rejects with a generic updater error.

- [ ] **Step 2: Run the adapter RED**

Run: `pnpm test -- src/services/tauriUpdateService.test.ts`

Expected: FAIL because the updater service does not exist.

- [ ] **Step 3: Install and implement the minimal adapters**

Add `@tauri-apps/plugin-updater` and `@tauri-apps/plugin-process` version `^2`. Implement `createTauriUpdateService(dependencies)` for tests and export a default instance using `@tauri-apps/api/app`, `@tauri-apps/api/core`, updater `check`, and process `relaunch`.

- [ ] **Step 4: Run adapter GREEN and TypeScript build**

Run:

```bash
pnpm test -- src/services/tauriUpdateService.test.ts
pnpm exec tsc -b --noEmit
```

Expected: all PASS.

- [ ] **Step 5: Commit**

```bash
git add src/services/updatePort.ts src/services/tauriUpdateService.ts src/services/tauriUpdateService.test.ts package.json pnpm-lock.yaml
git commit -m "feat: add signed update service"
```

---

### Task 4: In-app update experience

**Files:**
- Create: `src/components/UpdateStatus.tsx`
- Create: `src/components/UpdateStatus.test.tsx`
- Modify: `src/app/App.tsx`
- Modify: `src/app/App.css`
- Modify: `src/app/App.test.tsx`

**Interfaces:**
- Consumes: `UpdatePort` and `AvailableUpdate` from Task 3.
- Produces: non-blocking startup check, manual retry, explicit install consent, progress, and safe error states.

- [ ] **Step 1: Write component RED cases**

With a fake `UpdatePort`, assert:

1. It checks once on mount without blocking the library dashboard.
2. No-update state shows current version and an unobtrusive `已是最新版` result only after manual check.
3. An available version shows notes, `立即更新`, and `稍後更新`.
4. `稍後更新` closes the notice and never calls `install`.
5. `立即更新` calls `install`, disables duplicate clicks, and renders numeric progress when known.
6. Check/install failure renders `更新失敗，請稍後再試。` and a retry control while the library remains usable.

- [ ] **Step 2: Run component RED**

Run: `pnpm test -- src/components/UpdateStatus.test.tsx`

Expected: FAIL because `UpdateStatus` does not exist.

- [ ] **Step 3: Implement and mount the minimal component**

Add `<UpdateStatus updater={updater} />` at the bottom of the sidebar. Extend `App` props with `updater: UpdatePort = tauriUpdateService`; keep `LibraryPort` injection intact. Treat release notes as React text, never `dangerouslySetInnerHTML`.

- [ ] **Step 4: Run GREEN and frontend regression**

Run:

```bash
pnpm test -- src/components/UpdateStatus.test.tsx src/app/App.test.tsx
pnpm test
pnpm build
```

Expected: all PASS.

- [ ] **Step 5: Commit**

```bash
git add src/components/UpdateStatus.tsx src/components/UpdateStatus.test.tsx src/app/App.tsx src/app/App.css src/app/App.test.tsx
git commit -m "feat: show safe in-app updates"
```

---

### Task 5: Tauri updater signing and runtime permissions

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/Cargo.lock`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/capabilities/default.json`
- Modify: `src-tauri/tauri.conf.json`
- Modify: `package.json`

**Interfaces:**
- Consumes: GitHub latest endpoint and a newly generated Tauri updater public key.
- Produces: signed updater artifacts and only the updater/process permissions required by the main window.

- [ ] **Step 1: Generate signing keys outside Git**

Run `pnpm tauri signer generate -w "$env:USERPROFILE\\.tauri\\manga-library.key"`, use a strong user-provided password, verify the private key path is outside the repository, and securely record a separate backup. Copy only the emitted public key into `tauri.conf.json`.

- [ ] **Step 2: Configure updater/runtime**

Add Rust dependencies `tauri-plugin-updater = "2"` and `tauri-plugin-process = "2"`; initialize both plugins before setup. Add the two JavaScript dependencies from Task 3, set version `0.2.0` consistently in all three manifests, enable updater artifacts, add the exact endpoint and `passive` mode, and grant only required capabilities.

- [ ] **Step 3: Validate configuration and complete local verification**

Parse the exact JSON values, validate the Tauri configuration through a real build/check, and inspect generated capability schemas. Configuration is treated as the approved TDD exception; real updater behavior is covered by Tasks 3, 4 and 7.

Run:

```bash
pnpm test
pnpm build
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

Expected: all PASS.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/lib.rs src-tauri/capabilities/default.json src-tauri/tauri.conf.json package.json
git commit -m "feat: enable signed desktop updates"
```

---

### Task 6: Public GitHub release pipeline

**Files:**
- Create: `.github/workflows/release.yml`
- Modify: `README.md`

**Interfaces:**
- Consumes: Git tag `v0.2.0`, `TAURI_SIGNING_PRIVATE_KEY`, and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` repository secrets.
- Produces: public GitHub Release containing NSIS setup, signature, and `latest.json` for `windows-x86_64`.

- [ ] **Step 1: Add the minimal release workflow**

Use `actions/checkout@v4`, `pnpm/action-setup@v4`, `actions/setup-node@v4`, `dtolnay/rust-toolchain@stable`, and `tauri-apps/tauri-action@v0`. Set Node 24, pnpm 10, cache pnpm, run the full verification commands before publishing, and pass the signing secrets only to the Tauri build step.

- [ ] **Step 2: Document the user-facing workflow**

README must explain: install `0.2.0` once, launch from desktop, future updates appear inside the app, data remains local, and the first unsigned install may trigger SmartScreen.

- [ ] **Step 3: Validate with GitHub Actions**

Push a non-release branch first and validate GitHub parses the workflow. The tag run in Task 7 must execute every declared command and publish the real assets. Workflow configuration is treated as the approved TDD exception; source-text assertions are forbidden.

- [ ] **Step 4: Commit**

```bash
git add .github/workflows/release.yml README.md
git commit -m "ci: publish signed Manga Library updates"
```

---

### Task 7: Create repository, publish 0.2.0, and prove 0.2.1 update

**Files:**
- Modify for 0.2.1 validation: `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`

**Interfaces:**
- Produces: public `https://github.com/Weijun826/manga-library`, desktop-installed `0.2.0`, and verified in-app upgrade to `0.2.1`.

- [ ] **Step 1: Create the public repository without overwriting**

Verify `Weijun826/manga-library` does not exist. If it exists, stop for user direction. Otherwise create it as public, add it as `origin`, and do not include installer artifacts, database files, signing keys, `.env`, or local backups.

- [ ] **Step 2: Add repository secrets with action-time confirmation**

Add the exact contents of the private key as `TAURI_SIGNING_PRIVATE_KEY` and its password as `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`. Never display either value in chat or command output.

- [ ] **Step 3: Push source and publish bootstrap version**

Push `codex/manga-library-mvp`, make it the reviewed source branch, create tag `v0.2.0`, and wait for the release workflow. Verify release assets include the NSIS setup, `.sig`, and `latest.json`; verify `latest.json` references `windows-x86_64` and the public release URL.

- [ ] **Step 4: Install 0.2.0 once and verify desktop behavior**

With user confirmation, run the `0.2.0` NSIS setup. Verify desktop shortcut exists, double-click launches the app, version shows `0.2.0`, existing manga records remain, and update check reports current.

- [ ] **Step 5: Publish a real 0.2.1 validation update**

Change all three version fields to `0.2.1`, run the complete local suite, commit `chore: prepare 0.2.1 update`, push tag `v0.2.1`, and wait for GitHub Actions release success.

- [ ] **Step 6: Prove the no-reinstall workflow**

Open installed `0.2.0`, verify it discovers `0.2.1`, choose `立即更新`, observe backup and download progress, allow passive installation/restart, and verify version `0.2.1`, desktop shortcut, and pre-existing manga data. Inspect the app data backup directory only to confirm exactly one new valid pre-update backup exists.

- [ ] **Step 7: Final verification and handoff**

Run full frontend and Rust suites again, confirm Git status is clean, provide the public release link and installed executable path, and document how future releases are made. Do not claim automatic updates work until the `0.2.0` to `0.2.1` test succeeds.

- [ ] **Step 8: Commit version validation**

```bash
git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json
git commit -m "chore: validate 0.2.1 update channel"
```
