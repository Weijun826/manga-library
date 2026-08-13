CREATE TABLE IF NOT EXISTS schema_migrations (
  version INTEGER PRIMARY KEY NOT NULL,
  name TEXT NOT NULL UNIQUE,
  applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE series (
  id TEXT PRIMARY KEY NOT NULL,
  title TEXT NOT NULL,
  original_title TEXT,
  description TEXT,
  publication_status TEXT NOT NULL DEFAULT 'unknown'
    CHECK (publication_status IN ('ongoing','completed','hiatus','unknown')),
  representative_cover_asset_id TEXT REFERENCES cover_assets(id) ON DELETE SET NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE contributors (
  id TEXT PRIMARY KEY NOT NULL,
  display_name TEXT NOT NULL
);

CREATE TABLE series_contributors (
  series_id TEXT NOT NULL REFERENCES series(id) ON DELETE CASCADE,
  contributor_id TEXT NOT NULL REFERENCES contributors(id) ON DELETE CASCADE,
  role TEXT NOT NULL CHECK (role IN ('author','story','art')),
  sort_order INTEGER NOT NULL DEFAULT 0 CHECK (sort_order >= 0),
  PRIMARY KEY (series_id, contributor_id, role)
);

CREATE TABLE editions (
  id TEXT PRIMARY KEY NOT NULL,
  series_id TEXT NOT NULL REFERENCES series(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  language_code TEXT NOT NULL,
  region_code TEXT NOT NULL,
  publisher TEXT NOT NULL,
  format TEXT NOT NULL CHECK (format IN ('tankobon','omnibus','deluxe','box_set','other')),
  release_status TEXT NOT NULL DEFAULT 'unknown'
    CHECK (release_status IN ('ongoing','completed','hiatus','unknown')),
  known_volume_count INTEGER CHECK (known_volume_count IS NULL OR known_volume_count >= 0),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE cover_assets (
  id TEXT PRIMARY KEY NOT NULL,
  source_type TEXT NOT NULL CHECK (source_type IN ('remote_cache','user_file','generated')),
  source_url TEXT,
  relative_path TEXT NOT NULL,
  mime_type TEXT NOT NULL,
  byte_size INTEGER NOT NULL CHECK (byte_size >= 0),
  sha256 TEXT NOT NULL UNIQUE,
  created_at TEXT NOT NULL
);

CREATE TABLE volumes (
  id TEXT PRIMARY KEY NOT NULL,
  edition_id TEXT NOT NULL REFERENCES editions(id) ON DELETE CASCADE,
  display_label TEXT NOT NULL,
  sort_key TEXT NOT NULL,
  title_override TEXT,
  isbn_10 TEXT,
  isbn_13 TEXT,
  translator TEXT,
  availability_status TEXT NOT NULL DEFAULT 'unknown'
    CHECK (availability_status IN ('released','upcoming','unknown')),
  release_date TEXT,
  release_date_precision TEXT NOT NULL DEFAULT 'unknown'
    CHECK (release_date_precision IN ('day','month','year','unknown')),
  list_price_amount INTEGER CHECK (list_price_amount IS NULL OR list_price_amount >= 0),
  list_price_currency TEXT,
  cover_asset_id TEXT REFERENCES cover_assets(id) ON DELETE SET NULL,
  metadata_source TEXT NOT NULL DEFAULT 'manual'
    CHECK (metadata_source IN ('manual','google_books','open_library','ncl_import')),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE (edition_id, sort_key, display_label)
);

CREATE UNIQUE INDEX volumes_isbn_10_unique
  ON volumes (isbn_10)
  WHERE isbn_10 IS NOT NULL;

CREATE UNIQUE INDEX volumes_isbn_13_unique
  ON volumes (isbn_13)
  WHERE isbn_13 IS NOT NULL;

CREATE TABLE collection_items (
  id TEXT PRIMARY KEY NOT NULL,
  volume_id TEXT NOT NULL UNIQUE REFERENCES volumes(id) ON DELETE CASCADE,
  is_owned INTEGER NOT NULL DEFAULT 0 CHECK (is_owned IN (0, 1)),
  is_read INTEGER NOT NULL DEFAULT 0 CHECK (is_read IN (0, 1)),
  is_wishlisted INTEGER NOT NULL DEFAULT 0 CHECK (is_wishlisted IN (0, 1)),
  purchase_price_amount INTEGER CHECK (purchase_price_amount IS NULL OR purchase_price_amount >= 0),
  purchase_price_currency TEXT,
  acquired_on TEXT,
  condition TEXT NOT NULL DEFAULT 'unknown' CHECK (condition IN ('new','like_new','good','fair','poor','unknown')),
  storage_location TEXT,
  notes TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  CHECK (NOT (is_owned = 1 AND is_wishlisted = 1))
);

CREATE TABLE metadata_cache (
  query_key TEXT PRIMARY KEY NOT NULL,
  source TEXT NOT NULL CHECK (source IN ('manual','google_books','open_library','ncl_import')),
  response_json TEXT NOT NULL,
  retrieved_at TEXT NOT NULL,
  expires_at TEXT NOT NULL
);

CREATE TABLE metadata_provenance (
  entity_type TEXT NOT NULL,
  entity_id TEXT NOT NULL,
  field_name TEXT NOT NULL,
  source TEXT NOT NULL CHECK (source IN ('manual','google_books','open_library','ncl_import')),
  external_id TEXT,
  retrieved_at TEXT NOT NULL,
  PRIMARY KEY (entity_type, entity_id, field_name)
);
