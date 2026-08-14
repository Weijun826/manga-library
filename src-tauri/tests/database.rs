use std::collections::{BTreeMap, HashSet};

use manga_shelf_lib::db::{
    models::{
        AddVolumeInput, AvailabilityStatus, BookCondition, CollectionItemPatch, CollectionItemView,
        CompletionFilter, ContributorInput, ContributorRole, CreateEditionInput,
        CreateSeriesBatchInput, CreateSeriesInput, CreateVolumeInput, DatePrecision, EditionFormat,
        MetadataSource, PublicationStatus, PublicationStatusFilter, SeriesDetail, SeriesFilter,
        UpdateSeriesMetadataInput, UpdateVolumeDetailsInput,
    },
    repository, Database,
};
use rusqlite::{params, Transaction};
use uuid::{Uuid, Version};

#[test]
fn repository_update_series_metadata_preserves_volume_and_collection_identity() {
    let db = Database::in_memory().expect("database");
    let created = repository::create_series_batch(&db, ten_volume_batch()).expect("create series");
    let original_volume_id = created.editions[0].volumes[0].id.clone();
    let original_collection = created.editions[0].volumes[0].collection.clone();

    let updated = repository::update_series_metadata(
        &db,
        &created.id,
        UpdateSeriesMetadataInput {
            title: "海風冒險譚 新版".to_string(),
            original_title: None,
            description: Some("更新後簡介".to_string()),
            publication_status: PublicationStatus::Ongoing,
            author: "新作者".to_string(),
            edition_id: created.editions[0].id.clone(),
            edition_name: "續刊單行本".to_string(),
            publisher: "新出版社".to_string(),
        },
    )
    .expect("update metadata");

    assert_eq!(updated.title, "海風冒險譚 新版");
    assert_eq!(updated.original_title, None);
    assert_eq!(updated.description.as_deref(), Some("更新後簡介"));
    assert_eq!(updated.contributors[0].name, "新作者");
    assert_eq!(updated.editions[0].name, "續刊單行本");
    assert_eq!(updated.editions[0].publisher, "新出版社");
    assert_eq!(updated.editions[0].volumes[0].id, original_volume_id);
    assert_eq!(
        updated.editions[0].volumes[0].collection,
        original_collection
    );
}

#[test]
fn repository_update_series_metadata_rejects_empty_required_fields_without_partial_changes() {
    let db = Database::in_memory().expect("database");
    let created = repository::create_series_batch(&db, ten_volume_batch()).expect("create series");

    let error = repository::update_series_metadata(
        &db,
        &created.id,
        UpdateSeriesMetadataInput {
            title: "已改但不該保存".to_string(),
            original_title: None,
            description: None,
            publication_status: PublicationStatus::Ongoing,
            author: "   ".to_string(),
            edition_id: created.editions[0].id.clone(),
            edition_name: "單行本".to_string(),
            publisher: "出版社".to_string(),
        },
    )
    .expect_err("empty author must fail");

    assert_eq!(error.code, "invalid_series_metadata");
    let unchanged = repository::get_series_detail(&db, &created.id).expect("read unchanged series");
    assert_eq!(unchanged.title, created.title);
    assert_eq!(unchanged.contributors[0].name, "測試作者");
}

