import { describe, expect, it } from "vitest";
import { compareVolumeLabels, makeVolumeSortKey } from "./volumeOrder";

describe("volume ordering", () => {
  it("orders numeric, decimal, directional and one-shot labels", () => {
    const labels = ["10", "下", "1.5", "全一冊", "2", "上", "1"];
    expect(labels.sort(compareVolumeLabels)).toEqual(["1", "1.5", "2", "10", "上", "下", "全一冊"]);
  });

  it("creates equal-width numeric sort keys", () => {
    expect(makeVolumeSortKey("2")).toBe("0:000002.000");
  });

  it("normalizes leading zeros in numeric labels", () => {
    expect(makeVolumeSortKey("0000002")).toBe("0:000002.000");
  });

  it("accepts the six-digit numeric boundary", () => {
    expect(makeVolumeSortKey("999999")).toBe("0:999999.000");
  });

  it("rejects seven-digit numeric labels", () => {
    expect(() => makeVolumeSortKey("1000000")).toThrow("invalid_volume_label");
  });

  it("trims fallback labels", () => {
    expect(makeVolumeSortKey(" 特裝版 ")).toBe("9:特裝版");
  });

  it("rejects an empty label", () => {
    expect(() => makeVolumeSortKey("")).toThrow();
  });
});
