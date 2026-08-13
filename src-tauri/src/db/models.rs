use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub(crate) struct Migration {
    pub version: i64,
    pub name: &'static str,
    pub sql: &'static str,
}

macro_rules! string_enum {
    (
        $(#[$meta:meta])*
        pub enum $name:ident {
            $($variant:ident => $value:literal),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        pub enum $name {
            $(#[serde(rename = $value)] $variant),+
        }
    };
}

macro_rules! impl_as_str {
    ($name:ident { $($variant:ident => $value:literal),+ $(,)? }) => {
        impl $name {
            pub(crate) const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $value),+
                }
            }
        }
    };
}

macro_rules! impl_from_db {
    ($name:ident { $($variant:ident => $value:literal),+ $(,)? }) => {
        impl $name {
            pub(crate) fn from_db(value: &str) -> rusqlite::Result<Self> {
                match value {
                    $($value => Ok(Self::$variant)),+,
                    _ => Err(rusqlite::Error::InvalidQuery),
                }
            }
        }
    };
}

string_enum! {
    pub enum PublicationStatus {
        Ongoing => "ongoing",
        Completed => "completed",
        Hiatus => "hiatus",
        Unknown => "unknown",
    }
}

impl_as_str! {
    PublicationStatus {
        Ongoing => "ongoing",
        Completed => "completed",
        Hiatus => "hiatus",
        Unknown => "unknown",
    }
}

impl_from_db! {
    PublicationStatus {
        Ongoing => "ongoing",
        Completed => "completed",
        Hiatus => "hiatus",
        Unknown => "unknown",
    }
}

impl CoverMimeType {
    pub const fn content_type(&self) -> &'static str {
        match self {
            Self::Jpeg => "image/jpeg",
            Self::Png => "image/png",
            Self::Webp => "image/webp",
        }
    }
}

string_enum! {
    pub enum PublicationStatusFilter {
        All => "all",
        Ongoing => "ongoing",
        Completed => "completed",
        Hiatus => "hiatus",
        Unknown => "unknown",
    }
}

impl_as_str! {
    PublicationStatusFilter {
        All => "all",
        Ongoing => "ongoing",
        Completed => "completed",
        Hiatus => "hiatus",
        Unknown => "unknown",
    }
}

string_enum! {
    pub enum CompletionFilter {
        All => "all",
        Complete => "complete",
        Incomplete => "incomplete",
    }
}

impl_as_str! {
    CompletionFilter {
        All => "all",
        Complete => "complete",
        Incomplete => "incomplete",
    }
}

string_enum! {
    pub enum ContributorRole {
        Author => "author",
        Story => "story",
        Art => "art",
    }
}

impl_as_str! {
    ContributorRole {
        Author => "author",
        Story => "story",
        Art => "art",
    }
}

impl_from_db! {
    ContributorRole {
        Author => "author",
        Story => "story",
        Art => "art",
    }
}

string_enum! {
    pub enum EditionFormat {
        Tankobon => "tankobon",
        Omnibus => "omnibus",
        Deluxe => "deluxe",
        BoxSet => "box_set",
        Other => "other",
    }
}

impl_as_str! {
    EditionFormat {
        Tankobon => "tankobon",
        Omnibus => "omnibus",
        Deluxe => "deluxe",
        BoxSet => "box_set",
        Other => "other",
    }
}

impl_from_db! {
    EditionFormat {
        Tankobon => "tankobon",
        Omnibus => "omnibus",
        Deluxe => "deluxe",
        BoxSet => "box_set",
        Other => "other",
    }
}

string_enum! {
    pub enum AvailabilityStatus {
        Released => "released",
        Upcoming => "upcoming",
        Unknown => "unknown",
    }
}

impl_as_str! {
    AvailabilityStatus {
        Released => "released",
        Upcoming => "upcoming",
        Unknown => "unknown",
    }
}

impl_from_db! {
    AvailabilityStatus {
        Released => "released",
        Upcoming => "upcoming",
        Unknown => "unknown",
    }
}

string_enum! {
    pub enum DatePrecision {
        Day => "day",
        Month => "month",
        Year => "year",
        Unknown => "unknown",
    }
}

impl_as_str! {
    DatePrecision {
        Day => "day",
        Month => "month",
        Year => "year",
        Unknown => "unknown",
    }
}

impl_from_db! {
    DatePrecision {
        Day => "day",
        Month => "month",
        Year => "year",
        Unknown => "unknown",
    }
}

string_enum! {
    pub enum BookCondition {
        New => "new",
        LikeNew => "like_new",
        Good => "good",
        Fair => "fair",
        Poor => "poor",
        Unknown => "unknown",
    }
}

impl_as_str! {
    BookCondition {
        New => "new",
        LikeNew => "like_new",
        Good => "good",
        Fair => "fair",
        Poor => "poor",
        Unknown => "unknown",
    }
}

impl_from_db! {
    BookCondition {
        New => "new",
        LikeNew => "like_new",
        Good => "good",
        Fair => "fair",
        Poor => "poor",
        Unknown => "unknown",
    }
}

impl Default for BookCondition {
    fn default() -> Self {
        Self::Unknown
    }
}

string_enum! {
    pub enum MetadataSource {
        Manual => "manual",
        GoogleBooks => "google_books",
        OpenLibrary => "open_library",
        NclImport => "ncl_import",
    }
}

