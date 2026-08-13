# 漫畫資料編輯、連載冊數與漫畫主題設計

## 目標

讓使用者能安全修改漫畫系列基本資料、換上本機真實漫畫封面，並為連載系列持續新增冊數；同時把現有介面改成清楚且適合長時間使用的「現代漫畫封面」主題。

## 已核准範圍

- 從系列詳情頁進入獨立的完整編輯頁。
- 可修改系列名稱、原文名稱、作者、出版社、簡介、連載狀態與版本名稱。
- 可從本機選擇 JPG、PNG 或 WebP 作為系列封面，並可更換或移除。
- 每個版本可新增冊數，支援一般數字及特殊冊號標籤。
- 新增冊數時可填 ISBN、發售狀態與初始收藏狀態。
- 不在本階段刪除既有冊數、調整冊數順序、連網搜尋封面或新增進階設定。
- 主題採「A｜現代漫畫封面」。
- 主題與編輯完成後，才另外評估可點選設定與其他書庫功能。

## 編輯流程

系列詳情頁封面旁新增「編輯資料」。編輯頁預填目前資料，使用者可儲存或取消。取消不修改資料，也不保留暫存封面。

儲存基本資料時，後端在單一 transaction 內更新系列、主要作者、版本名稱與出版社。既有 series、edition、volume 與 collection IDs 不變，因此收藏、已讀與願望狀態完整保留。

必要欄位為系列名稱、作者、出版社與版本名稱；移除前後空白後不得為空。原文名稱與簡介可留空並存为 `NULL`。連載狀態只能是既有 domain enum。

本階段一個系列仍只編輯目前 MVP 建立的主要版本與主要作者；不新增多作者、多版本管理介面。

## 連載新增冊數

每個版本標題旁新增「新增冊數」。開啟後預填自然排序的下一個整數，例如現有 1–12 冊時預填 13；使用者仍可改成 `13.5`、`上`、`下` 或 `外傳` 等非空標籤。

表單包含：

- 冊號標籤（必填）。
- ISBN（選填，接受 ISBN-10 或 ISBN-13）。
- 發售狀態：已發售、即將發售、未知。
- 已擁有、已閱讀、願望清單。

若選擇已擁有，願望清單必須自動關閉。冊號使用既有 domain sort-key 規則；ISBN 有填寫時必須正規化並通過 checksum。相同版本不得有重複的冊號／sort-key 組合，整個資料庫不得有重複的非空 ISBN。

新增成功後重新讀取詳情與 dashboard，更新已知冊數、缺冊與收藏統計。本階段不提供刪除既有冊數，避免誤刪收藏資料。

## 本機系列封面

前端使用官方 Tauri dialog plugin，僅允許單選 JPG、JPEG、PNG、WebP。選取後顯示預覽，但按下「儲存變更」前不得更動正式資料。

專用 Rust command 接收使用者明確選擇的來源路徑，不授予前端廣泛的檔案系統寫入權限。後端執行：

1. 讀取前 512 bytes 並由 magic bytes 判斷實際格式，不信任副檔名。
2. 限制完整檔案最多 10 MiB，拒絕空檔與不支援格式。
3. 計算 SHA-256，以內容雜湊命名並複製到 app data 的 `covers` 目錄。
4. 以 relative path 建立或重用 `cover_assets` row，source type 為 `user_file`。
5. 在 transaction 內把系列的 `representative_cover_asset_id` 指向新 asset。

檔案寫入、資料庫更新或完整性驗證失敗時保留原資料與原封面。若新檔案已寫入但 transaction 失敗，立即清除該次新產生且未被引用的檔案。更換或移除封面後，只清除確認沒有任何資料庫 row 引用的舊 asset／檔案。

WebView 只透過 Tauri asset protocol 顯示 app data `covers` 目錄，scope 不包含使用者其他目錄。CSP 保持只允許 `self`、`asset:` 與 `http://asset.localhost` 圖片來源。

## 前後端介面

`LibraryPort` 新增：

```ts
updateSeriesMetadata(seriesId: string, input: UpdateSeriesMetadataInput): Promise<SeriesDetail>;
addVolume(editionId: string, input: AddVolumeInput): Promise<VolumeWithCollection>;
selectCoverImage(): Promise<string | null>;
setSeriesCover(seriesId: string, sourcePath: string): Promise<CoverAsset>;
removeSeriesCover(seriesId: string): Promise<void>;
```

Tauri Rust commands 對應：

- `update_series_metadata`
- `add_volume`
- `set_series_cover`
- `remove_series_cover`

Dialog selection 保持在 TypeScript adapter，實際檔案驗證、複製與資料庫修改只在 Rust 執行。

## 漫畫主題

主題採「現代漫畫封面」：

- 主內容使用溫暖米白紙色，降低長時間整理的疲勞。
- 側欄使用墨黑，維持清楚導覽與高對比。
- 漫畫紅作為主要 CTA、選取狀態與必要錯誤提示。
- 標題使用粗黑字與緊密字距；表單與長文字維持 Segoe UI／Microsoft JhengHei 的可讀性。
- 漫畫卡片以封面為主；真實封面可用時顯示圖片，否則保留文字封面 fallback。
- 封面加入 3–4 px 硬陰影，呈現印刷套色感；不使用持續動畫、大面積網點或對話框裝飾。
- 更新狀態留在側欄底部，避免搶走收藏內容視線。
- Windows 桌面為主要版面，保留現有小尺寸響應式行為。

## UI 狀態與錯誤

- 編輯與新增冊數各自有 idle、submitting、field-error、operation-error 狀態。
- 提交期間停用重複操作；關閉／取消不寫入。
- 使用者錯誤顯示具體欄位訊息；系統錯誤顯示安全通用訊息。
- 不顯示來源圖片絕對路徑、SQL、Rust／JavaScript stack、內部 asset relative path。
- 封面載入失敗時回退文字封面，不讓破圖破壞書庫。

## 測試與驗收

### Rust

- 編輯系列後原 volume／collection IDs 與收藏狀態不變。
- 任一欄位／資料庫操作失敗時整批 rollback。
- 新增下一個整數冊號與特殊冊號，並產生正確排序。
- 重複冊號與重複 ISBN 被拒絕。
- 已擁有冊數不得同時在願望清單。
- JPG、PNG、WebP magic bytes、10 MiB 邊界、空檔、偽造副檔名與過大檔案。
- asset 寫入或 transaction 失敗後原封面不變且無孤兒檔案。
- 移除封面不刪除仍被其他 row 引用的 asset。

### TypeScript／React

- 詳情頁可進入編輯頁，預填資料，取消不呼叫更新。
- 欄位驗證、成功儲存與安全錯誤重試。
- 選擇、更換、移除封面與取消選擇。
- 新增冊數預填下一冊、特殊冊號、ISBN 錯誤與收藏互斥。
- 有真實封面時顯示圖片；無封面或圖片載入失敗時顯示文字封面。
- 現有新增系列、搜尋、收藏、已讀、願望與刪除系列流程保持通過。

### 整合驗收

- 在已安裝 Windows app 編輯既有系列，重新啟動後資料仍在。
- 更換為本機真實封面，移動／刪除來源圖片後封面仍可顯示。
- 為連載系列新增下一冊與特殊冊號，dashboard 統計正確。
- 完成 Vitest、TypeScript/Vite build、Cargo tests、Cargo check、Cargo fmt check 與 NSIS build。