#[test]
fn repository_add_volume_supports_numeric_and_special_labels_and_enforces_boundaries() {
    let db = Database::in_memory().expect("database");
    let created = repository::create_series_batch(&db, ten_volume_batch()).expect("create series");
    let edition_id = &created.editions[0].id;

    let numeric = repository::add_volume(
        &db,
        edition_id,
        AddVolumeInput {
            display_label: "11".to_string(),
            isbn: Some("978-4-08-883316-3".to_string()),
            availability_status: AvailabilityStatus::Released,
            collection: CollectionItemView {
                is_owned: true,
                is_wishlisted: true,
                ..empty_collection()
            },
        },
    )
    .expect("add numeric volume");
    let special = repository::add_volume(
        &db,
        edition_id,
        AddVolumeInput {
            display_label: "外傳".to_string(),
            isbn: None,
            availability_status: AvailabilityStatus::Upcoming,
            collection: empty_collection(),
        },
    )
    .expect("add special volume");

    assert_eq!(numeric.sort_key, "0:000011.000");
    assert_eq!(numeric.isbn_13.as_deref(), Some("9784088833163"));
    assert!(numeric.collection.is_owned);
    assert!(!numeric.collection.is_wishlisted);
    assert_eq!(special.sort_key, "9:外傳");

    let duplicate_label = repository::add_volume(
        &db,
        edition_id,
        AddVolumeInput {
            display_label: "11".to_string(),
            isbn: None,
            availability_status: AvailabilityStatus::Unknown,
            collection: empty_collection(),
        },
    )
    .expect_err("duplicate label must fail");
    assert_eq!(duplicate_label.code, "volume_already_exists");

    let duplicate_isbn = repository::add_volume(
        &db,
        edition_id,
        AddVolumeInput {
            display_label: "12.5".to_string(),
            isbn: Some("9784088833163".to_string()),
            availability_status: AvailabilityStatus::Released,
            collection: empty_collection(),
        },
    )
    .expect_err("duplicate ISBN must fail");
    assert_eq!(duplicate_isbn.code, "isbn_already_exists");

    let detail = repository::get_series_detail(&db, &created.id).expect("read updated series");
    assert_eq!(detail.known_volume_count, 12);
}

#[test]
fn update_volume_details_normalizes_fields_and_saves_collection_atomically() {
    let db = Database::in_memory().expect("database");
    let created = repository::create_series_batch(&db, ten_volume_batch()).expect("create series");
    let volume = &created.editions[0].volumes[2];

    let updated = repository::update_volume_details(
        &db,
        &volume.id,
        UpdateVolumeDetailsInput {
            display_label: " 3.5 ".to_string(),
            isbn: Some("978-4-08-883316-3".to_string()),
            availability_status: AvailabilityStatus::Released,
            collection: CollectionItemView {
                is_owned: true,
                is_read: true,
                is_wishlisted: true,
                purchase_price_amount: Some(180),
                purchase_price_currency: Some("USD".to_string()),
                acquired_on: Some("2024-02-29".to_string()),
                condition: BookCondition::LikeNew,
                storage_location: Some("  書櫃 B  ".to_string()),
                notes: Some("  首刷  ".to_string()),
            },
        },
    )
    .expect("update volume details");

    assert_eq!(updated.display_label, "3.5");
    assert_eq!(updated.sort_key, "0:000003.500");
    assert_eq!(updated.isbn_13.as_deref(), Some("9784088833163"));
    assert!(updated.collection.is_owned);
    assert!(updated.collection.is_read);
    assert!(!updated.collection.is_wishlisted);
    assert_eq!(updated.collection.purchase_price_amount, Some(180));
    assert_eq!(
        updated.collection.purchase_price_currency.as_deref(),
        Some("TWD")
    );
    assert_eq!(
        updated.collection.acquired_on.as_deref(),
        Some("2024-02-29")
    );
    assert_eq!(
        updated.collection.storage_location.as_deref(),
        Some("書櫃 B")
    );
    assert_eq!(updated.collection.notes.as_deref(), Some("首刷"));
}

