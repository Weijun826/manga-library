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
