pub mod models;
pub mod repository;

use std::{
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
};

use rusqlite::{Connection, Transaction, TransactionBehavior};

use crate::error::AppError;
use models::Migration;

const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    name: "initial",
    sql: include_str!("../../migrations/0001_initial.sql"),
}];

#[derive(Clone)]
pub struct Database {
    connection: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, AppError> {
        let connection = Connection::open(path).map_err(|_| AppError::database_open())?;
        Self::initialize(connection)
    }

    pub fn in_memory() -> Result<Self, AppError> {
        let connection = Connection::open_in_memory().map_err(|_| AppError::database_open())?;
        Self::initialize(connection)
    }

    fn initialize(connection: Connection) -> Result<Self, AppError> {
        connection
            .execute_batch("PRAGMA foreign_keys = ON;")
            .map_err(|_| AppError::database_operation())?;

        let database = Self {
            connection: Arc::new(Mutex::new(connection)),
        };
        database.apply_migrations()?;
        Ok(database)
    }

    fn apply_migrations(&self) -> Result<(), AppError> {
        let mut connection = self.lock_connection()?;
        connection
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS schema_migrations (\
                    version INTEGER PRIMARY KEY NOT NULL,\
                    name TEXT NOT NULL UNIQUE,\
                    applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP\
                );",
            )
            .map_err(|_| AppError::database_operation())?;

        for migration in MIGRATIONS {
            let transaction = connection
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .map_err(|_| AppError::database_operation())?;
            let already_applied = transaction
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = ?1)",
                    [migration.version],
                    |row| row.get::<_, bool>(0),
                )
                .map_err(|_| AppError::database_operation())?;

            if !already_applied {
                transaction
                    .execute_batch(migration.sql)
                    .map_err(|_| AppError::database_operation())?;
                transaction
                    .execute(
                        "INSERT INTO schema_migrations (version, name) VALUES (?1, ?2)",
                        (migration.version, migration.name),
                    )
                    .map_err(|_| AppError::database_operation())?;
            }

            transaction
                .commit()
                .map_err(|_| AppError::database_operation())?;
        }

        Ok(())
    }

    pub fn with_transaction<T, F>(&self, operation: F) -> Result<T, AppError>
    where
        F: FnOnce(&Transaction<'_>) -> rusqlite::Result<T>,
    {
        let mut connection = self.lock_connection()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|_| AppError::database_operation())?;
        let value = operation(&transaction).map_err(|_| AppError::database_operation())?;
        transaction
            .commit()
            .map_err(|_| AppError::database_operation())?;
        Ok(value)
    }

    pub fn backup_to(&self, destination: &Path) -> Result<(), AppError> {
        let source = self.lock_connection()?;
        let mut destination =
            Connection::open(destination).map_err(|_| AppError::backup_failed())?;
        let backup = rusqlite::backup::Backup::new(&source, &mut destination)
            .map_err(|_| AppError::backup_failed())?;
        backup
            .run_to_completion(5, Duration::from_millis(50), None)
            .map_err(|_| AppError::backup_failed())
    }

    pub fn table_names(&self) -> Result<Vec<String>, AppError> {
        let connection = self.lock_connection()?;
        let mut statement = connection
            .prepare("SELECT name FROM sqlite_schema WHERE type = 'table' ORDER BY name")
            .map_err(|_| AppError::database_operation())?;
        let rows = statement
            .query_map([], |row| row.get(0))
            .map_err(|_| AppError::database_operation())?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|_| AppError::database_operation())
    }

    pub fn foreign_keys_enabled(&self) -> Result<bool, AppError> {
        let connection = self.lock_connection()?;
        connection
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .map_err(|_| AppError::database_operation())
    }

    fn lock_connection(&self) -> Result<std::sync::MutexGuard<'_, Connection>, AppError> {
        self.connection
            .lock()
            .map_err(|_| AppError::database_operation())
    }
}