#[test]
fn update_volume_details_rejects_duplicates_invalid_dates_and_negative_prices() {
    let db = Database::in_memory().expect("database");
    let created = repository::create_series_batch(&db, ten_volume_batch()).expect("create series");
    let volume = &created.editions[0].volumes[2];

    let cases = [
        (
            "duplicate label",
            UpdateVolumeDetailsInput {
                display_label: "2".to_string(),
                isbn: None,
                availability_status: AvailabilityStatus::Released,
                collection: empty_collection(),
            },
            "volume_already_exists",
        ),
        (
            "duplicate ISBN",
            UpdateVolumeDetailsInput {
                display_label: "3".to_string(),
                isbn: Some("9789572690017".to_string()),
                availability_status: AvailabilityStatus::Released,
                collection: empty_collection(),
            },
            "isbn_already_exists",
        ),
        (
            "invalid date",
            UpdateVolumeDetailsInput {
                display_label: "3".to_string(),
                isbn: None,
                availability_status: AvailabilityStatus::Released,
                collection: CollectionItemView {
                    acquired_on: Some("2023-02-29".to_string()),
                    ..empty_collection()
                },
            },
            "invalid_acquired_on",
        ),
        (
            "negative price",
            UpdateVolumeDetailsInput {
                display_label: "3".to_string(),
                isbn: None,
                availability_status: AvailabilityStatus::Released,
                collection: CollectionItemView {
                    purchase_price_amount: Some(-1),
                    ..empty_collection()
                },
            },
            "invalid_purchase_price",
        ),
    ];

    for (name, input, expected_code) in cases {
        let error = repository::update_volume_details(&db, &volume.id, input).expect_err(name);
        assert_eq!(error.code, expected_code, "{name}");
    }
}

#[test]
fn update_volume_details_rolls_back_volume_when_collection_write_fails() {
    let db = Database::in_memory().expect("database");
    let created = repository::create_series_batch(&db, ten_volume_batch()).expect("create series");
    let volume = &created.editions[0].volumes[2];
    db.with_transaction(|transaction| {
        transaction.execute_batch(
            "CREATE TRIGGER reject_collection_update BEFORE UPDATE ON collection_items \
             BEGIN SELECT RAISE(ABORT, 'reject collection update'); END;",
        )
    })
    .expect("install failure trigger");

    let result = repository::update_volume_details(
        &db,
        &volume.id,
        UpdateVolumeDetailsInput {
            display_label: "改壞了".to_string(),
            isbn: None,
            availability_status: AvailabilityStatus::Released,
            collection: CollectionItemView {
                is_owned: true,
                ..empty_collection()
            },
        },
    );
    assert!(result.is_err());

    let unchanged = repository::get_series_detail(&db, &created.id).expect("unchanged detail");
    let unchanged_volume = unchanged.editions[0]
        .volumes
        .iter()
        .find(|item| item.id == volume.id)
        .expect("same volume");
    assert_eq!(unchanged_volume.display_label, "3");
    assert_eq!(unchanged_volume.collection, volume.collection);
}

#[test]
fn repository_list_series_finds_a_series_by_contributor_name() {
    let db = Database::in_memory().expect("database");
    let created =
        repository::create_series_batch(&db, ten_volume_batch()).expect("create series batch");

    let matching = repository::list_series(
        &db,
        SeriesFilter {
            query: "測試作者".to_string(),
            publication_status: PublicationStatusFilter::All,
            collection: CompletionFilter::All,
            reading: CompletionFilter::All,
        },
    )
    .expect("contributor search");

    assert_eq!(matching.len(), 1);
    assert_eq!(matching[0].id, created.id);
}

#[test]
fn repository_create_series_batch_uses_unique_uuid_v4_observable_entity_ids() {
    let db = Database::in_memory().expect("database");
    let first =
        repository::create_series_batch(&db, ten_volume_batch()).expect("first series batch");

    let mut second_input = ten_volume_batch();
    second_input.series.title = "山海冒險譚".to_string();
    second_input.series.original_title = Some("Mountain Adventures".to_string());
    for volume in &mut second_input.volumes {
        volume.isbn_10 = None;
        volume.isbn_13 = None;
    }
    let second = repository::create_series_batch(&db, second_input).expect("second series batch");

    let first_ids = observable_entity_ids(&first);
    let second_ids = observable_entity_ids(&second);
    for id in first_ids.iter().chain(&second_ids) {
        let parsed = Uuid::parse_str(id).expect("observable entity ID must be a UUID");
        assert_eq!(parsed.get_version(), Some(Version::Random));
    }

    let all_ids = first_ids
        .iter()
        .chain(&second_ids)
        .copied()
        .collect::<Vec<_>>();
    let unique_ids = all_ids.iter().copied().collect::<HashSet<_>>();
    assert_eq!(unique_ids.len(), all_ids.len());
}

