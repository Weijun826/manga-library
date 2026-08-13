use std::{fs, path::Path};

use manga_shelf_lib::{commands::backup::create_update_backup, db::Database};
use rusqlite::Connection;
use tempfile::tempdir;

fn seed_series(database: &Database, title: &str) {
    database
        .with_transaction(|transaction| {
            transaction.execute(
                "INSERT INTO series (
                    id, title, original_title, description, publication_status,
                    representative_cover_asset_id, created_at, updated_at
                ) VALUES ('series-1', ?1, NULL, NULL, 'ongoing', NULL, '2026-08-13', '2026-08-13')",
                [title],
            )?;
            Ok(())
        })
        .expect("seed source database");
}

fn backup_names(directory: &Path) -> Vec<String> {
    let mut names = fs::read_dir(directory)
        .expect("read backup directory")
        .map(|entry| {
            entry
                .expect("read backup entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect::<Vec<_>>();
    names.sort();
    names
}

#[test]
fn update_backup_preserves_committed_library_data() {
    let directory = tempdir().expect("create temp directory");
    let database =
        Database::open(directory.path().join("library.sqlite3")).expect("open source database");
    seed_series(&database, "保留下來的漫畫");

    let backup_path = create_update_backup(
        &database,
        &directory.path().join("backups"),
        "0.2.0",
        1_786_628_000,
    )
    .expect("create update backup");

    let backup = Connection::open(backup_path).expect("open completed backup");
    let title: String = backup
        .query_row(
            "SELECT title FROM series WHERE id = 'series-1'",
            [],
            |row| row.get(0),
        )
        .expect("read backed-up series");
    let integrity: String = backup
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .expect("check backup integrity");

    assert_eq!(title, "保留下來的漫畫");
    assert_eq!(integrity, "ok");
}

#[test]
fn failed_backup_leaves_the_source_database_readable() {
    let directory = tempdir().expect("create temp directory");
    let database =
        Database::open(directory.path().join("library.sqlite3")).expect("open source database");
    seed_series(&database, "來源仍可讀");
    let destination_file = directory.path().join("not-a-directory");
    fs::write(&destination_file, b"occupied").expect("create blocking file");

    let error = create_update_backup(&database, &destination_file, "0.2.0", 1)
        .expect_err("backup must fail for a file destination");

    assert_eq!(error.code, "backup_failed");
    let source_title = database
        .with_transaction(|transaction| {
            transaction.query_row(
                "SELECT title FROM series WHERE id = 'series-1'",
                [],
                |row| row.get::<_, String>(0),
            )
        })
        .expect("source remains readable");
    assert_eq!(source_title, "來源仍可讀");
}

#[test]
fn successful_backup_rotation_keeps_only_the_newest_three_files() {
    let directory = tempdir().expect("create temp directory");
    let database =
        Database::open(directory.path().join("library.sqlite3")).expect("open source database");
    seed_series(&database, "輪替測試");
    let backups = directory.path().join("backups");

    for timestamp in 1..=4 {
        create_update_backup(&database, &backups, "0.2.0", timestamp)
            .expect("create rotating backup");
    }

    let names = backup_names(&backups);
    assert_eq!(names.len(), 3);
    assert!(names.iter().all(|name| !name.contains("-1-")));
    assert!(names.iter().any(|name| name.contains("-2-")));
    assert!(names.iter().any(|name| name.contains("-3-")));
    assert!(names.iter().any(|name| name.contains("-4-")));
}

#[test]
fn invalid_version_cannot_escape_the_backup_directory_or_delete_old_backups() {
    let directory = tempdir().expect("create temp directory");
    let database =
        Database::open(directory.path().join("library.sqlite3")).expect("open source database");
    let backups = directory.path().join("backups");
    fs::create_dir(&backups).expect("create backup directory");
    for name in [
        "library-before-0.1.0-1-a.sqlite3",
        "library-before-0.1.0-2-b.sqlite3",
    ] {
        fs::write(backups.join(name), b"existing backup").expect("create old backup");
    }

    let error = create_update_backup(&database, &backups, "../0.2.0", 3)
        .expect_err("unsafe version must be rejected");

    assert_eq!(error.code, "backup_failed");
    assert_eq!(backup_names(&backups).len(), 2);
    assert!(!directory.path().join("0.2.0-3.sqlite3").exists());
}
