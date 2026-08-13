import { beforeEach, describe, expect, it, vi } from "vitest";

import { selectCoverImage } from "./coverPicker";

const { openMock } = vi.hoisted(() => ({ openMock: vi.fn() }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: openMock }));

describe("selectCoverImage", () => {
  beforeEach(() => openMock.mockReset());

  it("selects one local JPG, PNG, or WebP image", async () => {
    openMock.mockResolvedValue("D:\\covers\\manga.webp");

    await expect(selectCoverImage()).resolves.toBe("D:\\covers\\manga.webp");

    expect(openMock).toHaveBeenCalledWith({
      multiple: false,
      directory: false,
      filters: [{ name: "漫畫封面", extensions: ["jpg", "jpeg", "png", "webp"] }],
    });
  });

  it("returns null when selection is cancelled", async () => {
    openMock.mockResolvedValue(null);
    await expect(selectCoverImage()).resolves.toBeNull();
  });
});