impl_as_str! {
    MetadataSource {
        Manual => "manual",
        GoogleBooks => "google_books",
        OpenLibrary => "open_library",
        NclImport => "ncl_import",
    }
}

string_enum! {
    pub enum CoverMimeType {
        Jpeg => "image/jpeg",
        Png => "image/png",
        Webp => "image/webp",
    }
}

impl_from_db! {
    CoverMimeType {
        Jpeg => "image/jpeg",
        Png => "image/png",
        Webp => "image/webp",
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeriesFilter {
    pub query: String,
    pub publication_status: PublicationStatusFilter,
    pub collection: CompletionFilter,
    pub reading: CompletionFilter,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardSummary {
    pub series_count: i64,
    pub owned_volume_count: i64,
    pub read_volume_count: i64,
    pub missing_volume_count: i64,
    pub recent_volumes: Vec<VolumeWithCollection>,
    pub incomplete_series: Vec<SeriesSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeriesSummary {
    pub id: String,
    pub title: String,
    pub original_title: Option<String>,
    pub publication_status: PublicationStatus,
    pub contributors: Vec<ContributorView>,
    pub publishers: Vec<String>,
    pub representative_cover: Option<CoverAsset>,
    pub known_volume_count: i64,
    pub owned_volume_count: i64,
    pub read_volume_count: i64,
    pub missing_volume_count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContributorView {
    pub name: String,
    pub role: ContributorRole,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverAsset {
    pub id: String,
    pub relative_path: String,
    pub mime_type: CoverMimeType,
    pub byte_size: i64,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CollectionItemView {
    pub is_owned: bool,
    pub is_read: bool,
    pub is_wishlisted: bool,
    pub purchase_price_amount: Option<i64>,
    pub purchase_price_currency: Option<String>,
    pub acquired_on: Option<String>,
    pub condition: BookCondition,
    pub storage_location: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumeWithCollection {
    pub id: String,
    pub edition_id: String,
    pub display_label: String,
    pub sort_key: String,
    pub title_override: Option<String>,
    pub isbn_10: Option<String>,
    pub isbn_13: Option<String>,
    pub translator: Option<String>,
    pub availability_status: AvailabilityStatus,
    pub release_date: Option<String>,
    pub release_date_precision: DatePrecision,
    pub list_price_amount: Option<i64>,
    pub list_price_currency: Option<String>,
    pub cover: Option<CoverAsset>,
    pub collection: CollectionItemView,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditionDetail {
    pub id: String,
    pub name: String,
    pub language_code: String,
    pub region_code: String,
    pub publisher: String,
    pub format: EditionFormat,
    pub release_status: PublicationStatus,
    pub known_volume_count: Option<i64>,
    pub volumes: Vec<VolumeWithCollection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeriesDetail {
    pub id: String,
    pub title: String,
    pub original_title: Option<String>,
    pub publication_status: PublicationStatus,
    pub contributors: Vec<ContributorView>,
    pub publishers: Vec<String>,
    pub representative_cover: Option<CoverAsset>,
    pub known_volume_count: i64,
    pub owned_volume_count: i64,
    pub read_volume_count: i64,
    pub missing_volume_count: i64,
    pub description: Option<String>,
    pub editions: Vec<EditionDetail>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSeriesBatchInput {
    pub series: CreateSeriesInput,
    pub edition: CreateEditionInput,
    pub volumes: Vec<CreateVolumeInput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSeriesInput {
    pub title: String,
    pub original_title: Option<String>,
    pub description: Option<String>,
    pub publication_status: PublicationStatus,
    pub contributors: Vec<ContributorInput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContributorInput {
    pub name: String,
    pub role: ContributorRole,
    pub sort_order: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEditionInput {
    pub name: String,
    pub language_code: String,
    pub region_code: String,
    pub publisher: String,
    pub format: EditionFormat,
    pub release_status: PublicationStatus,
    pub known_volume_count: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateVolumeInput {
    pub display_label: String,
    pub sort_key: String,
    pub title_override: Option<String>,
    pub isbn_10: Option<String>,
    pub isbn_13: Option<String>,
    pub translator: Option<String>,
    pub availability_status: AvailabilityStatus,
    pub release_date: Option<String>,
    pub release_date_precision: DatePrecision,
    pub list_price_amount: Option<i64>,
    pub list_price_currency: Option<String>,
    pub collection: CollectionItemView,
    pub provenance: BTreeMap<String, MetadataSource>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSeriesMetadataInput {
    pub title: String,
    pub original_title: Option<String>,
    pub description: Option<String>,
    pub publication_status: PublicationStatus,
    pub author: String,
    pub edition_id: String,
    pub edition_name: String,
    pub publisher: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddVolumeInput {
    pub display_label: String,
    pub isbn: Option<String>,
    pub availability_status: AvailabilityStatus,
    pub collection: CollectionItemView,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CollectionItemPatch {
    pub is_owned: Option<bool>,
    pub is_read: Option<bool>,
    pub is_wishlisted: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "double_option::deserialize"
    )]
    pub purchase_price_amount: Option<Option<i64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "double_option::deserialize"
    )]
    pub purchase_price_currency: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "double_option::deserialize"
    )]
    pub acquired_on: Option<Option<String>>,
    pub condition: Option<BookCondition>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "double_option::deserialize"
    )]
    pub storage_location: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "double_option::deserialize"
    )]
    pub notes: Option<Option<String>>,
}

mod double_option {
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de>,
    {
        Option::<T>::deserialize(deserializer).map(Some)
    }
}
