use tauri::State;

use crate::{
    db::{
        models::{
            CollectionItemPatch, CreateSeriesBatchInput, DashboardSummary, SeriesDetail,
            SeriesFilter, SeriesSummary, VolumeWithCollection,
        },
        repository, Database,
    },
    error::AppError,
    state::AppState,
};

#[tauri::command]
pub async fn get_dashboard(state: State<'_, AppState>) -> Result<DashboardSummary, AppError> {
    let database = database_handle(state);
    run_database_operation(move || repository::get_dashboard(&database)).await
}

#[tauri::command]
pub async fn list_series(
    state: State<'_, AppState>,
    filter: SeriesFilter,
) -> Result<Vec<SeriesSummary>, AppError> {
    let database = database_handle(state);
    run_database_operation(move || repository::list_series(&database, filter)).await
}

#[tauri::command]
pub async fn get_series_detail(
    state: State<'_, AppState>,
    series_id: String,
) -> Result<SeriesDetail, AppError> {
    let database = database_handle(state);
    run_database_operation(move || repository::get_series_detail(&database, &series_id)).await
}

#[tauri::command]
pub async fn create_series_batch(
    state: State<'_, AppState>,
    input: CreateSeriesBatchInput,
) -> Result<SeriesDetail, AppError> {
    let database = database_handle(state);
    run_database_operation(move || repository::create_series_batch(&database, input)).await
}

#[tauri::command]
pub async fn update_collection_item(
    state: State<'_, AppState>,
    volume_id: String,
    patch: CollectionItemPatch,
) -> Result<VolumeWithCollection, AppError> {
    let database = database_handle(state);
    run_database_operation(move || repository::update_collection_item(&database, &volume_id, patch))
        .await
}

#[tauri::command]
pub async fn find_volume_by_isbn(
    state: State<'_, AppState>,
    isbn: String,
) -> Result<Option<VolumeWithCollection>, AppError> {
    let database = database_handle(state);
    run_database_operation(move || repository::find_volume_by_isbn(&database, &isbn)).await
}

#[tauri::command]
pub async fn delete_series(state: State<'_, AppState>, series_id: String) -> Result<(), AppError> {
    let database = database_handle(state);
    run_database_operation(move || repository::delete_series(&database, &series_id)).await
}

fn database_handle(state: State<'_, AppState>) -> Database {
    state.database.clone()
}

async fn run_database_operation<T, F>(operation: F) -> Result<T, AppError>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, AppError> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(operation)
        .await
        .map_err(|_| {
            AppError::new(
                "background_task_failed",
                "The background database operation failed.",
            )
        })?
}
