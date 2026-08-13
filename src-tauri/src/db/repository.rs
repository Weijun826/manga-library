use std::collections::BTreeMap;

use rusqlite::{params, OptionalExtension, Row, Transaction};
use uuid::Uuid;

use crate::error::AppError;

use super::{models::*, Database};

const VOLUME_SELECT: &str = "SELECT \
    v.id, v.edition_id, v.display_label, v.sort_key, v.title_override, \
    v.isbn_10, v.isbn_13, v.translator, v.availability_status, v.release_date, \
    v.release_date_precision, v.list_price_amount, v.list_price_currency, \
    ca.id, ca.relative_path, ca.mime_type, ca.byte_size, ca.sha256, \
    COALESCE(ci.is_owned, 0), COALESCE(ci.is_read, 0), \
    COALESCE(ci.is_wishlisted, 0), ci.purchase_price_amount, \
    ci.purchase_price_currency, ci.acquired_on, COALESCE(ci.condition, 'unknown'), \
    ci.storage_location, ci.notes \
  FROM volumes v \
  LEFT JOIN cover_assets ca ON ca.id = v.cover_asset_id \
  LEFT JOIN collection_items ci ON ci.volume_id = v.id";

pub fn get_dashboard(database: &Database) -> Result<DashboardSummary, AppError> {
    database.with_transaction(get_dashboard_in_transaction)
}

pub fn list_series(
    database: &Database,
    filter: SeriesFilter,
) -> Result<Vec<SeriesSummary>, AppError> {
    database.with_transaction(|transaction| list_series_in_transaction(transaction, &filter))
}

pub fn get_series_detail(database: &Database, series_id: &str) -> Result<SeriesDetail, AppError> {
    database
        .with_transaction(|transaction| get_series_detail_in_transaction(transaction, series_id))
}

pub fn create_series_batch(
    database: &Database,
    input: CreateSeriesBatchInput,
) -> Result<SeriesDetail, AppError> {
    let series_id = new_uuid();
    let edition_id = new_uuid();
    let contributor_ids = input
        .series
        .contributors
        .iter()
        .map(|_| new_uuid())
        .collect::<Vec<_>>();
    let volume_ids = input.volumes.iter().map(|_| new_uuid()).collect::<Vec<_>>();
    let collection_ids = input.volumes.iter().map(|_| new_uuid()).collect::<Vec<_>>();

    database.with_transaction(|transaction| {
        transaction.execute(
            "INSERT INTO series (\
                id, title, original_title, description, publication_status, created_at, updated_at\
             ) VALUES (?1, ?2, ?3, ?4, ?5, \
                strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), \
                strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
            params![
                series_id,
                input.series.title,
                input.series.original_title,
                input.series.description,
                input.series.publication_status.as_str(),
            ],
        )?;

        for (index, contributor) in input.series.contributors.iter().enumerate() {
            transaction.execute(
                "INSERT INTO contributors (id, display_name) VALUES (?1, ?2)",
                params![contributor_ids[index], contributor.name],
            )?;
            transaction.execute(
                "INSERT INTO series_contributors (\
                    series_id, contributor_id, role, sort_order\
                 ) VALUES (?1, ?2, ?3, ?4)",
                params![
                    series_id,
                    contributor_ids[index],
                    contributor.role.as_str(),
                    contributor.sort_order,
                ],
            )?;
        }

        transaction.execute(
            "INSERT INTO editions (\
                id, series_id, name, language_code, region_code, publisher, format, \
                release_status, known_volume_count, created_at, updated_at\
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, \
                strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), \
                strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
            params![
                edition_id,
                series_id,
                input.edition.name,
                input.edition.language_code,
                input.edition.region_code,
                input.edition.publisher,
                input.edition.format.as_str(),
                input.edition.release_status.as_str(),
                input.edition.known_volume_count,
            ],
        )?;

        for (index, volume) in input.volumes.iter().enumerate() {
            let volume_id = &volume_ids[index];
            transaction.execute(
                "INSERT INTO volumes (\
                    id, edition_id, display_label, sort_key, title_override, isbn_10, isbn_13, \
                    translator, availability_status, release_date, release_date_precision, \
                    list_price_amount, list_price_currency, metadata_source, created_at, updated_at\
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, \
                    ?14, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), \
                    strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
                params![
                    volume_id,
                    edition_id,
                    volume.display_label,
                    volume.sort_key,
                    volume.title_override,
                    volume.isbn_10,
                    volume.isbn_13,
                    volume.translator,
                    volume.availability_status.as_str(),
                    volume.release_date,
                    volume.release_date_precision.as_str(),
                    volume.list_price_amount,
                    volume.list_price_currency,
                    MetadataSource::Manual.as_str(),
                ],
            )?;

            let collection = enforce_ownership_rule(volume.collection.clone());
            insert_collection_item(transaction, &collection_ids[index], volume_id, &collection)?;

            for (field_name, source) in &volume.provenance {
                transaction.execute(
                    "INSERT INTO metadata_provenance (\
                        entity_type, entity_id, field_name, source, external_id, retrieved_at\
                     ) VALUES (?1, ?2, ?3, ?4, NULL, \
                        strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
                    params!["volume", volume_id, field_name, source.as_str()],
                )?;
            }
        }

        Ok(())
    })?;

    get_series_detail(database, &series_id)
}

