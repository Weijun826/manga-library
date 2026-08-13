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
