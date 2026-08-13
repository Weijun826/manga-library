import { useEffect, useState } from "react";

import type { AvailableUpdate, UpdatePort, UpdateProgress } from "../services/updatePort";

type Status =
  | "idle"
  | "checking"
  | "upToDate"
  | "available"
  | "backingUp"
  | "downloading"
  | "installing"
  | "error";

export function UpdateStatus({ updater }: { updater: UpdatePort }) {
  const [currentVersion, setCurrentVersion] = useState<string | null>(null);
  const [available, setAvailable] = useState<AvailableUpdate | null>(null);
  const [progress, setProgress] = useState<UpdateProgress | null>(null);
  const [status, setStatus] = useState<Status>("checking");

  useEffect(() => {
    let active = true;
    void updater
      .currentVersion()
      .then((version) => {
        if (active) setCurrentVersion(version);
      })
      .catch(() => {
        if (active) setStatus("error");
      });
    void updater
      .check()
      .then((update) => {
        if (!active) return;
        setAvailable(update);
        setStatus(update ? "available" : "idle");
      })
      .catch(() => {
        if (active) setStatus("error");
      });
    return () => {
      active = false;
    };
  }, [updater]);

  async function checkManually() {
    setStatus("checking");
    setProgress(null);
    try {
      const update = await updater.check();
      setAvailable(update);
      setStatus(update ? "available" : "upToDate");
    } catch {
      setStatus("error");
    }
  }

  async function install() {
    if (!available || status === "backingUp" || status === "downloading") return;
    setStatus("backingUp");
    setProgress(null);
    try {
      await available.install((nextProgress) => {
        setProgress(nextProgress);
        setStatus("downloading");
      });
      setStatus("installing");
    } catch {
      setStatus("error");
    }
  }

  return (
    <section className="update-status" aria-label="應用程式更新">
      {currentVersion && <small>版本 {currentVersion}</small>}
      {status === "checking" && <span>正在檢查更新…</span>}
      {status === "idle" && (
        <button type="button" onClick={() => void checkManually()}>
          檢查更新
        </button>
      )}
      {status === "upToDate" && (
        <>
          <span>已是最新版</span>
          <button type="button" onClick={() => void checkManually()}>
            再次檢查
          </button>
        </>
      )}
      {status === "available" && available && (
        <div className="update-notice">
          <strong>可更新至 {available.version}</strong>
          {available.notes && <p>{available.notes}</p>}
          <div>
            <button type="button" className="update-primary" onClick={() => void install()}>
              立即更新
            </button>
            <button type="button" onClick={() => setStatus("idle")}>
              稍後更新
            </button>
          </div>
        </div>
      )}
      {status === "backingUp" && (
        <>
          <span>正在備份資料…</span>
          <button type="button" disabled>正在更新…</button>
        </>
      )}
      {status === "downloading" && (
        <>
          <span>{progress?.percent === null ? "正在下載更新…" : `下載 ${progress?.percent ?? 0}%`}</span>
          <button type="button" disabled>正在更新…</button>
        </>
      )}
      {status === "installing" && <span>正在完成更新並重新啟動…</span>}
      {status === "error" && (
        <>
          <span role="alert">更新失敗，請稍後再試。</span>
          <button type="button" onClick={() => void checkManually()}>
            重新檢查
          </button>
        </>
      )}
    </section>
  );
}
