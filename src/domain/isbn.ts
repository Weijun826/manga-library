import type { ParsedIsbn } from "./model";

export function normalizeIsbn(raw: string): string {
  return raw.replace(/[\s-]/g, "").toUpperCase();
}

function isValidIsbn10(isbn: string): boolean {
  if (!/^\d{9}[\dX]$/.test(isbn)) return false;

  const sum = [...isbn].reduce((total, character, index) => {
    const value = character === "X" ? 10 : Number(character);
    return total + value * (10 - index);
  }, 0);

  return sum % 11 === 0;
}

function isValidIsbn13(isbn: string): boolean {
  if (!/^\d{13}$/.test(isbn)) return false;

  const sum = [...isbn].reduce((total, character, index) => {
    return total + Number(character) * (index % 2 === 0 ? 1 : 3);
  }, 0);

  return sum % 10 === 0;
}

export function parseIsbn(raw: string): ParsedIsbn {
  const normalized = normalizeIsbn(raw);

  if (normalized.length === 10 && isValidIsbn10(normalized)) {
    return { normalized, kind: "isbn10" };
  }

  if (normalized.length === 13 && isValidIsbn13(normalized)) {
    return { normalized, kind: "isbn13" };
  }

  throw new Error(/^\d{9}[\dX]$|^\d{13}$/.test(normalized) ? "invalid_checksum" : "invalid_format");
}
