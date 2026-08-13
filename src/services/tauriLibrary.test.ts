import { beforeEach, describe, expect, it, vi } from "vitest";

import type {
  BackupInfo,
  CollectionItemPatch,
  CollectionItemView,
  CoverAsset,
  CreateSeriesBatchInput,
  DashboardSummary,
  RestoreResult,
  SeriesDetail,
  SeriesFilter,
  SeriesSummary,
  VolumeWithCollection,
} from "../domain/model";
import { tauriLibrary } from "./tauriLibrary";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));

vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

const coverFixture: CoverAsset = {
  id: "cover-1",
  relativePath: "covers/cover-1.webp",
  mimeType: "image/webp",
  byteSize: 512,
  sha256: "a".repeat(64),
};

const collectionFixture: CollectionItemView = {
  isOwned: true,
  isRead: false,
  isWishlisted: false,
  purchasePriceAmount: 120,
  purchasePriceCurrency: "TWD",
  acquiredOn: "2026-08-13",
  condition: "good",
  storageLocation: "書櫃 A",
  notes: null,
};

const volumeFixture: VolumeWithCollection = {
  id: "volume-1",
  editionId: "edition-1",
  displayLabel: "1",
  sortKey: "0:000001.000",
  titleOverride: null,
  isbn10: "080442957X",
  isbn13: "9789572690017",
  translator: "測試譯者",
  availabilityStatus: "released",
  releaseDate: "2026-08-13",
  releaseDatePrecision: "day",
  listPriceAmount: 140,
  listPriceCurrency: "TWD",
  cover: coverFixture,
  collection: collectionFixture,
};

const seriesSummaryFixture: SeriesSummary = {
  id: "series-1",
  title: "海風冒險譚",
  originalTitle: "Sea Breeze Adventures",
  publicationStatus: "completed",
  contributors: [{ name: "測試作者", role: "author" }],
  publishers: ["東立"],
  representativeCover: coverFixture,
  knownVolumeCount: 1,
  ownedVolumeCount: 1,
  readVolumeCount: 0,
  missingVolumeCount: 0,
};

const seriesDetailFixture: SeriesDetail = {
  ...seriesSummaryFixture,
  description: "測試系列",
  editions: [
    {
      id: "edition-1",
      name: "台灣單行本",
      languageCode: "zh-Hant",
      regionCode: "TW",
      publisher: "東立",
      format: "tankobon",
      releaseStatus: "completed",
      knownVolumeCount: 1,
      volumes: [volumeFixture],
    },
  ],
};

const dashboardFixture: DashboardSummary = {
  seriesCount: 1,
  ownedVolumeCount: 1,
  readVolumeCount: 0,
  missingVolumeCount: 0,
  recentVolumes: [volumeFixture],
  incompleteSeries: [],
};

const filterFixture: SeriesFilter = {
  query: "海風",
  publicationStatus: "completed",
  collection: "complete",
  reading: "incomplete",
};

const batchInputFixture: CreateSeriesBatchInput = {
  series: {
    title: "海風冒險譚",
    originalTitle: "Sea Breeze Adventures",
    description: "測試系列",
    publicationStatus: "completed",
    contributors: [{ name: "測試作者", role: "author", sortOrder: 0 }],
  },
  edition: {
    name: "台灣單行本",
    languageCode: "zh-Hant",
    regionCode: "TW",
    publisher: "東立",
    format: "tankobon",
    releaseStatus: "completed",
    knownVolumeCount: 1,
  },
  volumes: [
    {
      displayLabel: "1",
      sortKey: "0:000001.000",
      titleOverride: null,
      isbn10: "080442957X",
      isbn13: "9789572690017",
      translator: "測試譯者",
      availabilityStatus: "released",
      releaseDate: "2026-08-13",
      releaseDatePrecision: "day",
      listPriceAmount: 140,
      listPriceCurrency: "TWD",
      collection: collectionFixture,
      provenance: { displayLabel: "manual", isbn13: "google_books" },
    },
  ],
};

