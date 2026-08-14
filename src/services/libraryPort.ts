import type {
  AddVolumeInput,
  BackupInfo,
  CollectionItemPatch,
  CoverAsset,
  CreateSeriesBatchInput,
  DashboardSummary,
  RestoreResult,
  SeriesDetail,
  SeriesFilter,
  SeriesSummary,
  UpdateSeriesMetadataInput,
  UpdateVolumeDetailsInput,
  VolumeWithCollection,
} from "../domain/model";

export interface LibraryPort {
  getDashboard(): Promise<DashboardSummary>;
  listSeries(filter: SeriesFilter): Promise<SeriesSummary[]>;
  getSeriesDetail(seriesId: string): Promise<SeriesDetail>;
  createSeriesBatch(input: CreateSeriesBatchInput): Promise<SeriesDetail>;
  updateSeriesMetadata(
    seriesId: string,
    input: UpdateSeriesMetadataInput,
  ): Promise<SeriesDetail>;
  addVolume(editionId: string, input: AddVolumeInput): Promise<VolumeWithCollection>;
  updateCollectionItem(
    volumeId: string,
    patch: CollectionItemPatch,
  ): Promise<VolumeWithCollection>;
  updateVolumeDetails(
    volumeId: string,
    input: UpdateVolumeDetailsInput,
  ): Promise<VolumeWithCollection>;
  findVolumeByIsbn(isbn: string): Promise<VolumeWithCollection | null>;
  deleteSeries(seriesId: string): Promise<void>;
  exportBackup(destination: string): Promise<BackupInfo>;
  restoreBackup(source: string): Promise<RestoreResult>;
  setSeriesCover(seriesId: string, sourcePath: string): Promise<CoverAsset>;
  removeSeriesCover(seriesId: string): Promise<void>;
}
