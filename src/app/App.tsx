import { useCallback, useEffect, useRef, useState } from "react";
import type { FormEvent, ReactNode } from "react";

import { TextCover } from "../components/TextCover";
import { makeVolumeSortKey } from "../domain/volumeOrder";
import type {
  CollectionItemView,
  CreateSeriesBatchInput,
  DashboardSummary,
  EditionFormat,
  PublicationStatus,
  SeriesDetail,
  SeriesSummary,
  VolumeWithCollection,
} from "../domain/model";
import type { LibraryPort } from "../services/libraryPort";
import { tauriLibrary } from "../services/tauriLibrary";
import "./App.css";

type Screen = "dashboard" | "library" | "detail" | "add";
type LoadState<T> = { status: "loading" | "ready" | "error"; data: T | null; error?: string };

const initialCollection: CollectionItemView = {
  isOwned: false,
  isRead: false,
  isWishlisted: false,
  purchasePriceAmount: null,
  purchasePriceCurrency: null,
  acquiredOn: null,
  condition: "unknown",
  storageLocation: null,
  notes: null,
};

const initialForm = {
  title: "",
  author: "",
  publisher: "",
  volumeCount: "",
  originalTitle: "",
  description: "",
  editionName: "",
  publicationStatus: "unknown" as PublicationStatus,
  format: "tankobon" as EditionFormat,
};

function friendlyError() {
  return "目前無法完成這項操作，請稍後再試。";
}

function authorText(series: SeriesSummary) {
  return series.contributors.map((contributor) => contributor.name).join("、") || "作者未填寫";
}

function summaryText(series: SeriesSummary) {
  return `擁有 ${series.ownedVolumeCount}／已讀 ${series.readVolumeCount}／缺 ${series.missingVolumeCount}`;
}

function getFormErrors(form: typeof initialForm) {
  const errors: Record<string, string> = {};
  if (!form.title.trim()) errors.title = "請輸入系列名稱。";
  if (!form.author.trim()) errors.author = "請輸入作者。";
  if (!form.publisher.trim()) errors.publisher = "請輸入出版社。";
  const volumeCount = Number(form.volumeCount);
  if (!Number.isInteger(volumeCount) || volumeCount < 1 || volumeCount > 200) {
    errors.volumeCount = "冊數必須是 1 到 200 的整數。";
  }
  return errors;
}

function createInput(form: typeof initialForm): CreateSeriesBatchInput {
  const volumeCount = Number(form.volumeCount);
  return {
    series: {
      title: form.title.trim(),
      originalTitle: form.originalTitle.trim() || null,
      description: form.description.trim() || null,
      publicationStatus: form.publicationStatus,
      contributors: [{ name: form.author.trim(), role: "author", sortOrder: 0 }],
    },
    edition: {
      name: form.editionName.trim() || "單行本",
      languageCode: "ja",
      regionCode: "JP",
      publisher: form.publisher.trim(),
      format: form.format,
      releaseStatus: form.publicationStatus,
      knownVolumeCount: volumeCount,
    },
    volumes: Array.from({ length: volumeCount }, (_, index) => {
      const displayLabel = String(index + 1);
      return {
        displayLabel,
        sortKey: makeVolumeSortKey(displayLabel),
        titleOverride: null,
        isbn10: null,
        isbn13: null,
        translator: null,
        availabilityStatus: "released",
        releaseDate: null,
        releaseDatePrecision: "unknown",
        listPriceAmount: null,
        listPriceCurrency: null,
        collection: { ...initialCollection },
        provenance: { title: "manual", volume: "manual" },
      };
    }),
  };
}

