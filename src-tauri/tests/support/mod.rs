use std::collections::BTreeMap;

use manga_shelf_lib::db::models::{
    AvailabilityStatus, BookCondition, CollectionItemView, ContributorInput, ContributorRole,
    CreateEditionInput, CreateSeriesBatchInput, CreateSeriesInput, CreateVolumeInput,
    DatePrecision, EditionFormat, MetadataSource, PublicationStatus,
};

pub fn ten_volume_batch() -> CreateSeriesBatchInput {
    CreateSeriesBatchInput {
        series: CreateSeriesInput {
            title: "封面測試漫畫".to_string(),
            original_title: None,
            description: None,
            publication_status: PublicationStatus::Ongoing,
            contributors: vec![ContributorInput {
                name: "測試作者".to_string(),
                role: ContributorRole::Author,
                sort_order: 0,
            }],
        },
        edition: CreateEditionInput {
            name: "單行本".to_string(),
            language_code: "zh-Hant".to_string(),
            region_code: "TW".to_string(),
            publisher: "測試出版社".to_string(),
            format: EditionFormat::Tankobon,
            release_status: PublicationStatus::Ongoing,
            known_volume_count: Some(1),
        },
        volumes: vec![CreateVolumeInput {
            display_label: "1".to_string(),
            sort_key: "0:000001.000".to_string(),
            title_override: None,
            isbn_10: None,
            isbn_13: None,
            translator: None,
            availability_status: AvailabilityStatus::Released,
            release_date: None,
            release_date_precision: DatePrecision::Unknown,
            list_price_amount: None,
            list_price_currency: None,
            collection: CollectionItemView {
                condition: BookCondition::Unknown,
                ..CollectionItemView::default()
            },
            provenance: BTreeMap::from([("displayLabel".to_string(), MetadataSource::Manual)]),
        }],
    }
}