#[test]
fn repository_dashboard_incomplete_series_excludes_zero_owned_but_list_filter_keeps_it() {
    let db = Database::in_memory().expect("database");

    let mut zero_owned_input = ten_volume_batch();
    zero_owned_input.series.title = "完全未收藏系列".to_string();
    zero_owned_input.series.original_title = None;
    zero_owned_input.edition.publisher = "零收藏出版社".to_string();
    for volume in &mut zero_owned_input.volumes {
        volume.isbn_10 = None;
        volume.isbn_13 = None;
        volume.collection = empty_collection();
    }
    let zero_owned =
        repository::create_series_batch(&db, zero_owned_input).expect("zero-owned series batch");
    let partially_owned =
        repository::create_series_batch(&db, ten_volume_batch()).expect("partial series batch");

    let dashboard = repository::get_dashboard(&db).expect("dashboard");
    assert_eq!(dashboard.incomplete_series.len(), 1);
    assert_eq!(dashboard.incomplete_series[0].id, partially_owned.id);

    let general_incomplete = repository::list_series(
        &db,
        SeriesFilter {
            query: "完全未收藏系列".to_string(),
            publication_status: PublicationStatusFilter::All,
            collection: CompletionFilter::Incomplete,
            reading: CompletionFilter::All,
        },
    )
    .expect("general incomplete collection filter");
    assert_eq!(general_incomplete.len(), 1);
    assert_eq!(general_incomplete[0].id, zero_owned.id);
}

#[test]
fn repository_ten_volume_batch_tracks_missing_volumes_and_ownership_updates() {
    let db = Database::in_memory().expect("database");

    let created =
        repository::create_series_batch(&db, ten_volume_batch()).expect("create series batch");

    assert_eq!(created.title, "海風冒險譚");
    assert_eq!(created.known_volume_count, 10);
    assert_eq!(created.owned_volume_count, 6);
    assert_eq!(created.read_volume_count, 2);
    assert_eq!(created.missing_volume_count, 4);
    assert_eq!(created.contributors.len(), 1);
    assert_eq!(created.contributors[0].name, "測試作者");
    assert_eq!(created.contributors[0].role, ContributorRole::Author);
    assert_eq!(created.publishers, vec!["東立"]);
    assert!(created.representative_cover.is_none());
    assert_eq!(row_count(&db, "metadata_provenance"), 10);

    let dashboard = repository::get_dashboard(&db).expect("dashboard");
    assert_eq!(dashboard.series_count, 1);
    assert_eq!(dashboard.owned_volume_count, 6);
    assert_eq!(dashboard.read_volume_count, 2);
    assert_eq!(dashboard.missing_volume_count, 4);
    assert_eq!(dashboard.incomplete_series.len(), 1);
    assert_eq!(dashboard.incomplete_series[0].id, created.id);
    assert_eq!(dashboard.incomplete_series[0].missing_volume_count, 4);

    let detail = repository::get_series_detail(&db, &created.id).expect("series detail");
    let edition = &detail.editions[0];
    let missing_labels = edition
        .volumes
        .iter()
        .filter(|volume| {
            volume.availability_status == AvailabilityStatus::Released
                && !volume.collection.is_owned
        })
        .map(|volume| volume.display_label.as_str())
        .collect::<Vec<_>>();
    assert_eq!(missing_labels, vec!["7", "8", "9", "10"]);

    let volume_seven_id = edition
        .volumes
        .iter()
        .find(|volume| volume.display_label == "7")
        .expect("volume 7")
        .id
        .clone();
    let updated = repository::update_collection_item(
        &db,
        &volume_seven_id,
        CollectionItemPatch {
            is_owned: Some(true),
            ..CollectionItemPatch::default()
        },
    )
    .expect("own volume 7");

    assert!(updated.collection.is_owned);
    assert!(!updated.collection.is_wishlisted);
    assert_eq!(updated.collection.purchase_price_amount, None);
    assert_eq!(updated.collection.condition, BookCondition::Unknown);

    let updated_dashboard = repository::get_dashboard(&db).expect("updated dashboard");
    assert_eq!(updated_dashboard.owned_volume_count, 7);
    assert_eq!(updated_dashboard.missing_volume_count, 3);

    let updated_detail =
        repository::get_series_detail(&db, &created.id).expect("updated series detail");
    let volume_seven = updated_detail.editions[0]
        .volumes
        .iter()
        .find(|volume| volume.display_label == "7")
        .expect("updated volume 7");
    assert_eq!(volume_seven.collection.is_wishlisted, false);
}

