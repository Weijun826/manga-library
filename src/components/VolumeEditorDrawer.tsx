import { useState } from "react";
import type { FormEvent, MouseEvent } from "react";

import type { UpdateVolumeDetailsInput, VolumeWithCollection } from "../domain/model";

type FormState = {
  displayLabel: string;
  isbn: string;
  availabilityStatus: UpdateVolumeDetailsInput["availabilityStatus"];
  isOwned: boolean;
  isRead: boolean;
  isWishlisted: boolean;
  acquiredOn: string;
  purchasePrice: string;
  condition: VolumeWithCollection["collection"]["condition"];
  storageLocation: string;
  notes: string;
};

function initialForm(volume: VolumeWithCollection): FormState {
  return {
    displayLabel: volume.displayLabel,
    isbn: volume.isbn13 ?? volume.isbn10 ?? "",
    availabilityStatus: volume.availabilityStatus,
    isOwned: volume.collection.isOwned,
    isRead: volume.collection.isRead,
    isWishlisted: volume.collection.isWishlisted,
    acquiredOn: volume.collection.acquiredOn ?? "",
    purchasePrice: volume.collection.purchasePriceAmount?.toString() ?? "",
    condition: volume.collection.condition,
    storageLocation: volume.collection.storageLocation ?? "",
    notes: volume.collection.notes ?? "",
  };
}

function validDate(value: string) {
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(value);
  if (!match) return false;
  const year = Number(match[1]);
  const month = Number(match[2]);
  const day = Number(match[3]);
  const date = new Date(Date.UTC(year, month - 1, day));
  return date.getUTCFullYear() === year && date.getUTCMonth() === month - 1 && date.getUTCDate() === day;
}

function saveError(error: unknown) {
  const code = typeof error === "object" && error !== null && "code" in error
    ? String((error as { code: unknown }).code)
    : "";
  const messages: Record<string, string> = {
    invalid_volume_label: "請輸入有效的卷數或標籤。",
    volume_already_exists: "同一版本已經有這個卷數。",
    invalid_isbn: "ISBN 格式或檢查碼不正確。",
    isbn_already_exists: "這個 ISBN 已存在於其他卷冊。",
    invalid_acquired_on: "入手日期格式不正確。",
    invalid_purchase_price: "購買價格必須是 0 以上的整數。",
    volume_not_found: "找不到這個卷冊，請重新開啟系列。",
  };
  return messages[code] ?? "無法儲存卷冊資料，請稍後再試。";
}

