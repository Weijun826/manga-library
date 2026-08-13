export type PublicationStatus = "ongoing" | "completed" | "hiatus" | "unknown";
export type EditionFormat = "tankobon" | "omnibus" | "deluxe" | "box_set" | "other";
export type AvailabilityStatus = "released" | "upcoming" | "unknown";
export type DatePrecision = "day" | "month" | "year" | "unknown";
export type BookCondition = "new" | "like_new" | "good" | "fair" | "poor" | "unknown";
export type MetadataSource = "manual" | "google_books" | "open_library" | "ncl_import";

export interface ParsedIsbn {
  normalized: string;
  kind: "isbn10" | "isbn13";
}

export interface CollectionFlags {
  isOwned: boolean;
  isRead: boolean;
  isWishlisted: boolean;
}

export interface SeriesFilter {
  query: string;
  publicationStatus: PublicationStatus | "all";
  collection: "all" | "complete" | "incomplete";
  reading: "all" | "complete" | "incomplete";
}

export interface DashboardSummary {
  seriesCount: number;
  ownedVolumeCount: number;
  readVolumeCount: number;
  missingVolumeCount: number;
  recentVolumes: VolumeWithCollection[];
  incompleteSeries: SeriesSummary[];
}

export interface SeriesSummary {
  id: string;
  title: string;
  originalTitle: string | null;
  publicationStatus: PublicationStatus;
  contributors: Array<{ name: string; role: "author" | "story" | "art" }>;
  publishers: string[];
  representativeCover: CoverAsset | null;
  knownVolumeCount: number;
  ownedVolumeCount: number;
  readVolumeCount: number;
  missingVolumeCount: number;
}

export interface CoverAsset {
  id: string;
  relativePath: string;
  mimeType: "image/jpeg" | "image/png" | "image/webp";
  byteSize: number;
  sha256: string;
}

export interface CollectionItemView extends CollectionFlags {
  purchasePriceAmount: number | null;
  purchasePriceCurrency: string | null;
  acquiredOn: string | null;
  condition: BookCondition;
  storageLocation: string | null;
  notes: string | null;
}

export interface VolumeWithCollection {
  id: string;
  editionId: string;
  displayLabel: string;
  sortKey: string;
  titleOverride: string | null;
  isbn10: string | null;
  isbn13: string | null;
  translator: string | null;
  availabilityStatus: AvailabilityStatus;
  releaseDate: string | null;
  releaseDatePrecision: DatePrecision;
  listPriceAmount: number | null;
  listPriceCurrency: string | null;
  cover: CoverAsset | null;
  collection: CollectionItemView;
}

export interface EditionDetail {
  id: string;
  name: string;
  languageCode: string;
  regionCode: string;
  publisher: string;
  format: EditionFormat;
  releaseStatus: PublicationStatus;
  knownVolumeCount: number | null;
  volumes: VolumeWithCollection[];
}

export interface SeriesDetail extends SeriesSummary {
  description: string | null;
  editions: EditionDetail[];
}

export interface CreateSeriesBatchInput {
  series: {
    title: string;
    originalTitle: string | null;
    description: string | null;
    publicationStatus: PublicationStatus;
    contributors: Array<{
      name: string;
      role: "author" | "story" | "art";
      sortOrder: number;
    }>;
  };
  edition: {
    name: string;
    languageCode: string;
    regionCode: string;
    publisher: string;
    format: EditionFormat;
    releaseStatus: PublicationStatus;
    knownVolumeCount: number | null;
  };
  volumes: Array<{
    displayLabel: string;
    sortKey: string;
    titleOverride: string | null;
    isbn10: string | null;
    isbn13: string | null;
    translator: string | null;
    availabilityStatus: AvailabilityStatus;
    releaseDate: string | null;
    releaseDatePrecision: DatePrecision;
    listPriceAmount: number | null;
    listPriceCurrency: string | null;
    collection: CollectionItemView;
    provenance: Record<string, MetadataSource>;
  }>;
}

export type CollectionItemPatch = Partial<CollectionItemView>;

export interface BackupInfo {
  path: string;
  createdAt: string;
  sha256: string;
  byteSize: number;
}

export interface RestoreResult {
  restoredAt: string;
  preRestoreBackupPath: string;
}
