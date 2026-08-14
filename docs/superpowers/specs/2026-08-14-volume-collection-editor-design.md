# 卷冊收藏資料編輯設計

日期：2026-08-14

## 目標

讓使用者在系列詳情頁直接編輯既有卷冊的書目與收藏資料，不必刪除後重建。此功能只處理卷冊編輯，不包含刪除卷冊、批次編輯或網路書目查詢。

## 使用者介面

- 點選系列詳情中的任一卷冊列，從畫面右側開啟編輯抽屜。
- 抽屜保留目前系列內容作為背景，適合連續檢查不同卷冊。
- 可編輯欄位：
  - 卷數／顯示標籤
  - ISBN
  - 出版狀態：已出版、即將出版、未知
  - 收藏狀態：擁有、已讀、願望
  - 入手日期
  - 購買價格
  - 書況
  - 收納位置
  - 備註
- 購買價格為選填的非負整數，幣別固定使用 TWD；空白代表沒有記錄價格。
- 抽屜提供「取消」與「儲存」；儲存期間停用重複送出。
- 有未儲存變更時，按取消、背景或關閉按鈕都先顯示站內確認訊息。
- 儲存成功後關閉抽屜，重新載入系列詳情、書庫列表與首頁統計。
- 儲存失敗時保留抽屜與使用者輸入，並在表單內顯示可理解的錯誤。

## 資料與介面

前端新增 `UpdateVolumeDetailsInput`，內容包含卷冊欄位與完整 `CollectionItemView`。`LibraryPort` 新增：

```ts
updateVolumeDetails(
  volumeId: string,
  input: UpdateVolumeDetailsInput,
): Promise<VolumeWithCollection>;
```

Tauri adapter 對應單一 `update_volume_details` command，參數 envelope 使用既有 camelCase 慣例：`{ volumeId, input }`。

Rust repository 在同一個 SQLite transaction 內：

1. 驗證與正規化輸入。
2. 更新 `volumes` 的標籤、排序鍵、ISBN 與出版狀態。
3. upsert 對應的 `collection_items` 完整收藏資料。
4. 讀回並回傳最新的 `VolumeWithCollection`。

任何一步失敗都回滾，避免書目已更新但收藏資料未更新的半完成狀態。

## 驗證與規則

- 卷數／標籤去除前後空白後不可為空，並沿用現有自然排序鍵規則。
- 同一版本內不可有重複的卷數／標籤；檢查時排除目前編輯的卷冊。
- ISBN 允許空白。輸入時移除空格與連字號並正規化 `x`；非空值必須是有效 ISBN-10 或 ISBN-13。
- ISBN 不可與其他卷冊重複；檢查時排除目前編輯的卷冊。
- 入手日期允許空白；非空值必須是有效的 `YYYY-MM-DD` 日期。
- 購買價格允許空白；非空值必須是非負整數，儲存為 TWD。
- 收納位置與備註去除前後空白；空字串儲存為 `null`。
- 勾選「擁有」時強制取消「願望」。取消「擁有」不會自動加入願望。
- 「已讀」與「擁有」互相獨立，允許記錄借閱或曾讀過但未持有的卷冊。
- 出版狀態為「即將出版」或「未知」時仍可記錄收藏資料；首頁缺書統計維持既有規則，只計算已出版且未擁有的卷冊。

## 元件邊界

- `VolumeEditorDrawer`：管理表單狀態、前端即時驗證、未儲存關閉確認與錯誤顯示。
- `App`：控制抽屜開關、呼叫 `LibraryPort`、成功後刷新目前頁面資料。
- `LibraryPort`／Tauri adapter：提供一個明確的跨層更新方法，不在前端拆成兩次儲存。
- Rust command：只負責取得資料庫 handle 與轉交 repository。
- Repository：負責資料驗證、唯一性檢查、交易更新與回傳最新資料。

## 錯誤處理

應將可預期錯誤映射成繁體中文表單訊息：

- 卷數空白或格式錯誤
- 同版本卷數重複
- ISBN 格式或檢查碼錯誤
- ISBN 已存在
- 日期或價格格式錯誤
- 找不到卷冊

其他資料庫錯誤只顯示通用失敗訊息，不洩漏 SQL、檔案路徑或內部錯誤內容。

## 測試與完成條件

- Domain／repository tests：正規化 ISBN、拒絕重複標籤與 ISBN、驗證日期與價格、交易回滾、收藏規則、成功回傳完整資料。
- Adapter tests：command 名稱、camelCase envelope 與回傳值正確。
- Component tests：開啟並預填抽屜、修改完整欄位、擁有與願望互斥、驗證訊息、未儲存關閉確認、錯誤保留輸入。
- App integration tests：點選卷冊、成功儲存並刷新；失敗時抽屜保持開啟。
- 最終驗證：完整 Vitest、TypeScript／Vite build、Rust tests、`cargo check`、`cargo fmt --check`。

完成標準是使用者能從已安裝程式的系列詳情頁編輯既有卷冊，重開程式後資料仍存在，且既有新增卷冊、封面、搜尋與收藏切換功能不退步。