#[test]
fn repository_left_join_returns_exact_collection_defaults_when_row_is_absent() {
    let db = Database::in_memory().expect("database");
    db.with_transaction(|transaction| {
        seed_edition(
            transaction,
            "series-no-collection",
            "edition-no-collection",
            "tankobon",
        )?;
        insert_volume(
            transaction,
            "volume-no-collection",
            "edition-no-collection",
            "1",
            "001",
            None,
        )?;
        transaction.execute(
            "UPDATE volumes SET availability_status = ?1 WHERE id = ?2",
            params!["released", "volume-no-collection"],
        )?;
        Ok(())
    })
    .expect("seed volume without collection row");

    let detail = repository::get_series_detail(&db, "series-no-collection")
        .expect("series detail with collection defaults");
    let volume = &detail.editions[0].volumes[0];

    assert_eq!(detail.known_volume_count, 1);
    assert_eq!(detail.owned_volume_count, 0);
    assert_eq!(detail.read_volume_count, 0);
    assert_eq!(detail.missing_volume_count, 1);
    assert_eq!(volume.collection, empty_collection());
}

#[test]
fn repository_find_volume_by_isbn_requires_normalized_input_and_checks_both_columns() {
    let db = Database::in_memory().expect("database");
    repository::create_series_batch(&db, ten_volume_batch()).expect("create series batch");

    let by_isbn_10 = repository::find_volume_by_isbn(&db, "080442957X")
        .expect("normalized ISBN-10 lookup")
        .expect("ISBN-10 volume");
    assert_eq!(by_isbn_10.display_label, "1");
    assert_eq!(by_isbn_10.isbn_10.as_deref(), Some("080442957X"));

    let by_isbn_13 = repository::find_volume_by_isbn(&db, "9789572690017")
        .expect("normalized ISBN-13 lookup")
        .expect("ISBN-13 volume");
    assert_eq!(by_isbn_13.display_label, "2");
    assert_eq!(by_isbn_13.isbn_13.as_deref(), Some("9789572690017"));

    let unnormalized = repository::find_volume_by_isbn(&db, "978-957-26-9001-7")
        .expect_err("repository must reject unnormalized ISBN input");
    assert_eq!(unnormalized.code, "invalid_isbn");

    assert!(repository::find_volume_by_isbn(&db, "9789572690099")
        .expect("absent normalized ISBN")
        .is_none());
}

#[test]
fn repository_create_series_batch_rolls_back_every_row_when_one_volume_is_invalid() {
    let db = Database::in_memory().expect("database");
    let mut input = ten_volume_batch();
    input.volumes[9].isbn_13 = Some("9789572690017".to_string());

    assert!(repository::create_series_batch(&db, input).is_err());
    assert_eq!(row_count(&db, "series"), 0);
    assert_eq!(row_count(&db, "contributors"), 0);
    assert_eq!(row_count(&db, "editions"), 0);
    assert_eq!(row_count(&db, "volumes"), 0);
    assert_eq!(row_count(&db, "collection_items"), 0);
    assert_eq!(row_count(&db, "metadata_provenance"), 0);
}