export function App({ library = tauriLibrary }: { library?: LibraryPort }) {
  const [screen, setScreen] = useState<Screen>("dashboard");
  const [dashboard, setDashboard] = useState<LoadState<DashboardSummary>>({ status: "loading", data: null });
  const [series, setSeries] = useState<LoadState<SeriesSummary[]>>({ status: "loading", data: null });
  const [detail, setDetail] = useState<LoadState<SeriesDetail>>({ status: "loading", data: null });
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [query, setQuery] = useState("");
  const [form, setForm] = useState(initialForm);
  const [formErrors, setFormErrors] = useState<Record<string, string>>({});
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [updatingVolume, setUpdatingVolume] = useState<string | null>(null);
  const [deleteOpen, setDeleteOpen] = useState(false);
  const alive = useRef(true);
  const dashboardRequest = useRef(0);
  const seriesRequest = useRef(0);
  const detailRequest = useRef(0);

  useEffect(() => {
    alive.current = true;
    return () => { alive.current = false; };
  }, []);

  const loadDashboard = useCallback(async () => {
    const request = ++dashboardRequest.current;
    setDashboard({ status: "loading", data: null });
    try {
      const data = await library.getDashboard();
      if (alive.current && request === dashboardRequest.current) setDashboard({ status: "ready", data });
    } catch {
      if (alive.current && request === dashboardRequest.current) setDashboard({ status: "error", data: null });
    }
  }, [library]);

  const loadSeries = useCallback(async () => {
    const request = ++seriesRequest.current;
    setSeries({ status: "loading", data: null });
    try {
      const data = await library.listSeries({ query, publicationStatus: "all", collection: "all", reading: "all" });
      if (alive.current && request === seriesRequest.current) setSeries({ status: "ready", data });
    } catch {
      if (alive.current && request === seriesRequest.current) setSeries({ status: "error", data: null });
    }
  }, [library, query]);

  const loadDetail = useCallback(async (seriesId: string) => {
    const request = ++detailRequest.current;
    setDetail({ status: "loading", data: null });
    try {
      const data = await library.getSeriesDetail(seriesId);
      if (alive.current && request === detailRequest.current) setDetail({ status: "ready", data });
    } catch {
      if (alive.current && request === detailRequest.current) setDetail({ status: "error", data: null });
    }
  }, [library]);

  useEffect(() => { void loadDashboard(); }, [loadDashboard]);
  useEffect(() => { if (screen === "library") void loadSeries(); }, [screen, loadSeries]);
  useEffect(() => { if (screen === "detail" && selectedId) void loadDetail(selectedId); }, [screen, selectedId, loadDetail]);

  function goToLibrary() { setScreen("library"); }
  function openDetail(seriesId: string) { setSelectedId(seriesId); setScreen("detail"); setDeleteOpen(false); }
  function updateForm(field: keyof typeof initialForm, value: string) { setForm((current) => ({ ...current, [field]: value })); }

  async function submitForm(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const errors = getFormErrors(form);
    setFormErrors(errors);
    if (Object.keys(errors).length) return;
    setIsSubmitting(true);
    try {
      const created = await library.createSeriesBatch(createInput(form));
      if (!alive.current) return;
      setForm(initialForm);
      setSelectedId(created.id);
      setDetail({ status: "ready", data: created });
      setScreen("detail");
      void loadDashboard();
    } catch {
      if (alive.current) setFormErrors({ submit: friendlyError() });
    } finally {
      if (alive.current) setIsSubmitting(false);
    }
  }

  async function updateCollection(volume: VolumeWithCollection, flag: "isOwned" | "isRead" | "isWishlisted") {
    if (updatingVolume || (flag === "isWishlisted" && volume.collection.isOwned)) return;
    setUpdatingVolume(volume.id);
    const next = !volume.collection[flag];
    const patch = flag === "isOwned" && next ? { isOwned: true, isWishlisted: false } : { [flag]: next };
    try {
      await library.updateCollectionItem(volume.id, patch);
      if (alive.current && selectedId) await loadDetail(selectedId);
      void loadDashboard();
    } catch {
      if (alive.current) setDetail((current) => ({ ...current, error: friendlyError() }));
    } finally {
      if (alive.current) setUpdatingVolume(null);
    }
  }

  async function deleteSelected() {
    if (!selectedId) return;
    setIsSubmitting(true);
    try {
      await library.deleteSeries(selectedId);
      if (!alive.current) return;
      setDeleteOpen(false);
      setScreen("library");
      void loadDashboard();
    } catch {
      if (alive.current) setDetail((current) => ({ ...current, error: friendlyError() }));
    } finally {
      if (alive.current) setIsSubmitting(false);
    }
  }

  return (
    <main className="app-shell">
      <aside className="sidebar" aria-label="主要導覽">
        <div className="brand"><span className="brand-mark">冊</span><div><strong>漫畫書庫</strong><small>你的實體收藏</small></div></div>
        <nav>
          <button className={screen === "dashboard" ? "nav-link active" : "nav-link"} onClick={() => setScreen("dashboard")}>首頁</button>
          <button className={screen === "library" ? "nav-link active" : "nav-link"} onClick={goToLibrary}>我的漫畫</button>
        </nav>
        <button className="primary add-button" onClick={() => { setFormErrors({}); setScreen("add"); }}>＋ 新增漫畫</button>
      </aside>
      <section className="workspace">
        {screen === "dashboard" && <DashboardView state={dashboard} onRetry={loadDashboard} onAdd={() => setScreen("add")} onOpen={openDetail} />}
        {screen === "library" && <LibraryView state={series} query={query} onQueryChange={setQuery} onRetry={loadSeries} onOpen={openDetail} onAdd={() => setScreen("add")} />}
        {screen === "detail" && <DetailView state={detail} onRetry={() => selectedId && loadDetail(selectedId)} onBack={goToLibrary} onUpdate={updateCollection} updatingVolume={updatingVolume} onDelete={() => setDeleteOpen(true)} />}
        {screen === "add" && <AddView form={form} errors={formErrors} isSubmitting={isSubmitting} onChange={updateForm} onSubmit={submitForm} onCancel={goToLibrary} />}
      </section>
      {deleteOpen && detail.data && <ConfirmDialog title="刪除這個系列？" description={`「${detail.data.title}」和其冊數紀錄將一併移除。`} isSubmitting={isSubmitting} onCancel={() => setDeleteOpen(false)} onConfirm={deleteSelected} />}
    </main>
  );
}