pub fn update_series_metadata(
    database: &Database,
    series_id: &str,
    input: UpdateSeriesMetadataInput,
) -> Result<SeriesDetail, AppError> {
    let title = required_text(&input.title, "invalid_series_metadata")?;
    let author = required_text(&input.author, "invalid_series_metadata")?;
    let edition_name = required_text(&input.edition_name, "invalid_series_metadata")?;
    let publisher = required_text(&input.publisher, "invalid_series_metadata")?;
    let original_title = optional_text(input.original_title);
    let description = optional_text(input.description);

    database.with_transaction(|transaction| {
        let contributor_id: String = transaction.query_row(
            "SELECT sc.contributor_id FROM series_contributors sc \
             WHERE sc.series_id = ?1 AND sc.role = 'author' \
             ORDER BY sc.sort_order, sc.contributor_id LIMIT 1",
            [series_id],
            |row| row.get(0),
        )?;
        transaction.execute(
            "UPDATE series SET title = ?2, original_title = ?3, description = ?4, \
                publication_status = ?5, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
             WHERE id = ?1",
            params![
                series_id,
                title,
                original_title,
                description,
                input.publication_status.as_str()
            ],
        )?;
        transaction.execute(
            "UPDATE contributors SET display_name = ?2 WHERE id = ?1",
            params![contributor_id, author],
        )?;
        let updated_edition = transaction.execute(
            "UPDATE editions SET name = ?3, publisher = ?4, \
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
             WHERE id = ?1 AND series_id = ?2",
            params![input.edition_id, series_id, edition_name, publisher],
        )?;
        if updated_edition != 1 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        Ok(())
    })?;

    get_series_detail(database, series_id)
}

pub fn add_volume(
    database: &Database,
    edition_id: &str,
    input: AddVolumeInput,
) -> Result<VolumeWithCollection, AppError> {
    let display_label = required_text(&input.display_label, "invalid_volume_label")?;
    let sort_key = make_volume_sort_key(&display_label)?;
    let (isbn_10, isbn_13) = match input.isbn.as_deref().map(normalize_isbn) {
        None => (None, None),
        Some(isbn) if valid_isbn10(&isbn) => (Some(isbn), None),
        Some(isbn) if valid_isbn13(&isbn) => (None, Some(isbn)),
        Some(_) => return Err(AppError::new("invalid_isbn", "The ISBN is not valid.")),
    };

    let label_exists = database.with_transaction(|transaction| {
        transaction.query_row(
            "SELECT EXISTS(SELECT 1 FROM volumes WHERE edition_id = ?1 AND display_label = ?2)",
            params![edition_id, display_label],
            |row| row.get::<_, bool>(0),
        )
    })?;
    if label_exists {
        return Err(AppError::new(
            "volume_already_exists",
            "This volume already exists.",
        ));
    }
    if let Some(isbn) = isbn_10.as_ref().or(isbn_13.as_ref()) {
        let isbn_exists = database.with_transaction(|transaction| {
            transaction.query_row(
                "SELECT EXISTS(SELECT 1 FROM volumes WHERE isbn_10 = ?1 OR isbn_13 = ?1)",
                [isbn],
                |row| row.get::<_, bool>(0),
            )
        })?;
        if isbn_exists {
            return Err(AppError::new(
                "isbn_already_exists",
                "This ISBN already exists.",
            ));
        }
    }

    let volume_id = new_uuid();
    let collection_id = new_uuid();
    let collection = enforce_ownership_rule(input.collection);
    database.with_transaction(|transaction| {
        transaction.execute(
            "INSERT INTO volumes (id, edition_id, display_label, sort_key, isbn_10, isbn_13, \
                availability_status, release_date_precision, metadata_source, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'unknown', 'manual', \
                strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
            params![
                volume_id,
                edition_id,
                display_label,
                sort_key,
                isbn_10,
                isbn_13,
                input.availability_status.as_str(),
            ],
        )?;
        insert_collection_item(transaction, &collection_id, &volume_id, &collection)?;
        transaction.execute(
            "UPDATE editions SET known_volume_count = (SELECT COUNT(*) FROM volumes WHERE edition_id = ?1), \
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?1",
            [edition_id],
        )?;
        find_volume_by_id_in_transaction(transaction, &volume_id)
    })
}

