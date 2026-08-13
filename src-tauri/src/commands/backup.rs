use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use rusqlite::Connection;
use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

use crate::{db::Database, error::AppError, state::AppState};

const BACKUP_PREFIX: &str = "library-before-";
const BACKUP_SUFFIX: &str = ".sqlite3";
const BACKUPS_TO_KEEP: usize = 3;

fn valid_version(version: &str) -> bool {
    !version.is_empty()
        && version
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'+'))
}

fn backup_timestamp(path: &Path) -> Option<u64> {
    let name = path.file_name()?.to_str()?;
    let stem = name
        .strip_prefix(BACKUP_PREFIX)?
        .strip_suffix(BACKUP_SUFFIX)?;
    let (without_uuid, _) = stem.rsplit_once('-')?;
    let (_, timestamp) = without_uuid.rsplit_once('-')?;
    timestamp.parse().ok()
}

fn rotate_backups(backup_directory: &Path) -> Result<(), AppError> {
    let mut backups = fs::read_dir(backup_directory)
        .map_err(|_| AppError::backup_failed())?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter_map(|path| backup_timestamp(&path).map(|timestamp| (timestamp, path)))
        .collect::<Vec<_>>();
    backups.sort_by(|left, right| left.cmp(right));

    let remove_count = backups.len().saturating_sub(BACKUPS_TO_KEEP);
    for (_, path) in backups.into_iter().take(remove_count) {
        fs::remove_file(path).map_err(|_| AppError::backup_failed())?;
    }
    Ok(())
}

pub fn create_update_backup(
    database: &Database,
    backup_directory: &Path,
    current_version: &str,
    timestamp: u64,
) -> Result<PathBuf, AppError> {
    if !valid_version(current_version) {
        return Err(AppError::backup_failed());
    }

    fs::create_dir_all(backup_directory).map_err(|_| AppError::backup_failed())?;
    let identifier = Uuid::new_v4().simple().to_string();
    let final_path = backup_directory.join(format!(
        "{BACKUP_PREFIX}{current_version}-{timestamp}-{identifier}{BACKUP_SUFFIX}"
    ));
    let pending_path = final_path.with_extension("sqlite3.partial");

    if let Err(error) = database.backup_to(&pending_path) {
        let _ = fs::remove_file(&pending_path);
        return Err(error);
    }

    let integrity_is_valid = (|| {
        let backup = Connection::open(&pending_path).map_err(|_| AppError::backup_failed())?;
        let result = backup
            .query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0))
            .map_err(|_| AppError::backup_failed())?;
        Ok::<bool, AppError>(result == "ok")
    })()?;
    if !integrity_is_valid {
        let _ = fs::remove_file(&pending_path);
        return Err(AppError::backup_failed());
    }

    fs::rename(&pending_path, &final_path).map_err(|_| AppError::backup_failed())?;
    rotate_backups(backup_directory)?;
    Ok(final_path)
}

#[tauri::command]
pub async fn prepare_update_backup(
    state: State<'_, AppState>,
    app: AppHandle,
    current_version: String,
) -> Result<(), AppError> {
    let database = state.database.clone();
    let backup_directory = app
        .path()
        .app_data_dir()
        .map_err(|_| AppError::backup_failed())?
        .join("backups");

    tauri::async_runtime::spawn_blocking(move || {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| AppError::backup_failed())?
            .as_secs();
        create_update_backup(&database, &backup_directory, &current_version, timestamp)?;
        Ok(())
    })
    .await
    .map_err(|_| AppError::backup_failed())?
}