function DashboardView({ state, onRetry, onAdd, onOpen }: { state: LoadState<DashboardSummary>; onRetry: () => void; onAdd: () => void; onOpen: (id: string) => void }) {
  if (state.status === "loading") return <Loading label="正在讀取收藏…" />;
  if (state.status === "error" || !state.data) return <ErrorState onRetry={onRetry} />;
  const { data } = state;
  return <div className="page"><header className="page-heading"><div><p className="eyebrow">收藏總覽</p><h1>漫畫書庫</h1><p>把每一本想讀、擁有與讀完的漫畫整理在一起。</p></div><button className="primary" onClick={onAdd}>新增漫畫</button></header><div className="stat-grid"><Stat label="系列" value={data.seriesCount} /><Stat label="擁有冊數" value={data.ownedVolumeCount} /><Stat label="已讀冊數" value={data.readVolumeCount} /><Stat label="缺冊數" value={data.missingVolumeCount} /></div>{data.seriesCount === 0 ? <EmptyState onAdd={onAdd} /> : <div className="dashboard-columns"><section><h2>最近收藏</h2>{data.recentVolumes.length ? <div className="recent-list">{data.recentVolumes.map((volume) => <article key={volume.id} className="recent-item"><TextCover title={volume.titleOverride || `第 ${volume.displayLabel} 冊`} /><div><strong>{volume.titleOverride || `第 ${volume.displayLabel} 冊`}</strong><span>第 {volume.displayLabel} 冊 · {volume.collection.isOwned ? "已擁有" : "收藏中"}</span></div></article>)}</div> : <p className="muted">尚無最近收藏。</p>}</section><section><h2>正在收集</h2>{data.incompleteSeries.length ? <div className="series-stack">{data.incompleteSeries.map((item) => <SeriesCard key={item.id} series={item} onOpen={onOpen} />)}</div> : <p className="muted">目前沒有缺冊的系列。</p>}</section></div>}</div>;
}