describe("tauriLibrary", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("maps getDashboard to the stable Tauri command", async () => {
    invokeMock.mockResolvedValue(dashboardFixture);

    await expect(tauriLibrary.getDashboard()).resolves.toBe(dashboardFixture);

    expect(invokeMock).toHaveBeenCalledOnce();
    expect(invokeMock).toHaveBeenCalledWith("get_dashboard");
  });

  it("maps listSeries with the exact filter envelope", async () => {
    const summaries = [seriesSummaryFixture];
    invokeMock.mockResolvedValue(summaries);

    await expect(tauriLibrary.listSeries(filterFixture)).resolves.toBe(summaries);

    expect(invokeMock).toHaveBeenCalledOnce();
    expect(invokeMock).toHaveBeenCalledWith("list_series", { filter: filterFixture });
  });

  it("maps getSeriesDetail with a camelCase series ID", async () => {
    invokeMock.mockResolvedValue(seriesDetailFixture);

    await expect(tauriLibrary.getSeriesDetail("series-1")).resolves.toBe(seriesDetailFixture);

    expect(invokeMock).toHaveBeenCalledOnce();
    expect(invokeMock).toHaveBeenCalledWith("get_series_detail", { seriesId: "series-1" });
  });

  it("maps createSeriesBatch to the stable Tauri command", async () => {
    invokeMock.mockResolvedValue(seriesDetailFixture);

    await expect(tauriLibrary.createSeriesBatch(batchInputFixture)).resolves.toBe(
      seriesDetailFixture,
    );

    expect(invokeMock).toHaveBeenCalledOnce();
    expect(invokeMock).toHaveBeenCalledWith("create_series_batch", {
      input: batchInputFixture,
    });
  });

  it("maps updateCollectionItem with the volume ID and partial patch", async () => {
    const patch: CollectionItemPatch = {
      isOwned: true,
      isWishlisted: false,
      notes: null,
    };
    invokeMock.mockResolvedValue(volumeFixture);

    await expect(tauriLibrary.updateCollectionItem("volume-1", patch)).resolves.toBe(
      volumeFixture,
    );

    expect(invokeMock).toHaveBeenCalledOnce();
    expect(invokeMock).toHaveBeenCalledWith("update_collection_item", {
      volumeId: "volume-1",
      patch,
    });
  });

  it("maps findVolumeByIsbn with the normalized ISBN", async () => {
    invokeMock.mockResolvedValue(volumeFixture);

    await expect(tauriLibrary.findVolumeByIsbn("9789572690017")).resolves.toBe(volumeFixture);

    expect(invokeMock).toHaveBeenCalledOnce();
    expect(invokeMock).toHaveBeenCalledWith("find_volume_by_isbn", {
      isbn: "9789572690017",
    });
  });

  it("maps deleteSeries and resolves void", async () => {
    invokeMock.mockResolvedValue(undefined);

    await expect(tauriLibrary.deleteSeries("series-1")).resolves.toBeUndefined();

    expect(invokeMock).toHaveBeenCalledOnce();
    expect(invokeMock).toHaveBeenCalledWith("delete_series", { seriesId: "series-1" });
  });

  it("maps exportBackup with its destination", async () => {
    const backup: BackupInfo = {
      path: "D:/Backups/manga-library.zip",
      createdAt: "2026-08-13T12:00:00Z",
      sha256: "b".repeat(64),
      byteSize: 2048,
    };
    invokeMock.mockResolvedValue(backup);

    await expect(tauriLibrary.exportBackup("D:/Backups")).resolves.toBe(backup);

    expect(invokeMock).toHaveBeenCalledOnce();
    expect(invokeMock).toHaveBeenCalledWith("export_backup", { destination: "D:/Backups" });
  });

  it("maps restoreBackup with its source", async () => {
    const restored: RestoreResult = {
      restoredAt: "2026-08-13T12:30:00Z",
      preRestoreBackupPath: "D:/Backups/pre-restore.zip",
    };
    invokeMock.mockResolvedValue(restored);

    await expect(tauriLibrary.restoreBackup("D:/Backups/manga-library.zip")).resolves.toBe(
      restored,
    );

    expect(invokeMock).toHaveBeenCalledOnce();
    expect(invokeMock).toHaveBeenCalledWith("restore_backup", {
      source: "D:/Backups/manga-library.zip",
    });
  });

  it("maps importCover with a camelCase source path", async () => {
    invokeMock.mockResolvedValue(coverFixture);

    await expect(tauriLibrary.importCover("D:/Covers/cover.webp")).resolves.toBe(coverFixture);

    expect(invokeMock).toHaveBeenCalledOnce();
    expect(invokeMock).toHaveBeenCalledWith("import_cover", {
      sourcePath: "D:/Covers/cover.webp",
    });
  });

  it("maps clearUnusedCoverCache and returns the deletion count", async () => {
    invokeMock.mockResolvedValue(3);

    await expect(tauriLibrary.clearUnusedCoverCache()).resolves.toBe(3);

    expect(invokeMock).toHaveBeenCalledOnce();
    expect(invokeMock).toHaveBeenCalledWith("clear_unused_cover_cache");
  });
});
