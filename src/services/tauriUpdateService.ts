import { getVersion } from "@tauri-apps/api/app";
import { invoke } from "@tauri-apps/api/core";
import { relaunch } from "@tauri-apps/plugin-process";
import { check } from "@tauri-apps/plugin-updater";

import type { AvailableUpdate, UpdatePort, UpdateProgress } from "./updatePort";

type PluginDownloadEvent =
  | { event: "Started"; data: { contentLength?: number } }
  | { event: "Progress"; data: { chunkLength: number } }
  | { event: "Finished"; data?: never };

type PluginUpdate = {
  version: string;
  currentVersion: string;
  body?: string;
  date?: string;
  downloadAndInstall(onEvent: (event: PluginDownloadEvent) => void): Promise<void>;
};

type TauriUpdateDependencies = {
  getVersion(): Promise<string>;
  checkForUpdate(): Promise<PluginUpdate | null>;
  invoke(command: string, args?: Record<string, unknown>): Promise<unknown>;
  relaunch(): Promise<void>;
};

function updateFailed(): Error {
  return new Error("update_failed");
}

function safeTotal(value: number | undefined): number | null {
  return typeof value === "number" && Number.isFinite(value) && value >= 0 ? value : null;
}

function progressValue(downloadedBytes: number, totalBytes: number | null): UpdateProgress {
  return {
    downloadedBytes,
    totalBytes,
    percent:
      totalBytes && totalBytes > 0
        ? Math.min(100, Math.max(0, Math.round((downloadedBytes / totalBytes) * 100)))
        : null,
  };
}

export function createTauriUpdateService(dependencies: TauriUpdateDependencies): UpdatePort {
  return {
    async currentVersion() {
      try {
        return await dependencies.getVersion();
      } catch {
        throw updateFailed();
      }
    },

    async check() {
      let update: PluginUpdate | null;
      try {
        update = await dependencies.checkForUpdate();
      } catch {
        throw updateFailed();
      }
      if (!update) return null;

      const available: AvailableUpdate = {
        version: update.version,
        notes: update.body ?? "",
        async install(onProgress) {
          let downloadedBytes = 0;
          let totalBytes: number | null = null;
          try {
            await dependencies.invoke("prepare_update_backup", {
              currentVersion: update.currentVersion,
            });
            await update.downloadAndInstall((event) => {
              if (event.event === "Started") {
                downloadedBytes = 0;
                totalBytes = safeTotal(event.data.contentLength);
              } else if (event.event === "Progress") {
                const chunkLength = Number.isFinite(event.data.chunkLength)
                  ? Math.max(0, event.data.chunkLength)
                  : 0;
                downloadedBytes += chunkLength;
              } else if (event.event === "Finished" && totalBytes !== null) {
                downloadedBytes = totalBytes;
              }
              onProgress(progressValue(downloadedBytes, totalBytes));
            });
            await dependencies.relaunch();
          } catch {
            throw updateFailed();
          }
        },
      };
      return available;
    },
  };
}

export const tauriUpdateService = createTauriUpdateService({
  getVersion,
  checkForUpdate: async () => (await check()) as PluginUpdate | null,
  invoke,
  relaunch,
});
