import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";

import type {
  CreateSeriesBatchInput,
  DashboardSummary,
  SeriesDetail,
  SeriesSummary,
  VolumeWithCollection,
} from "../domain/model";
import type { LibraryPort } from "../services/libraryPort";
import { App } from "./App";

function summary(detail: SeriesDetail): SeriesSummary {
  const volumes = detail.editions.flatMap((edition) => edition.volumes);
  const owned = volumes.filter((volume) => volume.collection.isOwned).length;
  const read = volumes.filter((volume) => volume.collection.isRead).length;
  const missing = volumes.filter((volume) => volume.availabilityStatus === "released" && !volume.collection.isOwned).length;
  return { ...detail, ownedVolumeCount: owned, readVolumeCount: read, missingVolumeCount: missing };
}

function detailFixture(): SeriesDetail {
  const volume: VolumeWithCollection = {
    id: "vol-1", editionId: "edition-1", displayLabel: "1", sortKey: "0:000001.000", titleOverride: null,
    isbn10: null, isbn13: null, translator: null, availabilityStatus: "released", releaseDate: null,
    releaseDatePrecision: "unknown", listPriceAmount: null, listPriceCurrency: null, cover: null,
    collection: { isOwned: false, isRead: false, isWishlisted: true, purchasePriceAmount: null, purchasePriceCurrency: null, acquiredOn: null, condition: "unknown", storageLocation: null, notes: null },
  };
  return { id: "series-1", title: "測試漫畫", originalTitle: null, publicationStatus: "ongoing", contributors: [{ name: "測試作者", role: "author" }], publishers: ["測試出版社"], representativeCover: null, knownVolumeCount: 1, ownedVolumeCount: 0, readVolumeCount: 0, missingVolumeCount: 1, description: "可互動的測試資料。", editions: [{ id: "edition-1", name: "單行本", languageCode: "ja", regionCode: "JP", publisher: "測試出版社", format: "tankobon", releaseStatus: "ongoing", knownVolumeCount: 1, volumes: [volume] }] };
}

class MemoryLibrary implements LibraryPort {
  private details: SeriesDetail[];
  private nextId = 2;
  constructor(initial: SeriesDetail[] = []) { this.details = initial; }
  async getDashboard(): Promise<DashboardSummary> {
    const summaries = this.details.map(summary);
    return { seriesCount: summaries.length, ownedVolumeCount: summaries.reduce((total, item) => total + item.ownedVolumeCount, 0), readVolumeCount: summaries.reduce((total, item) => total + item.readVolumeCount, 0), missingVolumeCount: summaries.reduce((total, item) => total + item.missingVolumeCount, 0), recentVolumes: [], incompleteSeries: summaries.filter((item) => item.missingVolumeCount > 0) };
  }
  async listSeries(filter: { query: string }): Promise<SeriesSummary[]> {
    const query = filter.query.toLowerCase();
    return this.details.map(summary).filter((item) => !query || item.title.toLowerCase().includes(query) || item.contributors.some((contributor) => contributor.name.toLowerCase().includes(query)));
  }
  async getSeriesDetail(seriesId: string): Promise<SeriesDetail> {
    const detail = this.details.find((item) => item.id === seriesId);
    if (!detail) throw new Error("not found");
    return detail;
  }
  async createSeriesBatch(input: CreateSeriesBatchInput): Promise<SeriesDetail> {
    const id = `series-${this.nextId++}`;
    const detail: SeriesDetail = { id, title: input.series.title, originalTitle: input.series.originalTitle, publicationStatus: input.series.publicationStatus, contributors: input.series.contributors.map(({ name, role }) => ({ name, role })), publishers: [input.edition.publisher], representativeCover: null, knownVolumeCount: input.volumes.length, ownedVolumeCount: 0, readVolumeCount: 0, missingVolumeCount: input.volumes.length, description: input.series.description, editions: [{ id: `${id}-edition`, name: input.edition.name, languageCode: input.edition.languageCode, regionCode: input.edition.regionCode, publisher: input.edition.publisher, format: input.edition.format, releaseStatus: input.edition.releaseStatus, knownVolumeCount: input.edition.knownVolumeCount, volumes: input.volumes.map((volume, index) => ({ ...volume, id: `${id}-volume-${index + 1}`, editionId: `${id}-edition`, cover: null })) }] };
    this.details.push(detail);
    return detail;
  }
  async updateCollectionItem(volumeId: string, patch: Partial<VolumeWithCollection["collection"]>): Promise<VolumeWithCollection> {
    for (const detail of this.details) for (const edition of detail.editions) for (const volume of edition.volumes) if (volume.id === volumeId) {
      volume.collection = { ...volume.collection, ...patch };
      if (volume.collection.isOwned) volume.collection.isWishlisted = false;
      return volume;
    }
    throw new Error("not found");
  }
  async findVolumeByIsbn(): Promise<VolumeWithCollection | null> { return null; }
  async deleteSeries(seriesId: string): Promise<void> { this.details = this.details.filter((detail) => detail.id !== seriesId); }
  async exportBackup(): Promise<never> { throw new Error("not implemented"); }
  async restoreBackup(): Promise<never> { throw new Error("not implemented"); }
  async importCover(): Promise<never> { throw new Error("not implemented"); }
  async clearUnusedCoverCache(): Promise<number> { return 0; }
}