function LibraryView({ state, query, onQueryChange, onRetry, onOpen, onAdd }: { state: LoadState<SeriesSummary[]>; query: string; onQueryChange: (query: string) => void; onRetry: () => void; onOpen: (id: string) => void; onAdd: () => void }) {
  return <div className="page"><header className="page-heading compact"><div><p className="eyebrow">我的漫畫</p><h1>所有系列</h1></div><button className="primary" onClick={onAdd}>新增漫畫</button></header><label className="search"><span>搜尋</span><input value={query} onChange={(event) => onQueryChange(event.target.value)} placeholder="搜尋標題或作者" /></label>{state.status === "loading" ? <Loading label="正在讀取漫畫…" /> : state.status === "error" || !state.data ? <ErrorState onRetry={onRetry} /> : state.data.length === 0 ? <EmptyState onAdd={onAdd} title={query ? "找不到符合的漫畫" : undefined} /> : <div className="series-grid">{state.data.map((item) => <SeriesCard key={item.id} series={item} onOpen={onOpen} />)}</div>}</div>;
}

function DetailView({ state, onRetry, onBack, onUpdate, updatingVolume, onDelete }: { state: LoadState<SeriesDetail>; onRetry: () => void; onBack: () => void; onUpdate: (volume: VolumeWithCollection, flag: "isOwned" | "isRead" | "isWishlisted") => void; updatingVolume: string | null; onDelete: () => void }) {
  if (state.status === "loading") return <Loading label="正在開啟系列…" />;
  if (state.status === "error" || !state.data) return <ErrorState onRetry={onRetry} />;
  const series = state.data;
  return <div className="page"><button className="back" onClick={onBack}>← 回到我的漫畫</button><header className="detail-head"><TextCover title={series.title} large /><div><p className="eyebrow">{series.publishers.join("、") || "出版社未填寫"}</p><h1>{series.title}</h1><p>{authorText(series)}</p><p className="muted">{summaryText(series)}</p></div><button className="danger ghost" onClick={onDelete}>刪除系列</button></header>{state.error && <p className="form-error" role="alert">{state.error}</p>}{series.description && <p className="description">{series.description}</p>}{series.editions.map((edition) => <section className="edition" key={edition.id}><div className="edition-head"><div><h2>{edition.name}</h2><p>{edition.publisher} · {edition.volumes.length} 冊</p></div></div><div className="volume-list">{edition.volumes.map((volume) => <article className="volume-row" key={volume.id}><span className="volume-number">{volume.displayLabel}</span><div className="volume-actions"><Toggle label="擁有" pressed={volume.collection.isOwned} disabled={updatingVolume === volume.id} onClick={() => onUpdate(volume, "isOwned")} /><Toggle label="已讀" pressed={volume.collection.isRead} disabled={updatingVolume === volume.id} onClick={() => onUpdate(volume, "isRead")} /><Toggle label="願望" pressed={volume.collection.isWishlisted} disabled={updatingVolume === volume.id || volume.collection.isOwned} onClick={() => onUpdate(volume, "isWishlisted")} /></div></article>)}</div></section>)}</div>;
}