pub fn set_series_cover_asset(
    database: &Database,
    series_id: &str,
    asset_id: &str,
    relative_path: &str,
    mime_type: CoverMimeType,
    byte_size: i64,
    sha256: &str,
) -> Result<CoverAsset, AppError> {
    database.with_transaction(|transaction| {
        let existing_id = transaction
            .query_row(
                "SELECT id FROM cover_assets WHERE sha256 = ?1",
                [sha256],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        let bound_id = existing_id.as_deref().unwrap_or(asset_id);
        if existing_id.is_none() {
            transaction.execute(
                "INSERT INTO cover_assets (id, source_type, relative_path, mime_type, byte_size, sha256, created_at) \
                 VALUES (?1, 'user_file', ?2, ?3, ?4, ?5, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
                params![asset_id, relative_path, mime_type.content_type(), byte_size, sha256],
            )?;
        }
        if transaction.execute(
            "UPDATE series SET representative_cover_asset_id = ?2, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?1",
            params![series_id, bound_id],
        )? != 1 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        transaction.query_row(
            "SELECT id, relative_path, mime_type, byte_size, sha256 FROM cover_assets WHERE id = ?1",
            [bound_id],
            |row| {
                Ok(CoverAsset {
                    id: row.get(0)?,
                    relative_path: row.get(1)?,
                    mime_type: CoverMimeType::from_db(&row.get::<_, String>(2)?)?,
                    byte_size: row.get(3)?,
                    sha256: row.get(4)?,
                })
            },
        )
    })
}

pub fn clear_series_cover(
    database: &Database,
    series_id: &str,
) -> Result<Option<CoverAsset>, AppError> {
    database.with_transaction(|transaction| {
        let asset = transaction
            .query_row(
                "SELECT ca.id, ca.relative_path, ca.mime_type, ca.byte_size, ca.sha256 \
                 FROM series s JOIN cover_assets ca ON ca.id = s.representative_cover_asset_id WHERE s.id = ?1",
                [series_id],
                |row| {
                    Ok(CoverAsset {
                        id: row.get(0)?,
                        relative_path: row.get(1)?,
                        mime_type: CoverMimeType::from_db(&row.get::<_, String>(2)?)?,
                        byte_size: row.get(3)?,
                        sha256: row.get(4)?,
                    })
                },
            )
            .optional()?;
        if transaction.execute(
            "UPDATE series SET representative_cover_asset_id = NULL, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?1",
            [series_id],
        )? != 1 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        Ok(asset)
    })
}

pub fn delete_cover_asset_if_unreferenced(
    database: &Database,
    asset_id: &str,
) -> Result<bool, AppError> {
    database.with_transaction(|transaction| {
        let references: i64 = transaction.query_row(
            "SELECT (SELECT COUNT(*) FROM series WHERE representative_cover_asset_id = ?1) + \
                    (SELECT COUNT(*) FROM volumes WHERE cover_asset_id = ?1)",
            [asset_id],
            |row| row.get(0),
        )?;
        if references == 0 {
            transaction.execute("DELETE FROM cover_assets WHERE id = ?1", [asset_id])?;
            Ok(true)
        } else {
            Ok(false)
        }
    })
}

pub fn update_collection_item(
    database: &Database,
    volume_id: &str,
    patch: CollectionItemPatch,
) -> Result<VolumeWithCollection, AppError> {
    let collection_id = new_uuid();

    database.with_transaction(|transaction| {
        let current = find_volume_by_id_in_transaction(transaction, volume_id)?;
        let collection = apply_collection_patch(current.collection, patch);

        transaction.execute(
            "INSERT INTO collection_items (\
                id, volume_id, is_owned, is_read, is_wishlisted, purchase_price_amount, \
                purchase_price_currency, acquired_on, condition, storage_location, notes, \
                created_at, updated_at\
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, \
                strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), \
                strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) \
             ON CONFLICT(volume_id) DO UPDATE SET \
                is_owned = excluded.is_owned, \
                is_read = excluded.is_read, \
                is_wishlisted = excluded.is_wishlisted, \
                purchase_price_amount = excluded.purchase_price_amount, \
                purchase_price_currency = excluded.purchase_price_currency, \
                acquired_on = excluded.acquired_on, \
                condition = excluded.condition, \
                storage_location = excluded.storage_location, \
                notes = excluded.notes, \
                updated_at = excluded.updated_at",
            params![
                collection_id,
                volume_id,
                bool_integer(collection.is_owned),
                bool_integer(collection.is_read),
                bool_integer(collection.is_wishlisted),
                collection.purchase_price_amount,
                collection.purchase_price_currency,
                collection.acquired_on,
                collection.condition.as_str(),
                collection.storage_location,
                collection.notes,
            ],
        )?;

        find_volume_by_id_in_transaction(transaction, volume_id)
    })
}

pub fn find_volume_by_isbn(
    database: &Database,
    isbn: &str,
) -> Result<Option<VolumeWithCollection>, AppError> {
    if !is_normalized_isbn(isbn) {
        return Err(AppError::new(
            "invalid_isbn",
            "The ISBN must already be normalized.",
        ));
    }

    database.with_transaction(|transaction| {
        let sql = format!("{VOLUME_SELECT} WHERE v.isbn_10 = ?1 OR v.isbn_13 = ?1 LIMIT 1");
        transaction
            .query_row(&sql, [isbn], volume_from_row)
            .optional()
    })
}

pub fn delete_series(database: &Database, series_id: &str) -> Result<(), AppError> {
    database.with_transaction(|transaction| {
        transaction.execute(
            "DELETE FROM metadata_provenance \
             WHERE entity_type = ?1 AND entity_id IN (\
                SELECT v.id \
                FROM volumes v \
                JOIN editions e ON e.id = v.edition_id \
                WHERE e.series_id = ?2\
             )",
            params!["volume", series_id],
        )?;
        transaction.execute("DELETE FROM series WHERE id = ?1", [series_id])?;
        Ok(())
    })
}

fn get_dashboard_in_transaction(
    transaction: &Transaction<'_>,
) -> rusqlite::Result<DashboardSummary> {
    let (series_count, owned_volume_count, read_volume_count, missing_volume_count) = transaction
        .query_row(
        "SELECT \
                (SELECT COUNT(*) FROM series), \
                (SELECT COUNT(*) FROM collection_items WHERE is_owned = 1), \
                (SELECT COUNT(*) FROM collection_items WHERE is_read = 1), \
                (SELECT COUNT(*) \
                 FROM volumes v \
                 LEFT JOIN collection_items ci ON ci.volume_id = v.id \
                 WHERE v.availability_status = 'released' AND COALESCE(ci.is_owned, 0) = 0)",
        [],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    )?;

    let recent_sql = format!(
        "{VOLUME_SELECT} \
         WHERE ci.is_owned = 1 \
         ORDER BY COALESCE(ci.acquired_on, ci.updated_at) DESC, v.sort_key DESC \
         LIMIT 8"
    );
    let mut recent_statement = transaction.prepare(&recent_sql)?;
    let recent_volumes = recent_statement
        .query_map([], volume_from_row)?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let incomplete_series = list_series_in_transaction(
        transaction,
        &SeriesFilter {
            query: String::new(),
            publication_status: PublicationStatusFilter::All,
            collection: CompletionFilter::Incomplete,
            reading: CompletionFilter::All,
        },
    )?
    .into_iter()
    .filter(|series| series.owned_volume_count > 0)
    .collect();

    Ok(DashboardSummary {
        series_count,
        owned_volume_count,
        read_volume_count,
        missing_volume_count,
        recent_volumes,
        incomplete_series,
    })
}

fn list_series_in_transaction(
    transaction: &Transaction<'_>,
    filter: &SeriesFilter,
) -> rusqlite::Result<Vec<SeriesSummary>> {
    let query = escape_like_pattern(filter.query.trim());
    let mut statement = transaction.prepare(
        "SELECT \
            s.id, s.title, s.original_title, s.publication_status, \
            ca.id, ca.relative_path, ca.mime_type, ca.byte_size, ca.sha256, \
            COUNT(v.id), \
            COALESCE(SUM(CASE WHEN ci.is_owned = 1 THEN 1 ELSE 0 END), 0), \
            COALESCE(SUM(CASE WHEN ci.is_read = 1 THEN 1 ELSE 0 END), 0), \
            COALESCE(SUM(CASE \
                WHEN v.availability_status = 'released' AND COALESCE(ci.is_owned, 0) = 0 \
                THEN 1 ELSE 0 END), 0) \
         FROM series s \
         LEFT JOIN cover_assets ca ON ca.id = s.representative_cover_asset_id \
         LEFT JOIN editions e ON e.series_id = s.id \
         LEFT JOIN volumes v ON v.edition_id = e.id \
         LEFT JOIN collection_items ci ON ci.volume_id = v.id \
         WHERE (\
            ?1 = '' \
            OR s.title LIKE '%' || ?1 || '%' ESCAPE '\\' \
            OR COALESCE(s.original_title, '') LIKE '%' || ?1 || '%' ESCAPE '\\' \
            OR EXISTS (\
                SELECT 1 FROM editions search_edition \
                WHERE search_edition.series_id = s.id \
                  AND search_edition.publisher LIKE '%' || ?1 || '%' ESCAPE '\\'\
            ) \
            OR EXISTS (\
                SELECT 1 \
                FROM series_contributors search_series_contributor \
                JOIN contributors search_contributor \
                  ON search_contributor.id = search_series_contributor.contributor_id \
                WHERE search_series_contributor.series_id = s.id \
                  AND search_contributor.display_name LIKE '%' || ?1 || '%' ESCAPE '\\'\
            )\
         ) \
         AND (?2 = 'all' OR s.publication_status = ?2) \
         GROUP BY s.id, s.title, s.original_title, s.publication_status, \
            ca.id, ca.relative_path, ca.mime_type, ca.byte_size, ca.sha256 \
         HAVING (\
            ?3 = 'all' \
            OR (?3 = 'complete' AND COALESCE(SUM(CASE \
                WHEN v.availability_status = 'released' AND COALESCE(ci.is_owned, 0) = 0 \
                THEN 1 ELSE 0 END), 0) = 0) \
            OR (?3 = 'incomplete' AND COALESCE(SUM(CASE \
                WHEN v.availability_status = 'released' AND COALESCE(ci.is_owned, 0) = 0 \
                THEN 1 ELSE 0 END), 0) > 0)\
         ) \
         AND (\
            ?4 = 'all' \
            OR (?4 = 'complete' AND COALESCE(SUM(CASE \
                WHEN v.availability_status = 'released' AND COALESCE(ci.is_read, 0) = 0 \
                THEN 1 ELSE 0 END), 0) = 0) \
            OR (?4 = 'incomplete' AND COALESCE(SUM(CASE \
                WHEN v.availability_status = 'released' AND COALESCE(ci.is_read, 0) = 0 \
                THEN 1 ELSE 0 END), 0) > 0)\
         ) \
         ORDER BY s.title COLLATE NOCASE, s.id",
    )?;

    let mut summaries = statement
        .query_map(
            params![
                query,
                filter.publication_status.as_str(),
                filter.collection.as_str(),
                filter.reading.as_str(),
            ],
            series_summary_from_row,
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let mut contributors = load_all_contributors(transaction)?;
    let mut publishers = load_all_publishers(transaction)?;
    for summary in &mut summaries {
        summary.contributors = contributors.remove(&summary.id).unwrap_or_default();
        summary.publishers = publishers.remove(&summary.id).unwrap_or_default();
    }

    Ok(summaries)
}

fn get_series_detail_in_transaction(
    transaction: &Transaction<'_>,
    series_id: &str,
) -> rusqlite::Result<SeriesDetail> {
    let mut summary = transaction.query_row(
        "SELECT \
            s.id, s.title, s.original_title, s.publication_status, \
            ca.id, ca.relative_path, ca.mime_type, ca.byte_size, ca.sha256, \
            COUNT(v.id), \
            COALESCE(SUM(CASE WHEN ci.is_owned = 1 THEN 1 ELSE 0 END), 0), \
            COALESCE(SUM(CASE WHEN ci.is_read = 1 THEN 1 ELSE 0 END), 0), \
            COALESCE(SUM(CASE \
                WHEN v.availability_status = 'released' AND COALESCE(ci.is_owned, 0) = 0 \
                THEN 1 ELSE 0 END), 0) \
         FROM series s \
         LEFT JOIN cover_assets ca ON ca.id = s.representative_cover_asset_id \
         LEFT JOIN editions e ON e.series_id = s.id \
         LEFT JOIN volumes v ON v.edition_id = e.id \
         LEFT JOIN collection_items ci ON ci.volume_id = v.id \
         WHERE s.id = ?1 \
         GROUP BY s.id, s.title, s.original_title, s.publication_status, \
            ca.id, ca.relative_path, ca.mime_type, ca.byte_size, ca.sha256",
        [series_id],
        series_summary_from_row,
    )?;
    summary.contributors = load_contributors(transaction, series_id)?;
    summary.publishers = load_publishers(transaction, series_id)?;

    let description = transaction.query_row(
        "SELECT description FROM series WHERE id = ?1",
        [series_id],
        |row| row.get(0),
    )?;

    let mut edition_statement = transaction.prepare(
        "SELECT id, name, language_code, region_code, publisher, format, \
            release_status, known_volume_count \
         FROM editions \
         WHERE series_id = ?1 \
         ORDER BY created_at, id",
    )?;
    let mut editions = edition_statement
        .query_map([series_id], edition_from_row)?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let volume_sql = format!(
        "{VOLUME_SELECT} \
         WHERE v.edition_id IN (SELECT id FROM editions WHERE series_id = ?1) \
         ORDER BY v.edition_id, v.sort_key, v.display_label"
    );
    let mut volume_statement = transaction.prepare(&volume_sql)?;
    let volumes = volume_statement
        .query_map([series_id], volume_from_row)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let mut volumes_by_edition = BTreeMap::<String, Vec<VolumeWithCollection>>::new();
    for volume in volumes {
        volumes_by_edition
            .entry(volume.edition_id.clone())
            .or_default()
            .push(volume);
    }
    for edition in &mut editions {
        edition.volumes = volumes_by_edition.remove(&edition.id).unwrap_or_default();
    }

    Ok(SeriesDetail {
        id: summary.id,
        title: summary.title,
        original_title: summary.original_title,
        publication_status: summary.publication_status,
        contributors: summary.contributors,
        publishers: summary.publishers,
        representative_cover: summary.representative_cover,
        known_volume_count: summary.known_volume_count,
        owned_volume_count: summary.owned_volume_count,
        read_volume_count: summary.read_volume_count,
        missing_volume_count: summary.missing_volume_count,
        description,
        editions,
    })
}

fn series_summary_from_row(row: &Row<'_>) -> rusqlite::Result<SeriesSummary> {
    Ok(SeriesSummary {
        id: row.get(0)?,
        title: row.get(1)?,
        original_title: row.get(2)?,
        publication_status: PublicationStatus::from_db(&row.get::<_, String>(3)?)?,
        contributors: Vec::new(),
        publishers: Vec::new(),
        representative_cover: cover_from_row(row, 4)?,
        known_volume_count: row.get(9)?,
        owned_volume_count: row.get(10)?,
        read_volume_count: row.get(11)?,
        missing_volume_count: row.get(12)?,
    })
}

fn edition_from_row(row: &Row<'_>) -> rusqlite::Result<EditionDetail> {
    Ok(EditionDetail {
        id: row.get(0)?,
        name: row.get(1)?,
        language_code: row.get(2)?,
        region_code: row.get(3)?,
        publisher: row.get(4)?,
        format: EditionFormat::from_db(&row.get::<_, String>(5)?)?,
        release_status: PublicationStatus::from_db(&row.get::<_, String>(6)?)?,
        known_volume_count: row.get(7)?,
        volumes: Vec::new(),
    })
}

fn volume_from_row(row: &Row<'_>) -> rusqlite::Result<VolumeWithCollection> {
    Ok(VolumeWithCollection {
        id: row.get(0)?,
        edition_id: row.get(1)?,
        display_label: row.get(2)?,
        sort_key: row.get(3)?,
        title_override: row.get(4)?,
        isbn_10: row.get(5)?,
        isbn_13: row.get(6)?,
        translator: row.get(7)?,
        availability_status: AvailabilityStatus::from_db(&row.get::<_, String>(8)?)?,
        release_date: row.get(9)?,
        release_date_precision: DatePrecision::from_db(&row.get::<_, String>(10)?)?,
        list_price_amount: row.get(11)?,
        list_price_currency: row.get(12)?,
        cover: cover_from_row(row, 13)?,
        collection: CollectionItemView {
            is_owned: row.get::<_, i64>(18)? != 0,
            is_read: row.get::<_, i64>(19)? != 0,
            is_wishlisted: row.get::<_, i64>(20)? != 0,
            purchase_price_amount: row.get(21)?,
            purchase_price_currency: row.get(22)?,
            acquired_on: row.get(23)?,
            condition: BookCondition::from_db(&row.get::<_, String>(24)?)?,
            storage_location: row.get(25)?,
            notes: row.get(26)?,
        },
    })
}

fn cover_from_row(row: &Row<'_>, start: usize) -> rusqlite::Result<Option<CoverAsset>> {
    let Some(id) = row.get::<_, Option<String>>(start)? else {
        return Ok(None);
    };

    let relative_path = row
        .get::<_, Option<String>>(start + 1)?
        .ok_or(rusqlite::Error::InvalidQuery)?;
    let mime_type = row
        .get::<_, Option<String>>(start + 2)?
        .ok_or(rusqlite::Error::InvalidQuery)?;
    let byte_size = row
        .get::<_, Option<i64>>(start + 3)?
        .ok_or(rusqlite::Error::InvalidQuery)?;
    let sha256 = row
        .get::<_, Option<String>>(start + 4)?
        .ok_or(rusqlite::Error::InvalidQuery)?;

    Ok(Some(CoverAsset {
        id,
        relative_path,
        mime_type: CoverMimeType::from_db(&mime_type)?,
        byte_size,
        sha256,
    }))
}

fn find_volume_by_id_in_transaction(
    transaction: &Transaction<'_>,
    volume_id: &str,
) -> rusqlite::Result<VolumeWithCollection> {
    let sql = format!("{VOLUME_SELECT} WHERE v.id = ?1");
    transaction.query_row(&sql, [volume_id], volume_from_row)
}

fn insert_collection_item(
    transaction: &Transaction<'_>,
    collection_id: &str,
    volume_id: &str,
    collection: &CollectionItemView,
) -> rusqlite::Result<()> {
    transaction.execute(
        "INSERT INTO collection_items (\
            id, volume_id, is_owned, is_read, is_wishlisted, purchase_price_amount, \
            purchase_price_currency, acquired_on, condition, storage_location, notes, \
            created_at, updated_at\
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, \
            strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), \
            strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
        params![
            collection_id,
            volume_id,
            bool_integer(collection.is_owned),
            bool_integer(collection.is_read),
            bool_integer(collection.is_wishlisted),
            collection.purchase_price_amount,
            collection.purchase_price_currency,
            collection.acquired_on,
            collection.condition.as_str(),
            collection.storage_location,
            collection.notes,
        ],
    )?;
    Ok(())
}

fn load_all_contributors(
    transaction: &Transaction<'_>,
) -> rusqlite::Result<BTreeMap<String, Vec<ContributorView>>> {
    let mut statement = transaction.prepare(
        "SELECT sc.series_id, c.display_name, sc.role \
         FROM series_contributors sc \
         JOIN contributors c ON c.id = sc.contributor_id \
         ORDER BY sc.series_id, sc.sort_order, c.display_name, sc.role",
    )?;
    let rows = statement.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            ContributorView {
                name: row.get(1)?,
                role: ContributorRole::from_db(&row.get::<_, String>(2)?)?,
            },
        ))
    })?;
    let mut contributors = BTreeMap::<String, Vec<ContributorView>>::new();
    for row in rows {
        let (series_id, contributor) = row?;
        contributors.entry(series_id).or_default().push(contributor);
    }
    Ok(contributors)
}

