use manga_shelf_lib::db::Database;
use rusqlite::{params, Transaction};

#[test]
fn migration_creates_schema_and_enforces_foreign_keys() {
    let db = Database::in_memory().expect("database");
    let names = db.table_names().expect("table names");
    assert_eq!(
        names,
        vec![
            "collection_items",
            "contributors",
            "cover_assets",
            "editions",
            "metadata_cache",
            "metadata_provenance",
            "schema_migrations",
            "series",
            "series_contributors",
            "volumes",
        ]
    );
    assert_eq!(db.foreign_keys_enabled().unwrap(), true);
}

#[test]
fn duplicate_non_null_isbn_13_is_rejected() {
    let db = Database::in_memory().expect("database");
    db.with_transaction(|transaction| {
        seed_edition(transaction, "series-1", "edition-1", "tankobon")
    })
    .expect("seed edition");

    let result = db.with_transaction(|transaction| {
        insert_volume(
            transaction,
            "volume-1",
            "edition-1",
            "1",
            "001",
            Some("9789572690017"),
        )?;
        insert_volume(
            transaction,
            "volume-2",
            "edition-1",
            "2",
            "002",
            Some("9789572690017"),
        )?;
        Ok(())
    });

    assert!(result.is_err());
}

#[test]
fn different_editions_can_each_contain_volume_label_one() {
    let db = Database::in_memory().expect("database");

    db.with_transaction(|transaction| {
        insert_series(transaction, "series-1")?;
        insert_edition(transaction, "edition-normal", "series-1", "tankobon")?;
        insert_edition(transaction, "edition-deluxe", "series-1", "deluxe")?;
        insert_volume(transaction, "normal-1", "edition-normal", "1", "001", None)?;
        insert_volume(transaction, "deluxe-1", "edition-deluxe", "1", "001", None)?;
        Ok(())
    })
    .expect("same label in separate editions");

    assert_eq!(row_count(&db, "volumes"), 2);
}

#[test]
fn failed_multi_row_transaction_rolls_back_all_rows() {
    let db = Database::in_memory().expect("database");
    db.with_transaction(|transaction| {
        seed_edition(transaction, "series-1", "edition-1", "tankobon")
    })
    .expect("seed edition");

    let result = db.with_transaction(|transaction| {
        insert_volume(transaction, "volume-1", "edition-1", "1", "001", None)?;
        insert_volume(transaction, "volume-2", "missing-edition", "2", "002", None)?;
        Ok(())
    });

    assert!(result.is_err());
    assert_eq!(row_count(&db, "volumes"), 0);
}

#[test]
fn reopening_file_database_preserves_rows() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join("library.sqlite3");

    {
        let db = Database::open(&path).expect("open file database");
        db.with_transaction(|transaction| insert_series(transaction, "series-1"))
            .expect("insert series");
    }

    let reopened = Database::open(&path).expect("reopen file database");
    assert_eq!(row_count(&reopened, "series"), 1);
    assert_eq!(row_count(&reopened, "schema_migrations"), 1);
}

#[test]
fn deleting_series_cascades_through_collection_item() {
    let db = Database::in_memory().expect("database");
    db.with_transaction(|transaction| {
        seed_edition(transaction, "series-1", "edition-1", "tankobon")?;
        insert_volume(transaction, "volume-1", "edition-1", "1", "001", None)?;
        transaction.execute(
            "INSERT INTO collection_items (id, volume_id, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?3)",
            params!["collection-1", "volume-1", "2026-08-13T00:00:00Z"],
        )?;
        transaction.execute("DELETE FROM series WHERE id = ?1", ["series-1"])?;
        Ok(())
    })
    .expect("cascade delete");

    assert_eq!(row_count(&db, "editions"), 0);
    assert_eq!(row_count(&db, "volumes"), 0);
    assert_eq!(row_count(&db, "collection_items"), 0);
}

#[test]
fn serialized_database_error_does_not_reveal_sql() {
    let db = Database::in_memory().expect("database");
    let error = db
        .with_transaction(|transaction| {
            transaction.execute("SELECT secret_value FROM private_missing_table", [])?;
            Ok(())
        })
        .expect_err("invalid SQL must fail");

    assert_eq!(
        serde_json::to_value(error).expect("serialize error"),
        serde_json::json!({
            "code": "database_operation_failed",
            "message": "The database operation failed."
        })
    );
}

fn seed_edition(
    transaction: &Transaction<'_>,
    series_id: &str,
    edition_id: &str,
    format: &str,
) -> rusqlite::Result<()> {
    insert_series(transaction, series_id)?;
    insert_edition(transaction, edition_id, series_id, format)
}

fn insert_series(transaction: &Transaction<'_>, id: &str) -> rusqlite::Result<()> {
    transaction.execute(
        "INSERT INTO series (id, title, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)",
        params![id, "Test Series", "2026-08-13T00:00:00Z"],
    )?;
    Ok(())
}

fn insert_edition(
    transaction: &Transaction<'_>,
    id: &str,
    series_id: &str,
    format: &str,
) -> rusqlite::Result<()> {
    transaction.execute(
        "INSERT INTO editions (\
            id, series_id, name, language_code, region_code, publisher, format, created_at, updated_at\
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)",
        params![
            id,
            series_id,
            "Test Edition",
            "zh-Hant",
            "TW",
            "Test Publisher",
            format,
            "2026-08-13T00:00:00Z"
        ],
    )?;
    Ok(())
}

fn insert_volume(
    transaction: &Transaction<'_>,
    id: &str,
    edition_id: &str,
    display_label: &str,
    sort_key: &str,
    isbn_13: Option<&str>,
) -> rusqlite::Result<()> {
    transaction.execute(
        "INSERT INTO volumes (\
            id, edition_id, display_label, sort_key, isbn_13, created_at, updated_at\
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
        params![
            id,
            edition_id,
            display_label,
            sort_key,
            isbn_13,
            "2026-08-13T00:00:00Z"
        ],
    )?;
    Ok(())
}

fn row_count(database: &Database, table: &str) -> i64 {
    database
        .with_transaction(|transaction| {
            transaction.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
        })
        .expect("row count")
}
