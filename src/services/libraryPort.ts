import type {
  BackupInfo,
  CollectionItemPatch,
  CoverAsset,
  CreateSeriesBatchInput,
  DashboardSummary,
  RestoreResult,
  SeriesDetail,
  SeriesFilter,
  SeriesSummary,
  VolumeWithCollection,
} from "../domain/model";

export interface LibraryPort {
  getDashboard(): Promise<DashboardSummary>;
  listSeries(filter: SeriesFilter): Promise<SeriesSummary[]>;
  getSeriesDetail(seriesId: string): Promise<SeriesDetail>;
  createSeriesBatch(input: CreateSeriesBatchInput): Promise<SeriesDetail>;
  updateCollectionItem(
    volumeId: string,
    patch: CollectionItemPatch,
  ): Promise<VolumeWithCollection>;
  findVolumeByIsbn(isbn: string): Promise<VolumeWithCollection | null>;
  deleteSeries(seriesId: string): Promise<void>;
  exportBackup(destination: string): Promise<BackupInfo>;
  restoreBackup(source: string): Promise<RestoreResult>;
  importCover(sourcePath: string): Promise<CoverAsset>;
  clearUnusedCoverCache(): Promise<number>;
}
