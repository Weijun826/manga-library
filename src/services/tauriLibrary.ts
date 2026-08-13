import { invoke } from "@tauri-apps/api/core";

import type {
  BackupInfo,
  CoverAsset,
  DashboardSummary,
  RestoreResult,
  SeriesDetail,
  SeriesSummary,
  VolumeWithCollection,
} from "../domain/model";
import type { LibraryPort } from "./libraryPort";

export const tauriLibrary: LibraryPort = {
  getDashboard() {
    return invoke<DashboardSummary>("get_dashboard");
  },

  listSeries(filter) {
    return invoke<SeriesSummary[]>("list_series", { filter });
  },

  getSeriesDetail(seriesId) {
    return invoke<SeriesDetail>("get_series_detail", { seriesId });
  },

  createSeriesBatch(input) {
    return invoke<SeriesDetail>("create_series_batch", { input });
  },

  updateSeriesMetadata(seriesId, input) {
    return invoke<SeriesDetail>("update_series_metadata", { seriesId, input });
  },

  addVolume(editionId, input) {
    return invoke<VolumeWithCollection>("add_volume", { editionId, input });
  },

  updateCollectionItem(volumeId, patch) {
    return invoke<VolumeWithCollection>("update_collection_item", { volumeId, patch });
  },

  findVolumeByIsbn(isbn) {
    return invoke<VolumeWithCollection | null>("find_volume_by_isbn", { isbn });
  },

  deleteSeries(seriesId) {
    return invoke<void>("delete_series", { seriesId });
  },

  exportBackup(destination) {
    return invoke<BackupInfo>("export_backup", { destination });
  },

  restoreBackup(source) {
    return invoke<RestoreResult>("restore_backup", { source });
  },

  importCover(sourcePath) {
    return invoke<CoverAsset>("import_cover", { sourcePath });
  },

  clearUnusedCoverCache() {
    return invoke<number>("clear_unused_cover_cache");
  },
};