fn load_all_publishers(
    transaction: &Transaction<'_>,
) -> rusqlite::Result<BTreeMap<String, Vec<String>>> {
    let mut statement = transaction.prepare(
        "SELECT series_id, publisher \
         FROM editions \
         GROUP BY series_id, publisher \
         ORDER BY series_id, publisher",
    )?;
    let rows = statement.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    let mut publishers = BTreeMap::<String, Vec<String>>::new();
    for row in rows {
        let (series_id, publisher) = row?;
        publishers.entry(series_id).or_default().push(publisher);
    }
    Ok(publishers)
}

fn load_contributors(
    transaction: &Transaction<'_>,
    series_id: &str,
) -> rusqlite::Result<Vec<ContributorView>> {
    let mut statement = transaction.prepare(
        "SELECT c.display_name, sc.role \
         FROM series_contributors sc \
         JOIN contributors c ON c.id = sc.contributor_id \
         WHERE sc.series_id = ?1 \
         ORDER BY sc.sort_order, c.display_name, sc.role",
    )?;
    let rows = statement.query_map([series_id], |row| {
        Ok(ContributorView {
            name: row.get(0)?,
            role: ContributorRole::from_db(&row.get::<_, String>(1)?)?,
        })
    })?;
    rows.collect()
}

fn load_publishers(
    transaction: &Transaction<'_>,
    series_id: &str,
) -> rusqlite::Result<Vec<String>> {
    let mut statement = transaction.prepare(
        "SELECT publisher \
         FROM editions \
         WHERE series_id = ?1 \
         GROUP BY publisher \
         ORDER BY publisher",
    )?;
    let rows = statement.query_map([series_id], |row| row.get(0))?;
    rows.collect()
}