function AddView({ form, errors, isSubmitting, onChange, onSubmit, onCancel }: { form: typeof initialForm; errors: Record<string, string>; isSubmitting: boolean; onChange: (field: keyof typeof initialForm, value: string) => void; onSubmit: (event: FormEvent<HTMLFormElement>) => void; onCancel: () => void }) {
  return <div className="page form-page"><header className="page-heading compact"><div><p className="eyebrow">手動建立</p><h1>新增漫畫</h1><p>不需要搜尋或網路連線，先建立你的實體收藏。</p></div></header><form className="add-form" onSubmit={onSubmit} noValidate><FormField label="系列名稱" required error={errors.title}><input value={form.title} onChange={(event) => onChange("title", event.target.value)} /></FormField><FormField label="作者" required error={errors.author}><input value={form.author} onChange={(event) => onChange("author", event.target.value)} /></FormField><FormField label="出版社" required error={errors.publisher}><input value={form.publisher} onChange={(event) => onChange("publisher", event.target.value)} /></FormField><FormField label="冊數" required error={errors.volumeCount}><input type="number" min="1" max="200" value={form.volumeCount} onChange={(event) => onChange("volumeCount", event.target.value)} /></FormField><FormField label="原文名稱"><input value={form.originalTitle} onChange={(event) => onChange("originalTitle", event.target.value)} /></FormField><FormField label="版本名稱"><input placeholder="未填寫時使用「單行本」" value={form.editionName} onChange={(event) => onChange("editionName", event.target.value)} /></FormField><FormField label="連載狀態"><select value={form.publicationStatus} onChange={(event) => onChange("publicationStatus", event.target.value)}><option value="unknown">未知</option><option value="ongoing">連載中</option><option value="completed">已完結</option><option value="hiatus">休刊中</option></select></FormField><FormField label="版本格式"><select value={form.format} onChange={(event) => onChange("format", event.target.value)}><option value="tankobon">單行本</option><option value="omnibus">合訂本</option><option value="deluxe">豪華版</option><option value="box_set">套裝</option><option value="other">其他</option></select></FormField><FormField label="簡介" wide><textarea rows={4} value={form.description} onChange={(event) => onChange("description", event.target.value)} /></FormField>{errors.submit && <p className="form-error" role="alert">{errors.submit}</p>}<div className="form-actions"><button type="button" className="secondary" onClick={onCancel} disabled={isSubmitting}>取消</button><button type="submit" className="primary" disabled={isSubmitting}>{isSubmitting ? "正在新增…" : "建立系列"}</button></div></form></div>;
}

function FormField({ label, required, error, children, wide = false }: { label: string; required?: boolean; error?: string; children: ReactNode; wide?: boolean }) { return <label className={wide ? "field wide" : "field"}><span>{label}{required ? " *" : ""}</span>{children}{error && <small className="form-error">{error}</small>}</label>; }
function SeriesCard({ series, onOpen }: { series: SeriesSummary; onOpen: (id: string) => void }) { return <button className="series-card" onClick={() => onOpen(series.id)}><TextCover title={series.title} /><span className="series-card-body"><strong>{series.title}</strong><span>{authorText(series)} · {series.publishers.join("、") || "出版社未填寫"}</span><small>{summaryText(series)}</small></span></button>; }
function Stat({ label, value }: { label: string; value: number }) { return <article className="stat"><span>{label}</span><strong>{value}</strong></article>; }
function Toggle({ label, pressed, disabled, onClick }: { label: string; pressed: boolean; disabled: boolean; onClick: () => void }) { return <button className={pressed ? "toggle selected" : "toggle"} aria-pressed={pressed} disabled={disabled} onClick={onClick}>{label}</button>; }
function Loading({ label }: { label: string }) { return <div className="state" role="status">{label}</div>; }
function ErrorState({ onRetry }: { onRetry: () => void }) { return <div className="state error" role="alert"><p>{friendlyError()}</p><button className="secondary" onClick={onRetry}>重試</button></div>; }
function EmptyState({ onAdd, title = "還沒有漫畫" }: { onAdd: () => void; title?: string }) { return <section className="empty-state"><span aria-hidden="true">＋</span><h2>{title}</h2><p>從第一個系列開始，建立屬於你的漫畫收藏。</p><button className="primary" onClick={onAdd}>新增漫畫</button></section>; }
function ConfirmDialog({ title, description, isSubmitting, onCancel, onConfirm }: { title: string; description: string; isSubmitting: boolean; onCancel: () => void; onConfirm: () => void }) { return <div className="dialog-backdrop" role="presentation"><section className="dialog" role="dialog" aria-modal="true" aria-labelledby="confirm-title"><h2 id="confirm-title">{title}</h2><p>{description}</p><div><button className="secondary" onClick={onCancel} disabled={isSubmitting}>取消</button><button className="danger" onClick={onConfirm} disabled={isSubmitting}>{isSubmitting ? "正在刪除…" : "確認刪除"}</button></div></section></div>; }
