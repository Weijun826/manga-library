use std::{
    fs,
    path::{Component, Path, PathBuf},
};

use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

use crate::{
    db::{
        models::{CoverAsset, CoverMimeType},
        repository, Database,
    },
    error::AppError,
    state::AppState,
};

const MAX_COVER_BYTES: u64 = 10 * 1024 * 1024;

#[tauri::command]
pub async fn set_series_cover(
    app: AppHandle,
    state: State<'_, AppState>,
    series_id: String,
    source_path: String,
) -> Result<CoverAsset, AppError> {
    let app_data = app.path().app_data_dir().map_err(|_| cover_error())?;
    let database = state.database.clone();
    tauri::async_runtime::spawn_blocking(move || {
        store_series_cover(&database, &app_data, &series_id, Path::new(&source_path))
    })
    .await
    .map_err(|_| cover_error())?
}

#[tauri::command]
pub async fn remove_series_cover_command(
    app: AppHandle,
    state: State<'_, AppState>,
    series_id: String,
) -> Result<(), AppError> {
    let app_data = app.path().app_data_dir().map_err(|_| cover_error())?;
    let database = state.database.clone();
    tauri::async_runtime::spawn_blocking(move || {
        remove_series_cover(&database, &app_data, &series_id)
    })
    .await
    .map_err(|_| cover_error())?
}

pub fn store_series_cover(
    database: &Database,
    app_data: &Path,
    series_id: &str,
    source: &Path,
) -> Result<CoverAsset, AppError> {
    let metadata = fs::metadata(source).map_err(|_| cover_error())?;
    if metadata.len() > MAX_COVER_BYTES {
        return Err(AppError::new(
            "cover_too_large",
            "The cover image is too large.",
        ));
    }
    let bytes = fs::read(source).map_err(|_| cover_error())?;
    let (mime_type, extension) = detect_image(&bytes)?;
    let sha256 = format!("{:x}", Sha256::digest(&bytes));
    let relative_path = format!("covers/{sha256}.{extension}");
    let target = app_data.join(&relative_path);
    let old_asset = repository::get_series_detail(database, series_id)?.representative_cover;
    let created_file = if target.exists() {
        false
    } else {
        let directory = target.parent().ok_or_else(cover_error)?;
        fs::create_dir_all(directory).map_err(|_| cover_error())?;
        let temporary = directory.join(format!(".{}.tmp", Uuid::new_v4()));
        fs::write(&temporary, &bytes).map_err(|_| cover_error())?;
        if let Err(error) = fs::rename(&temporary, &target) {
            let _ = fs::remove_file(&temporary);
            return Err(AppError::new("cover_copy_failed", error.to_string()));
        }
        true
    };
    let result = repository::set_series_cover_asset(
        database,
        series_id,
        &Uuid::new_v4().to_string(),
        &relative_path,
        mime_type,
        bytes.len() as i64,
        &sha256,
    );
    let asset = match result {
        Ok(asset) => asset,
        Err(error) => {
            if created_file {
                let _ = fs::remove_file(&target);
            }
            return Err(error);
        }
    };
    if let Some(old) = old_asset.filter(|old| old.id != asset.id) {
        cleanup_asset(database, app_data, &old)?;
    }
    Ok(asset)
}

pub fn remove_series_cover(
    database: &Database,
    app_data: &Path,
    series_id: &str,
) -> Result<(), AppError> {
    if let Some(asset) = repository::clear_series_cover(database, series_id)? {
        cleanup_asset(database, app_data, &asset)?;
    }
    Ok(())
}

fn cleanup_asset(database: &Database, app_data: &Path, asset: &CoverAsset) -> Result<(), AppError> {
    if repository::delete_cover_asset_if_unreferenced(database, &asset.id)? {
        if let Some(path) = managed_cover_path(app_data, &asset.relative_path) {
            let _ = fs::remove_file(path);
        }
    }
    Ok(())
}

fn managed_cover_path(app_data: &Path, relative: &str) -> Option<PathBuf> {
    let path = Path::new(relative);
    let mut components = path.components();
    match (components.next(), components.next(), components.next()) {
        (Some(Component::Normal(folder)), Some(Component::Normal(file)), None)
            if folder == "covers" && !file.is_empty() =>
        {
            Some(app_data.join(path))
        }
        _ => None,
    }
}

fn detect_image(bytes: &[u8]) -> Result<(CoverMimeType, &'static str), AppError> {
    if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        Ok((CoverMimeType::Jpeg, "jpg"))
    } else if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        Ok((CoverMimeType::Png, "png"))
    } else if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        Ok((CoverMimeType::Webp, "webp"))
    } else {
        Err(AppError::new(
            "invalid_cover_image",
            "The selected file is not a supported image.",
        ))
    }
}

fn cover_error() -> AppError {
    AppError::new(
        "cover_operation_failed",
        "Unable to store the selected cover image.",
    )
}