fn apply_collection_patch(
    mut collection: CollectionItemView,
    patch: CollectionItemPatch,
) -> CollectionItemView {
    if let Some(value) = patch.is_owned {
        collection.is_owned = value;
    }
    if let Some(value) = patch.is_read {
        collection.is_read = value;
    }
    if let Some(value) = patch.is_wishlisted {
        collection.is_wishlisted = value;
    }
    if let Some(value) = patch.purchase_price_amount {
        collection.purchase_price_amount = value;
    }
    if let Some(value) = patch.purchase_price_currency {
        collection.purchase_price_currency = value;
    }
    if let Some(value) = patch.acquired_on {
        collection.acquired_on = value;
    }
    if let Some(value) = patch.condition {
        collection.condition = value;
    }
    if let Some(value) = patch.storage_location {
        collection.storage_location = value;
    }
    if let Some(value) = patch.notes {
        collection.notes = value;
    }

    enforce_ownership_rule(collection)
}

fn enforce_ownership_rule(mut collection: CollectionItemView) -> CollectionItemView {
    if collection.is_owned {
        collection.is_wishlisted = false;
    }
    collection
}

fn is_normalized_isbn(isbn: &str) -> bool {
    let bytes = isbn.as_bytes();
    match bytes.len() {
        10 => {
            bytes[..9].iter().all(u8::is_ascii_digit)
                && (bytes[9].is_ascii_digit() || bytes[9] == b'X')
        }
        13 => bytes.iter().all(u8::is_ascii_digit),
        _ => false,
    }
}