#[test]
fn repository_list_series_filters_are_parameterized_and_delete_removes_the_series() {
    let db = Database::in_memory().expect("database");
    let created =
        repository::create_series_batch(&db, ten_volume_batch()).expect("create series batch");

    let matching = repository::list_series(
        &db,
        SeriesFilter {
            query: "海風".to_string(),
            publication_status: PublicationStatusFilter::Completed,
            collection: CompletionFilter::Incomplete,
            reading: CompletionFilter::Incomplete,
        },
    )
    .expect("matching series filter");
    assert_eq!(matching.len(), 1);
    assert_eq!(matching[0].id, created.id);
    assert_eq!(matching[0].missing_volume_count, 4);

    let injection_text = repository::list_series(
        &db,
        SeriesFilter {
            query: "' OR 1=1 --".to_string(),
            publication_status: PublicationStatusFilter::All,
            collection: CompletionFilter::All,
            reading: CompletionFilter::All,
        },
    )
    .expect("parameterized text filter");
    assert!(injection_text.is_empty());

    repository::delete_series(&db, &created.id).expect("delete series");
    assert_eq!(row_count(&db, "series"), 0);
    assert_eq!(row_count(&db, "editions"), 0);
    assert_eq!(row_count(&db, "volumes"), 0);
    assert_eq!(row_count(&db, "collection_items"), 0);
    assert_eq!(row_count(&db, "metadata_provenance"), 0);
}

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

fn ten_volume_batch() -> CreateSeriesBatchInput {
    CreateSeriesBatchInput {
        series: CreateSeriesInput {
            title: "海風冒險譚".to_string(),
            original_title: Some("Sea Breeze Adventures".to_string()),
            description: Some("十冊驗收資料".to_string()),
            publication_status: PublicationStatus::Completed,
            contributors: vec![ContributorInput {
                name: "測試作者".to_string(),
                role: ContributorRole::Author,
                sort_order: 0,
            }],
        },
        edition: CreateEditionInput {
            name: "台灣單行本".to_string(),
            language_code: "zh-Hant".to_string(),
            region_code: "TW".to_string(),
            publisher: "東立".to_string(),
            format: EditionFormat::Tankobon,
            release_status: PublicationStatus::Completed,
            known_volume_count: Some(10),
        },
        volumes: (1..=10)
            .map(|number| CreateVolumeInput {
                display_label: number.to_string(),
                sort_key: format!("0:{number:06}.000"),
                title_override: None,
                isbn_10: (number == 1).then(|| "080442957X".to_string()),
                isbn_13: (number == 2).then(|| "9789572690017".to_string()),
                translator: Some("測試譯者".to_string()),
                availability_status: AvailabilityStatus::Released,
                release_date: Some(format!("2026-{number:02}-01")),
                release_date_precision: DatePrecision::Day,
                list_price_amount: Some(120),
                list_price_currency: Some("TWD".to_string()),
                collection: if number <= 6 {
                    owned_collection(number <= 2, number)
                } else {
                    CollectionItemView {
                        is_wishlisted: number == 7,
                        ..empty_collection()
                    }
                },
                provenance: BTreeMap::from([("displayLabel".to_string(), MetadataSource::Manual)]),
            })
            .collect(),
    }
}

fn observable_entity_ids(detail: &SeriesDetail) -> Vec<&str> {
    let mut ids = vec![detail.id.as_str()];
    for edition in &detail.editions {
        ids.push(edition.id.as_str());
        ids.extend(edition.volumes.iter().map(|volume| volume.id.as_str()));
    }
    ids
}

fn owned_collection(is_read: bool, volume_number: i32) -> CollectionItemView {
    CollectionItemView {
        is_owned: true,
        is_read,
        is_wishlisted: false,
        purchase_price_amount: Some(100 + i64::from(volume_number)),
        purchase_price_currency: Some("TWD".to_string()),
        acquired_on: Some(format!("2026-07-{volume_number:02}")),
        condition: BookCondition::Good,
        storage_location: Some("書櫃 A".to_string()),
        notes: Some(format!("第 {volume_number} 冊")),
    }
}

fn empty_collection() -> CollectionItemView {
    CollectionItemView {
        is_owned: false,
        is_read: false,
        is_wishlisted: false,
        purchase_price_amount: None,
        purchase_price_currency: None,
        acquired_on: None,
        condition: BookCondition::Unknown,
        storage_location: None,
        notes: None,
    }
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
