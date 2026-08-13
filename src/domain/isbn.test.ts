import { describe, expect, it } from "vitest";
import { normalizeIsbn, parseIsbn } from "./isbn";

describe("parseIsbn", () => {
  it("normalizes a hyphenated ISBN-13", () => {
    expect(parseIsbn("978-957-26-9001-7")).toEqual({ normalized: "9789572690017", kind: "isbn13" });
  });

  it("accepts ISBN-10 ending in X", () => {
    expect(parseIsbn("0-8044-2957-X")).toEqual({ normalized: "080442957X", kind: "isbn10" });
  });

  it("rejects a bad checksum", () => {
    expect(() => parseIsbn("9789572690018")).toThrow("invalid_checksum");
  });

  it("rejects an invalid format", () => {
    expect(() => parseIsbn("978957269001X")).toThrow("invalid_format");
  });
});

describe("normalizeIsbn", () => {
  it("removes spaces and hyphens while uppercasing x", () => {
    expect(normalizeIsbn(" 0-8044 2957-x ")).toBe("080442957X");
  });
});
