# 漫畫書庫桌面捷徑與自動更新設計

## 目標

讓使用者能從 Windows 桌面直接開啟漫畫書庫，不必使用 PowerShell；後續版本由應用程式檢查、下載、驗證並安裝，不必再次手動下載安裝器。更新不得遺失既有 SQLite 漫畫資料。

## 已核准範圍

- 程式維持安裝在 Windows 目前使用者的標準應用程式目錄。
- 安裝器建立「漫畫書庫」桌面捷徑，桌面不放置完整程式資料夾。
- 原始碼與更新檔放在公開 GitHub 儲存庫。
- 應用程式啟動時自動檢查 GitHub Releases；介面另提供手動「檢查更新」。
- 有新版時顯示版本與更新說明，提供「立即更新」與「稍後更新」。
- 使用者選擇立即更新後，程式自動下載、驗證、安裝並重新啟動。
- 更新前自動備份 SQLite 資料庫；備份失敗時不得開始更新。
- 不在本階段加入 Microsoft Store、背景強制更新、多平台發布或付費 Windows 程式碼簽章。

## 架構

### 桌面啟動

NSIS 採 `currentUser` 安裝，安裝器設定建立桌面捷徑。程式與解除安裝器仍位於 `%LOCALAPPDATA%\\漫畫書庫`，使用者只需雙擊桌面圖示。

### 更新來源與發布

公開 GitHub 儲存庫同時保存原始碼並透過 GitHub Releases 發布 Windows x86_64 NSIS 更新檔。應用程式的 updater endpoint 指向最新 release 的 `latest.json`。

發布新版時：

1. 同步提高 `package.json`、`src-tauri/Cargo.toml` 與 `src-tauri/tauri.conf.json` 的 SemVer 版本。
2. 建立版本標籤。
3. GitHub Actions 執行前端測試、TypeScript/Vite build、Rust 測試與 Cargo check。
4. 通過後建置 NSIS updater artifact、以 Tauri 私鑰簽章並建立 GitHub Release。
5. Tauri Action 產生 `latest.json`，供已安裝的應用程式檢查。

### 更新安全

- 使用 Tauri updater 公鑰驗證每個更新檔；簽章驗證不可停用。
- Tauri 簽章私鑰只存於 GitHub Actions Secret，不寫入程式碼、Git 或發布檔。
- 公鑰寫入 `tauri.conf.json`。
- 正式環境僅允許 HTTPS updater endpoint。
- Windows updater 使用 `passive` 安裝模式，適用目前使用者安裝且保留進度畫面。
- Tauri 更新簽章只保護更新來源與完整性；未購買 Authenticode 憑證前，第一次安裝仍可能顯示 SmartScreen 警告。

### 資料保護

更新開始前，Rust 備份服務使用 SQLite backup API 建立一致性備份，不直接複製使用中的資料庫檔。備份存於應用程式資料目錄的 `backups` 子目錄，命名包含 UTC 時間與更新前版本。

只保留最近三份自動更新備份，且必須先成功建立新備份，才可清理更舊的備份。備份或清理發生錯誤時顯示可理解的錯誤；備份失敗時終止更新，不動目前程式與資料庫。

### 前端更新體驗

更新狀態以獨立服務管理，畫面只消費明確狀態：

- `idle`：尚未檢查。
- `checking`：正在檢查。
- `upToDate`：目前已是最新版。
- `available`：顯示新版本、說明、立即更新與稍後更新。
- `backingUp`：正在保護資料。
- `downloading`：顯示下載進度。
- `installing`：提示即將重新啟動。
- `error`：顯示安全、可重試的錯誤，不洩漏本機路徑或內部例外。

啟動自動檢查失敗時不阻止使用書庫；使用者仍可在介面手動重試。應用程式不得在使用者尚未選擇「立即更新」時自動關閉。

## 元件與檔案責任

- `src/services/updatePort.ts`：定義前端可測試的更新介面與狀態型別。
- `src/services/tauriUpdateService.ts`：封裝 Tauri updater API、進度事件與重新啟動。
- `src/components/UpdateStatus.tsx`：呈現版本、更新通知、進度與錯誤。
- `src/app/App.tsx`：在應用程式殼層掛載更新元件，不承擔更新細節。
- `src-tauri/src/commands/backup.rs`：在更新前建立並輪替 SQLite 備份。
- `src-tauri/src/lib.rs`：註冊 updater/process plugin 與備份 command。
- `src-tauri/tauri.conf.json`：設定 updater 公鑰、GitHub endpoint、artifact 與 Windows passive 安裝。
- `.github/workflows/release.yml`：測試、建置、簽章及發布 release。

## 錯誤處理

- 沒有新版不是錯誤，不顯示干擾性通知。
- 網路失敗只影響更新檢查，不影響書庫操作。
- 簽章不符時拒絕安裝並顯示「更新驗證失敗」。
- 備份失敗時拒絕更新並保留現況。
- 下載或安裝失敗時保留目前版本，提供重新檢查。
- UI 不顯示私鑰、資料庫絕對路徑、Rust/JavaScript 堆疊或原始第三方錯誤。

## 測試與驗收

- 前端單元測試：無更新、有更新、稍後更新、進度、檢查失敗與更新失敗。
- 服務測試：command/envelope、版本資訊、進度事件與重新啟動只在成功安裝後發生。
- Rust 測試：一致性備份、備份失敗阻止更新、最多保留三份、原資料庫不變。
- CI：Vitest、TypeScript/Vite build、Cargo tests、Cargo check、Cargo fmt check 全部通過才可發布。
- 發布驗收：`latest.json` 版本、Windows x86_64 URL 與簽章有效；舊版能發現新版並透過 passive updater 完成更新與重啟。
- 桌面驗收：全新安裝後桌面有「漫畫書庫」捷徑，雙擊可啟動；更新後捷徑仍可用，既有漫畫資料仍存在。

## 首次設定限制

自動更新正式啟用前，必須完成一次人工設定：建立公開 GitHub 儲存庫、產生 Tauri updater 金鑰、將私鑰與密碼存入 GitHub Actions Secrets，並把公開 endpoint 與公鑰寫入應用程式。遺失私鑰會使已安裝版本無法信任後續更新，因此私鑰必須另有安全備份。

目前已安裝的 `0.1.0` 不包含 updater，無法自行取得 `0.2.0`。使用者需要最後一次安裝 updater-enabled `0.2.0`；從 `0.2.0` 升級至 `0.2.1` 起，才使用程式內更新流程，不再手動下載安裝器。
