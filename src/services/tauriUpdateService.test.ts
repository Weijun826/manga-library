import { describe, expect, it } from "vitest";

import { createTauriUpdateService } from "./tauriUpdateService";
import type { UpdateProgress } from "./updatePort";

type DownloadEvent =
  | { event: "Started"; data: { contentLength?: number } }
  | { event: "Progress"; data: { chunkLength: number } }
  | { event: "Finished"; data?: never };

function pluginUpdate(
  downloadAndInstall: (onEvent: (event: DownloadEvent) => void) => Promise<void> = async () => {},
) {
  return {
    version: "0.3.0",
    currentVersion: "0.2.0",
    body: "新增安全更新功能。",
    date: "2026-08-13T12:00:00Z",
    downloadAndInstall,
  };
}

function dependencies(overrides: Record<string, unknown> = {}) {
  return {
    getVersion: async () => "0.2.0",
    checkForUpdate: async () => pluginUpdate(),
    invoke: async () => undefined,
    relaunch: async () => undefined,
    ...overrides,
  };
}

describe("tauriUpdateService", () => {
  it("returns the installed application version", async () => {
    const updater = createTauriUpdateService(dependencies());

    await expect(updater.currentVersion()).resolves.toBe("0.2.0");
  });

  it("returns null when the updater reports no newer release", async () => {
    const updater = createTauriUpdateService(
      dependencies({ checkForUpdate: async () => null }),
    );

    await expect(updater.check()).resolves.toBeNull();
  });

  it("maps available version metadata as plain text", async () => {
    const updater = createTauriUpdateService(dependencies());

    const available = await updater.check();

    expect(available).not.toBeNull();
    expect(available?.version).toBe("0.3.0");
    expect(available?.notes).toBe("新增安全更新功能。");
  });

  it("backs up before downloading and relaunches only after installation", async () => {
    const events: string[] = [];
    const updater = createTauriUpdateService(
      dependencies({
        invoke: async (command: string, args?: Record<string, unknown>) => {
          events.push(`${command}:${String(args?.currentVersion)}`);
        },
        checkForUpdate: async () =>
          pluginUpdate(async () => {
            events.push("downloadAndInstall");
          }),
        relaunch: async () => {
          events.push("relaunch");
        },
      }),
    );
    const available = await updater.check();

    await available?.install(() => {});

    expect(events).toEqual([
      "prepare_update_backup:0.2.0",
      "downloadAndInstall",
      "relaunch",
    ]);
  });

  it("reports bounded progress and supports an unknown total", async () => {
    const progress: UpdateProgress[] = [];
    const updater = createTauriUpdateService(
      dependencies({
        checkForUpdate: async () =>
          pluginUpdate(async (onEvent) => {
            onEvent({ event: "Started", data: { contentLength: 100 } });
            onEvent({ event: "Progress", data: { chunkLength: 25 } });
            onEvent({ event: "Progress", data: { chunkLength: 200 } });
            onEvent({ event: "Finished" });
          }),
      }),
    );
    const available = await updater.check();

    await available?.install((value) => progress.push(value));

    expect(progress).toEqual([
      { downloadedBytes: 0, totalBytes: 100, percent: 0 },
      { downloadedBytes: 25, totalBytes: 100, percent: 25 },
      { downloadedBytes: 225, totalBytes: 100, percent: 100 },
      { downloadedBytes: 100, totalBytes: 100, percent: 100 },
    ]);

    const unknownProgress: UpdateProgress[] = [];
    const unknownUpdater = createTauriUpdateService(
      dependencies({
        checkForUpdate: async () =>
          pluginUpdate(async (onEvent) => {
            onEvent({ event: "Started", data: {} });
            onEvent({ event: "Progress", data: { chunkLength: 50 } });
          }),
      }),
    );
    const unknown = await unknownUpdater.check();
    await unknown?.install((value) => unknownProgress.push(value));

    expect(unknownProgress[unknownProgress.length - 1]).toEqual({
      downloadedBytes: 50,
      totalBytes: null,
      percent: null,
    });
  });

  it("uses a generic error and never downloads or relaunches when backup fails", async () => {
    let downloaded = false;
    let relaunched = false;
    const updater = createTauriUpdateService(
      dependencies({
        invoke: async () => {
          throw new Error("C:\\private\\library.sqlite3");
        },
        checkForUpdate: async () =>
          pluginUpdate(async () => {
            downloaded = true;
          }),
        relaunch: async () => {
          relaunched = true;
        },
      }),
    );
    const available = await updater.check();

    await expect(available?.install(() => {})).rejects.toThrow("update_failed");
    expect(downloaded).toBe(false);
    expect(relaunched).toBe(false);
  });

  it("does not relaunch when download or installation fails", async () => {
    let relaunched = false;
    const updater = createTauriUpdateService(
      dependencies({
        checkForUpdate: async () =>
          pluginUpdate(async () => {
            throw new Error("remote internal detail");
          }),
        relaunch: async () => {
          relaunched = true;
        },
      }),
    );
    const available = await updater.check();

    await expect(available?.install(() => {})).rejects.toThrow("update_failed");
    expect(relaunched).toBe(false);
  });
});