export function VolumeEditorDrawer({ volume, busy, onClose, onSave }: {
  volume: VolumeWithCollection;
  busy: boolean;
  onClose: () => void;
  onSave: (input: UpdateVolumeDetailsInput) => Promise<void>;
}) {
  const original = initialForm(volume);
  const [form, setForm] = useState(original);
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [confirmClose, setConfirmClose] = useState(false);
  const dirty = JSON.stringify(form) !== JSON.stringify(original);

  function change(field: keyof FormState, value: string | boolean) {
    setForm((current) => ({ ...current, [field]: value }));
    setErrors((current) => ({ ...current, [field]: "", submit: "" }));
  }
  function requestClose() { if (dirty) setConfirmClose(true); else onClose(); }
  function backdropClick(event: MouseEvent<HTMLDivElement>) { if (event.target === event.currentTarget) requestClose(); }

  async function submit(event: FormEvent) {
    event.preventDefault();
    const nextErrors: Record<string, string> = {};
    const label = form.displayLabel.trim();
    if (!label) nextErrors.displayLabel = "請輸入卷數或標籤。";
    if (form.acquiredOn && !validDate(form.acquiredOn)) nextErrors.acquiredOn = "請輸入有效日期，例如 2026-08-14。";
    const price = form.purchasePrice === "" ? null : Number(form.purchasePrice);
    if (price !== null && (!Number.isInteger(price) || price < 0)) nextErrors.purchasePrice = "價格必須是 0 以上的整數。";
    setErrors(nextErrors);
    if (Object.keys(nextErrors).length) return;
    try {
      await onSave({
        displayLabel: label,
        isbn: form.isbn.trim() || null,
        availabilityStatus: form.availabilityStatus,
        collection: {
          isOwned: form.isOwned,
          isRead: form.isRead,
          isWishlisted: form.isOwned ? false : form.isWishlisted,
          purchasePriceAmount: price,
          purchasePriceCurrency: price === null ? null : "TWD",
          acquiredOn: form.acquiredOn || null,
          condition: form.condition,
          storageLocation: form.storageLocation.trim() || null,
          notes: form.notes.trim() || null,
        },
      });
    } catch (error) {
      setErrors({ submit: saveError(error) });
    }
  }

  return <div className="drawer-backdrop" onClick={backdropClick}>
    <aside className="volume-drawer" role="dialog" aria-modal="true" aria-labelledby="volume-editor-title">
      <header><div><p className="eyebrow">卷冊收藏資料</p><h2 id="volume-editor-title">編輯第 {volume.displayLabel} 冊</h2></div><button className="drawer-close" type="button" aria-label="關閉" onClick={requestClose}>×</button></header>
      <form onSubmit={(event) => void submit(event)} noValidate>
        <label className="field"><span>卷數／標籤</span><input value={form.displayLabel} onChange={(event) => change("displayLabel", event.target.value)} />{errors.displayLabel && <small className="form-error">{errors.displayLabel}</small>}</label>
        <label className="field"><span>ISBN</span><input value={form.isbn} onChange={(event) => change("isbn", event.target.value)} /></label>
        <label className="field"><span>出版狀態</span><select value={form.availabilityStatus} onChange={(event) => change("availabilityStatus", event.target.value)}><option value="released">已出版</option><option value="upcoming">即將出版</option><option value="unknown">未知</option></select></label>
        <fieldset className="drawer-checks"><legend>收藏狀態</legend><label><input type="checkbox" checked={form.isOwned} onChange={(event) => { change("isOwned", event.target.checked); if (event.target.checked) change("isWishlisted", false); }} />擁有</label><label><input type="checkbox" checked={form.isRead} onChange={(event) => change("isRead", event.target.checked)} />已讀</label><label><input type="checkbox" checked={form.isWishlisted} disabled={form.isOwned} onChange={(event) => change("isWishlisted", event.target.checked)} />願望</label></fieldset>
        <label className="field"><span>入手日期</span><input inputMode="numeric" placeholder="YYYY-MM-DD" value={form.acquiredOn} onChange={(event) => change("acquiredOn", event.target.value)} />{errors.acquiredOn && <small className="form-error">{errors.acquiredOn}</small>}</label>
        <label className="field"><span>購買價格（TWD）</span><input type="number" min="0" step="1" value={form.purchasePrice} onChange={(event) => change("purchasePrice", event.target.value)} />{errors.purchasePrice && <small className="form-error">{errors.purchasePrice}</small>}</label>
        <label className="field"><span>書況</span><select value={form.condition} onChange={(event) => change("condition", event.target.value)}><option value="unknown">未知</option><option value="new">全新</option><option value="like_new">近全新</option><option value="good">良好</option><option value="fair">普通</option><option value="poor">不佳</option></select></label>
        <label className="field"><span>收納位置</span><input value={form.storageLocation} onChange={(event) => change("storageLocation", event.target.value)} /></label>
        <label className="field"><span>備註</span><textarea rows={4} value={form.notes} onChange={(event) => change("notes", event.target.value)} /></label>
        {errors.submit && <p className="form-error" role="alert">{errors.submit}</p>}
        <div className="drawer-actions"><button type="button" className="secondary" disabled={busy} onClick={requestClose}>取消</button><button className="primary" disabled={busy}>{busy ? "儲存中…" : "儲存變更"}</button></div>
      </form>
      {confirmClose && <section className="drawer-confirm" role="alertdialog" aria-labelledby="discard-title"><h3 id="discard-title">放棄未儲存變更？</h3><p>剛才修改的內容不會保存。</p><div><button className="secondary" onClick={() => setConfirmClose(false)}>繼續編輯</button><button className="danger" onClick={onClose}>放棄變更</button></div></section>}
    </aside>
  </div>;
}