fn required_text(value: &str, code: &str) -> Result<String, AppError> {
    let value = value.trim();
    if value.is_empty() {
        Err(AppError::new(code, "A required value is missing."))
    } else {
        Ok(value.to_string())
    }
}

fn optional_text(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let value = value.trim().to_string();
        (!value.is_empty()).then_some(value)
    })
}

fn make_volume_sort_key(label: &str) -> Result<String, AppError> {
    let parts = label.split('.').collect::<Vec<_>>();
    let numeric = parts.len() <= 2
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
        && parts.get(1).is_none_or(|decimal| decimal.len() <= 3);
    if numeric {
        let integer = parts[0].trim_start_matches('0');
        let integer = if integer.is_empty() { "0" } else { integer };
        if integer.len() > 6 {
            return Err(AppError::new(
                "invalid_volume_label",
                "The volume label is invalid.",
            ));
        }
        let decimal = parts.get(1).copied().unwrap_or("");
        return Ok(format!("0:{integer:0>6}.{decimal:0<3}"));
    }
    Ok(match label {
        "上" | "下" => format!("1:{label}"),
        "全一冊" => "2:全一冊".to_string(),
        _ => format!("9:{label}"),
    })
}

fn normalize_isbn(value: &str) -> String {
    value
        .chars()
        .filter(|character| *character != ' ' && *character != '-')
        .flat_map(char::to_uppercase)
        .collect()
}

fn valid_isbn10(isbn: &str) -> bool {
    if !is_normalized_isbn(isbn) || isbn.len() != 10 {
        return false;
    }
    isbn.bytes()
        .enumerate()
        .map(|(index, byte)| {
            let value = if byte == b'X' {
                10
            } else {
                u32::from(byte - b'0')
            };
            value * (10 - index as u32)
        })
        .sum::<u32>()
        % 11
        == 0
}

fn valid_isbn13(isbn: &str) -> bool {
    is_normalized_isbn(isbn)
        && isbn.len() == 13
        && isbn
            .bytes()
            .enumerate()
            .map(|(index, byte)| u32::from(byte - b'0') * if index % 2 == 0 { 1 } else { 3 })
            .sum::<u32>()
            % 10
            == 0
}

fn escape_like_pattern(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

fn bool_integer(value: bool) -> i64 {
    if value {
        1
    } else {
        0
    }
}

fn new_uuid() -> String {
    Uuid::new_v4().to_string()
}
