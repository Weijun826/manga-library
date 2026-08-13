use std::fs;

use manga_shelf_lib::{
    commands::cover::{remove_series_cover, store_series_cover},
    db::{models::CoverMimeType, repository, Database},
};
use tempfile::tempdir;

mod support;
use support::ten_volume_batch;

fn png_bytes() -> Vec<u8> {
    let mut bytes = b"\x89PNG\r\n\x1a\n".to_vec();
    bytes.extend_from_slice(b"manga-cover-content");
    bytes
}

#[test]
fn local_cover_is_validated_copied_deduplicated_replaced_and_removed() {
    let root = tempdir().expect("temp dir");
    let db = Database::in_memory().expect("database");
    let first = repository::create_series_batch(&db, ten_volume_batch()).expect("first series");
    let second = repository::create_series_batch(&db, ten_volume_batch()).expect("second series");
    let source = root.path().join("chosen.jpg");
    fs::write(&source, png_bytes()).expect("write source");

    let first_asset =
        store_series_cover(&db, root.path(), &first.id, &source).expect("store cover");
    let second_asset =
        store_series_cover(&db, root.path(), &second.id, &source).expect("reuse cover");

    assert_eq!(first_asset.id, second_asset.id);
    assert_eq!(first_asset.mime_type, CoverMimeType::Png);
    assert!(root.path().join(&first_asset.relative_path).is_file());
    fs::remove_file(&source).expect("move source away");
    assert!(root.path().join(&first_asset.relative_path).is_file());

    remove_series_cover(&db, root.path(), &first.id).expect("remove first reference");
    assert!(root.path().join(&first_asset.relative_path).is_file());
    remove_series_cover(&db, root.path(), &second.id).expect("remove final reference");
    assert!(!root.path().join(&first_asset.relative_path).exists());
    assert!(repository::get_series_detail(&db, &first.id)
        .expect("detail")
        .representative_cover
        .is_none());
}

#[test]
fn local_cover_rejects_empty_forged_and_oversized_files_without_database_changes() {
    let root = tempdir().expect("temp dir");
    let db = Database::in_memory().expect("database");
    let series = repository::create_series_batch(&db, ten_volume_batch()).expect("series");

    for (name, bytes) in [
        ("empty.png", vec![]),
        ("forged.png", b"not-an-image".to_vec()),
    ] {
        let source = root.path().join(name);
        fs::write(&source, bytes).expect("write invalid source");
        let error = store_series_cover(&db, root.path(), &series.id, &source)
            .expect_err("invalid cover must fail");
        assert_eq!(error.code, "invalid_cover_image");
    }

    let oversized = root.path().join("oversized.webp");
    let file = fs::File::create(&oversized).expect("create oversized");
    file.set_len(10 * 1024 * 1024 + 1).expect("size oversized");
    let error = store_series_cover(&db, root.path(), &series.id, &oversized)
        .expect_err("oversized cover must fail");
    assert_eq!(error.code, "cover_too_large");

    assert!(repository::get_series_detail(&db, &series.id)
        .expect("detail")
        .representative_cover
        .is_none());
    assert!(!root.path().join("covers").exists());
}
