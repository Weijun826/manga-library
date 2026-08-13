import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it } from "vitest";

import type { AvailableUpdate, UpdatePort, UpdateProgress } from "../services/updatePort";
import { UpdateStatus } from "./UpdateStatus";

afterEach(cleanup);

class FakeUpdater implements UpdatePort {
  checks = 0;

  constructor(
    private readonly results: Array<AvailableUpdate | null | Error | Promise<AvailableUpdate | null>>,
  ) {}

  async currentVersion() {
    return "0.2.0";
  }

  async check() {
    const result = this.results[Math.min(this.checks, this.results.length - 1)];
    this.checks += 1;
    if (result instanceof Error) throw result;
    return await result;
  }
}

function availableUpdate(
  install: (onProgress: (progress: UpdateProgress) => void) => Promise<void> = async () => {},
): AvailableUpdate {
  return {
    version: "0.3.0",
    notes: "修正更新流程並保護資料。",
    install,
  };
}

describe("UpdateStatus", () => {
  it("checks once on startup without blocking the library content", async () => {
    const neverFinishes = new Promise<AvailableUpdate | null>(() => {});
    const updater = new FakeUpdater([neverFinishes]);

    render(
      <>
        <h1>漫畫書庫</h1>
        <UpdateStatus updater={updater} />
      </>,
    );

    expect(screen.getByRole("heading", { name: "漫畫書庫" })).toBeVisible();
    expect(await screen.findByText("正在檢查更新…")).toBeVisible();
    expect(updater.checks).toBe(1);
  });

  it("shows the installed version and reports no update only after a manual check", async () => {
    const user = userEvent.setup();
    const updater = new FakeUpdater([null, null]);
    render(<UpdateStatus updater={updater} />);

    expect(await screen.findByText("版本 0.2.0")).toBeVisible();
    await waitFor(() => expect(updater.checks).toBe(1));
    expect(screen.queryByText("已是最新版")).not.toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "檢查更新" }));

    expect(await screen.findByText("已是最新版")).toBeVisible();
    expect(updater.checks).toBe(2);
  });

  it("shows an available update as text and allows postponing it", async () => {
    const user = userEvent.setup();
    const update = availableUpdate();
    update.notes = "<img src=x onerror=alert(1)> 安全說明";
    render(<UpdateStatus updater={new FakeUpdater([update])} />);

    expect(await screen.findByText("可更新至 0.3.0")).toBeVisible();
    expect(screen.getByText("<img src=x onerror=alert(1)> 安全說明")).toBeVisible();
    expect(document.querySelector("img")).toBeNull();

    await user.click(screen.getByRole("button", { name: "稍後更新" }));

    expect(screen.queryByText("可更新至 0.3.0")).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: "檢查更新" })).toBeVisible();
  });

  it("installs only after confirmation and displays download progress", async () => {
    const user = userEvent.setup();
    let installs = 0;
    let finishInstall: (() => void) | undefined;
    const installGate = new Promise<void>((resolve) => {
      finishInstall = resolve;
    });
    const update = availableUpdate(async (onProgress) => {
      installs += 1;
      onProgress({ downloadedBytes: 25, totalBytes: 100, percent: 25 });
      await installGate;
    });
    render(<UpdateStatus updater={new FakeUpdater([update])} />);
    await screen.findByText("可更新至 0.3.0");

    await user.click(screen.getByRole("button", { name: "立即更新" }));

    expect(await screen.findByText("下載 25%")).toBeVisible();
    expect(screen.getByRole("button", { name: "正在更新…" })).toBeDisabled();
    expect(installs).toBe(1);
    finishInstall?.();
    expect(await screen.findByText("正在完成更新並重新啟動…")).toBeVisible();
  });

  it("keeps the library usable and offers retry after a safe update error", async () => {
    const user = userEvent.setup();
    const updater = new FakeUpdater([new Error("private remote detail"), null]);
    render(
      <>
        <button>新增漫畫</button>
        <UpdateStatus updater={updater} />
      </>,
    );

    expect(await screen.findByText("更新失敗，請稍後再試。")).toBeVisible();
    expect(screen.getByRole("button", { name: "新增漫畫" })).toBeEnabled();
    expect(screen.queryByText("private remote detail")).not.toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "重新檢查" }));

    expect(await screen.findByText("已是最新版")).toBeVisible();
    expect(updater.checks).toBe(2);
  });
});