describe("App MVP integration", () => {
  it("shows an empty dashboard and offers manual creation", async () => {
    const user = userEvent.setup();
    render(<App library={new MemoryLibrary()} />);
    expect(await screen.findByRole("heading", { name: "漫畫書庫" })).toBeVisible();
    expect(screen.getByRole("heading", { name: "還沒有漫畫" })).toBeVisible();
    await user.click(screen.getAllByRole("button", { name: "新增漫畫" })[0]);
    expect(screen.getByRole("heading", { name: "新增漫畫" })).toBeVisible();
  });

  it("creates volumes manually and opens the returned series detail", async () => {
    const user = userEvent.setup();
    render(<App library={new MemoryLibrary()} />);
    await screen.findByRole("heading", { name: "還沒有漫畫" });
    await user.click(screen.getAllByRole("button", { name: "新增漫畫" })[0]);
    await user.type(screen.getByLabelText("系列名稱 *"), "手動新漫畫");
    await user.type(screen.getByLabelText("作者 *"), "手動作者");
    await user.type(screen.getByLabelText("出版社 *"), "手動出版社");
    await user.type(screen.getByLabelText("冊數 *"), "3");
    await user.click(screen.getByRole("button", { name: "建立系列" }));
    expect(await screen.findByRole("heading", { name: "手動新漫畫" })).toBeVisible();
    expect(screen.getByText("單行本")).toBeVisible();
    expect(screen.getAllByRole("button", { name: "擁有" })).toHaveLength(3);
  });

  it("searches by title and author in the real library view", async () => {
    const user = userEvent.setup();
    render(<App library={new MemoryLibrary([detailFixture()])} />);
    await screen.findByRole("heading", { name: "漫畫書庫" });
    await user.click(screen.getByRole("button", { name: "我的漫畫" }));
    expect(await screen.findByText("測試漫畫")).toBeVisible();
    await user.type(screen.getByLabelText("搜尋"), "測試作者");
    expect(await screen.findByText("測試作者 · 測試出版社")).toBeVisible();
    await user.clear(screen.getByLabelText("搜尋"));
    await user.type(screen.getByLabelText("搜尋"), "不存在");
    expect(await screen.findByRole("heading", { name: "找不到符合的漫畫" })).toBeVisible();
  });

  it("opens a series and persists ownership while closing wishlist", async () => {
    const user = userEvent.setup();
    render(<App library={new MemoryLibrary([detailFixture()])} />);
    await screen.findByRole("heading", { name: "漫畫書庫" });
    await user.click(screen.getByRole("button", { name: "我的漫畫" }));
    await user.click(await screen.findByRole("button", { name: /測試漫畫/ }));
    expect(await screen.findByRole("heading", { name: "測試漫畫" })).toBeVisible();
    await user.click(screen.getByRole("button", { name: "擁有" }));
    await waitFor(() => expect(screen.getByRole("button", { name: "擁有" })).toHaveAttribute("aria-pressed", "true"));
    expect(screen.getByRole("button", { name: "願望" })).toBeDisabled();
  });

  it("requires an in-app confirmation before deleting a series", async () => {
    const user = userEvent.setup();
    render(<App library={new MemoryLibrary([detailFixture()])} />);
    await screen.findByRole("heading", { name: "漫畫書庫" });
    await user.click(screen.getByRole("button", { name: "我的漫畫" }));
    await user.click(await screen.findByRole("button", { name: /測試漫畫/ }));
    await user.click(await screen.findByRole("button", { name: "刪除系列" }));
    expect(screen.getByRole("dialog", { name: "刪除這個系列？" })).toBeVisible();
    await user.click(screen.getByRole("button", { name: "確認刪除" }));
    expect(await screen.findByRole("heading", { name: "還沒有漫畫" })).toBeVisible();
  });
});
