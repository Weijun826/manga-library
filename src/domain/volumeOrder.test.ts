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

  it("rejects an empty label", () => {
    expect(() => makeVolumeSortKey("")).toThrow();
  });
});
